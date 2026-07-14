use super::SurfaceParser;
use crate::surface_model::SurfaceDecl;
use crate::token::TokenKind;

impl SurfaceParser<'_> {
    pub(super) fn parse_instance_decl(
        &self,
        top_start: usize,
        decl_start: usize,
        end: usize,
    ) -> Option<SurfaceDecl> {
        let (type_parameters, after_type_parameters) =
            self.parse_optional_type_parameters(decl_start + 1, end);
        let trait_index = self.next_significant_token(after_type_parameters, end)?;
        let trait_name = self.identifier_name_at(trait_index)?;
        let mut arguments = Vec::new();
        let mut after_head = trait_index + 1;
        let next = self.next_significant_token(after_head, end);
        if next.is_some_and(|index| self.is_angle_left(index)) {
            let (parsed, closing_angle) = self.parse_type_arguments(next? + 1, end)?;
            arguments = parsed;
            after_head = closing_angle + 1;
        }

        let body_start = self
            .find_significant_token(after_head, end, |kind| {
                kind == TokenKind::PunctuationBraceLeft
            })
            .unwrap_or(end);
        let constraints = self
            .find_raw(after_head, body_start, "where")
            .map(|where_index| self.parse_constraints(where_index + 1, body_start))
            .unwrap_or_default();
        let methods = self.parse_instance_methods(body_start, end);

        Some(SurfaceDecl::Instance {
            type_parameters,
            trait_name,
            arguments,
            constraints,
            methods,
            span: self.declaration_span(top_start, end)?,
        })
    }
}
