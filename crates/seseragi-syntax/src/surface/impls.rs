use super::SurfaceParser;
use crate::surface_model::{SurfaceDecl, SurfaceImplMember, SurfaceMethod, Visibility};
use crate::token::TokenKind;

impl SurfaceParser<'_> {
    pub(super) fn parse_impl_decl(
        &self,
        top_start: usize,
        decl_start: usize,
        end: usize,
    ) -> Option<SurfaceDecl> {
        let (type_parameters, after_type_parameters) =
            self.parse_optional_type_parameters(decl_start + 1, end);
        let target_start = self.next_significant_token(after_type_parameters, end)?;
        let (target, after_target) = self.parse_type_ref(target_start, end)?;
        let body_start = self.find_significant_token(after_target, end, |kind| {
            kind == TokenKind::PunctuationBraceLeft
        })?;
        let constraints = self
            .find_raw(after_target, body_start, "where")
            .map(|where_index| self.parse_constraints(where_index + 1, body_start))
            .unwrap_or_default();

        Some(SurfaceDecl::Impl {
            type_parameters,
            target,
            constraints,
            members: self
                .impl_member_ranges(body_start, end)
                .into_iter()
                .filter_map(|(start, member_end)| self.parse_impl_member(start, member_end))
                .collect(),
            span: self.declaration_span(top_start, end)?,
        })
    }

    fn parse_impl_member(&self, start: usize, end: usize) -> Option<SurfaceImplMember> {
        let prefix = crate::declaration::declaration_prefix_in(
            self.tokens,
            start,
            end,
            crate::declaration::DeclarationContext::Member,
        )
        .ok()?;
        let visibility = if prefix.public {
            Visibility::Public
        } else {
            Visibility::Private
        };
        let decl_start = prefix.keyword;

        if self.kind_at(decl_start) == Some(TokenKind::KeywordFn) {
            let declaration = self.parse_fn_decl(visibility, start, decl_start, end)?;
            let SurfaceDecl::Fn {
                name,
                name_span,
                type_parameters,
                parameters,
                return_type,
                constraints,
                body,
                span,
                ..
            } = declaration
            else {
                return None;
            };
            return Some(SurfaceImplMember::Method {
                visibility,
                method: SurfaceMethod {
                    name,
                    name_span,
                    type_parameters,
                    parameters,
                    return_type,
                    constraints,
                    body,
                    span,
                },
            });
        }

        if self.raw_at(decl_start) != Some("operator") {
            return None;
        }
        self.parse_impl_operator(visibility, start, decl_start, end)
    }

    fn parse_impl_operator(
        &self,
        visibility: Visibility,
        top_start: usize,
        decl_start: usize,
        end: usize,
    ) -> Option<SurfaceImplMember> {
        let spelling_start = self.next_significant_token(decl_start + 1, end)?;
        let spelling_span_start = self.byte_span(spelling_start)?.start;
        let (spelling, after_spelling) = self.operator_spelling(spelling_start)?;
        let self_index = self.next_significant_token(after_spelling, end)?;
        if self.raw_at(self_index) != Some("self") {
            return None;
        }
        let self_span = self.byte_span(self_index)?;
        let arrow = self.next_significant_token(self_index + 1, end)?;
        if self.kind_at(arrow) != Some(TokenKind::OperatorArrow) {
            return None;
        }
        let equals =
            self.find_significant_token(arrow + 1, end, |kind| kind == TokenKind::OperatorEquals)?;
        let (parameters, return_type) = self.parse_curried_signature(arrow + 1, equals)?;
        let spelling_end = self.previous_significant_token(after_spelling)?;

        Some(SurfaceImplMember::Operator {
            visibility,
            spelling,
            spelling_span: crate::ByteSpan {
                start: spelling_span_start,
                end: self.byte_span(spelling_end)?.end,
            },
            self_span,
            parameters,
            return_type,
            body: self.parse_expression(equals + 1, end),
            span: self.declaration_span(top_start, end)?,
        })
    }

    fn impl_member_ranges(&self, body_start: usize, end: usize) -> Vec<(usize, usize)> {
        self.method_ranges(body_start, end)
    }
}
