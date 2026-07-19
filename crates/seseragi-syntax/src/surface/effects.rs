use super::SurfaceParser;
use crate::surface_model::{
    ByteSpan, SurfaceDecl, SurfaceParameter, SurfaceRequirement, TypeRef, Visibility,
};
use crate::token::TokenKind;

#[cfg(test)]
mod tests;

impl SurfaceParser<'_> {
    pub(super) fn parse_effect_fn_decl(
        &self,
        visibility: Visibility,
        top_start: usize,
        decl_start: usize,
        end: usize,
    ) -> Option<SurfaceDecl> {
        let fn_index =
            self.find_significant_token(decl_start + 1, end, |kind| kind == TokenKind::KeywordFn)?;
        let name_index = self.next_significant_token(fn_index + 1, end)?;
        let name = self.identifier_name_at(name_index)?;
        let (type_parameters, after_type_parameters) =
            self.parse_optional_type_parameters(name_index + 1, end);
        let equals = self.find_significant_token(after_type_parameters, end, |kind| {
            kind == TokenKind::OperatorEquals
        });
        let header_end = equals.unwrap_or(end);
        let arrow = (after_type_parameters..header_end)
            .rev()
            .find(|index| self.kind_at(*index) == Some(TokenKind::OperatorArrow));
        let with = self.find_significant_token(after_type_parameters, header_end, |kind| {
            kind == TokenKind::KeywordWith
        });
        let fails = self.find_significant_token(after_type_parameters, header_end, |kind| {
            kind == TokenKind::KeywordFails
        });
        let where_index = self.find_raw(after_type_parameters, header_end, "where");
        let return_end = [with, fails, where_index]
            .into_iter()
            .flatten()
            .filter(|index| arrow.is_none_or(|arrow| *index > arrow))
            .min()
            .unwrap_or(header_end);
        let curried_signature =
            arrow.and_then(|_| self.parse_curried_signature(after_type_parameters, return_end));
        let parameter_end = [arrow, with, fails, where_index]
            .into_iter()
            .flatten()
            .min()
            .unwrap_or(header_end);
        let parameters = curried_signature
            .as_ref()
            .map(|(parameters, _)| parameters.clone())
            .unwrap_or_else(|| self.parse_effect_parameters(after_type_parameters, parameter_end));
        let return_type = curried_signature
            .map(|(_, return_type)| return_type)
            .or_else(|| arrow.and_then(|arrow| self.parse_type_name(arrow + 1, return_end)));
        let requirements = with
            .map(|with| {
                let requirement_end = [fails, where_index]
                    .into_iter()
                    .flatten()
                    .filter(|index| *index > with)
                    .min()
                    .unwrap_or(header_end);
                self.parse_effect_requirements(with + 1, requirement_end)
            })
            .unwrap_or_default();
        let failure = fails.and_then(|fails| {
            let failure_end = where_index
                .filter(|where_index| *where_index > fails)
                .unwrap_or(header_end);
            self.parse_type_name(fails + 1, failure_end)
        });
        let constraints = where_index
            .map(|where_index| self.parse_constraints(where_index + 1, header_end))
            .unwrap_or_default();
        let body = equals.and_then(|equals| self.parse_expression(equals + 1, end));

        Some(SurfaceDecl::EffectFn {
            visibility,
            name,
            name_span: self.byte_span(name_index)?,
            type_parameters,
            parameters,
            inferred_contract: arrow.is_none(),
            return_type,
            requirements,
            failure,
            constraints,
            body,
            span: self.declaration_span(top_start, end)?,
        })
    }

    fn parse_effect_parameters(&self, start: usize, end: usize) -> Vec<SurfaceParameter> {
        let mut parameters = Vec::new();
        let mut cursor = start;
        while let Some(name_index) = self.next_significant_token(cursor, end) {
            if self.kind_at(name_index) != Some(TokenKind::IdentifierLower) {
                cursor = name_index + 1;
                continue;
            }
            let Some(colon) = self.next_significant_token(name_index + 1, end) else {
                break;
            };
            if self.kind_at(colon) != Some(TokenKind::PunctuationColon) {
                cursor = colon + 1;
                continue;
            }
            let Some((type_ref, after_type)) = self.parse_type_ref(colon + 1, end) else {
                break;
            };
            parameters.push(SurfaceParameter {
                name: self.tokens[name_index].raw.clone(),
                name_span: self.byte_span(name_index).unwrap_or(ByteSpan {
                    start: self.tokens[name_index].start,
                    end: self.tokens[name_index].end,
                }),
                type_ref,
            });
            cursor = after_type;
        }
        parameters
    }

    fn parse_effect_requirements(&self, start: usize, end: usize) -> Vec<SurfaceRequirement> {
        let mut requirements = Vec::new();
        let mut cursor = start;
        while let Some(name_index) = self.next_significant_token(cursor, end) {
            let Some(name) = self.identifier_name_at(name_index) else {
                cursor = name_index + 1;
                continue;
            };
            let name_span = self.byte_span(name_index).unwrap_or(ByteSpan {
                start: self.tokens[name_index].start,
                end: self.tokens[name_index].end,
            });
            let after_name = self.next_significant_token(name_index + 1, end);
            if after_name
                .is_some_and(|index| self.kind_at(index) == Some(TokenKind::PunctuationColon))
            {
                let colon = after_name.expect("checked above");
                if let Some((type_ref, after_type)) = self.parse_type_ref(colon + 1, end) {
                    let type_span = type_ref_span(&type_ref);
                    requirements.push(SurfaceRequirement::Field {
                        name,
                        name_span,
                        type_ref,
                        span: ByteSpan {
                            start: name_span.start,
                            end: type_span.end,
                        },
                    });
                    cursor = skip_comma(self, after_type, end);
                    continue;
                }
            }
            requirements.push(SurfaceRequirement::Shorthand {
                name,
                span: name_span,
            });
            cursor = skip_comma(self, name_index + 1, end);
        }
        requirements
    }
}

fn skip_comma(parser: &SurfaceParser<'_>, start: usize, end: usize) -> usize {
    parser
        .next_significant_token(start, end)
        .filter(|index| parser.kind_at(*index) == Some(TokenKind::PunctuationComma))
        .map_or(start, |comma| comma + 1)
}

fn type_ref_span(type_ref: &TypeRef) -> ByteSpan {
    match type_ref {
        TypeRef::Named { span, .. }
        | TypeRef::Hole { span }
        | TypeRef::Record { span, .. }
        | TypeRef::Tuple { span, .. }
        | TypeRef::Function { span, .. } => *span,
    }
}
