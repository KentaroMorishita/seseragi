use super::{SurfaceParser, TypeRef};
use crate::surface_model::{SurfaceDecl, SurfaceParameter, Visibility};
use crate::token::TokenKind;

impl SurfaceParser<'_> {
    pub(super) fn parse_operator_decl(
        &self,
        visibility: Visibility,
        top_start: usize,
        decl_start: usize,
        end: usize,
    ) -> Option<SurfaceDecl> {
        let (type_parameters, after_type_parameters) =
            self.parse_optional_type_parameters(decl_start + 1, end);
        let fixity_index = self.next_significant_token(after_type_parameters, end)?;
        let precedence_index = self.next_significant_token(fixity_index + 1, end)?;
        let precedence = self.tokens.get(precedence_index)?.raw.parse::<u32>().ok()?;
        let spelling_start = self.next_significant_token(precedence_index + 1, end)?;
        let (spelling, after_spelling) = self.operator_spelling(spelling_start)?;
        let equals = self.find_significant_token(after_spelling, end, |kind| {
            kind == TokenKind::OperatorEquals
        })?;
        let where_index = self.find_raw(after_spelling, equals, "where");
        let signature_end = where_index.unwrap_or(equals);
        let constraints = where_index
            .map(|where_index| self.parse_constraint_names(where_index + 1, equals))
            .unwrap_or_default();
        let (parameters, return_type) =
            self.parse_operator_signature(after_spelling, signature_end)?;

        Some(SurfaceDecl::Operator {
            visibility,
            type_parameters,
            fixity: self.tokens.get(fixity_index)?.raw.clone(),
            precedence,
            spelling,
            parameters,
            return_type,
            constraints,
            span: self.declaration_span(top_start, end)?,
        })
    }

    fn parse_operator_signature(
        &self,
        start: usize,
        end: usize,
    ) -> Option<(Vec<SurfaceParameter>, TypeRef)> {
        let mut parameters = Vec::new();
        let mut cursor = start;

        loop {
            let name_index = self.next_significant_token(cursor, end)?;
            let name = self.identifier_name_at(name_index)?;
            let colon = self.find_significant_token(name_index + 1, end, |kind| {
                kind == TokenKind::PunctuationColon
            })?;
            let (type_ref, after_type) = self.parse_type_ref(colon + 1, end)?;
            let arrow = self.next_significant_token(after_type, end)?;
            if self.kind_at(arrow) != Some(TokenKind::OperatorArrow) {
                return None;
            }
            parameters.push(SurfaceParameter {
                name,
                name_span: self.byte_span(name_index)?,
                type_ref,
            });
            cursor = arrow + 1;

            let next = self.next_significant_token(cursor, end)?;
            let next_after_type = self.parse_type_ref(next, end);
            let Some((return_type, after_return_type)) = next_after_type else {
                continue;
            };
            if self
                .next_significant_token(after_return_type, end)
                .is_none()
            {
                return Some((parameters, return_type));
            }
        }
    }

    pub(super) fn operator_spelling(&self, start: usize) -> Option<(String, usize)> {
        let first = self.tokens.get(start)?;
        if !is_operator_spelling_token(first.kind) {
            return None;
        }
        let mut spelling = first.raw.clone();
        let mut cursor = start + 1;
        let mut previous_end = first.end;
        while let Some(token) = self.tokens.get(cursor) {
            if token.start != previous_end || !is_operator_spelling_token(token.kind) {
                break;
            }
            spelling.push_str(&token.raw);
            previous_end = token.end;
            cursor += 1;
        }
        Some((spelling, cursor))
    }
}

fn is_operator_spelling_token(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::OperatorArithmetic
            | TokenKind::OperatorComparison
            | TokenKind::OperatorCustom
            | TokenKind::OperatorPipeline
            | TokenKind::OperatorBind
            | TokenKind::OperatorApply
            | TokenKind::OperatorRangeExclusive
            | TokenKind::OperatorRangeInclusive
    )
}
