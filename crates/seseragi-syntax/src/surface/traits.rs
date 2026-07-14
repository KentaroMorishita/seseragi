use super::SurfaceParser;
use crate::surface_model::{SurfaceDecl, Visibility};
use crate::token::TokenKind;

impl SurfaceParser<'_> {
    pub(super) fn parse_trait_decl(
        &self,
        visibility: Visibility,
        top_start: usize,
        decl_start: usize,
        end: usize,
    ) -> Option<SurfaceDecl> {
        let name_index = self.next_significant_token(decl_start + 1, end)?;
        let name = self.identifier_name_at(name_index)?;
        let (type_parameters, after_type_parameters) =
            self.parse_optional_type_parameters(name_index + 1, end);
        let body_start = self
            .find_significant_token(after_type_parameters, end, |kind| {
                kind == TokenKind::PunctuationBraceLeft
            })
            .unwrap_or(end);
        let constraints = self
            .find_raw(after_type_parameters, body_start, "where")
            .map(|where_index| self.parse_constraints(where_index + 1, body_start))
            .unwrap_or_default();
        let methods = self.parse_trait_methods(body_start, end);

        Some(SurfaceDecl::Trait {
            visibility,
            name,
            name_span: self.byte_span(name_index)?,
            type_parameters,
            constraints,
            methods,
            span: self.declaration_span(top_start, end)?,
        })
    }
}
