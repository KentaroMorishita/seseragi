use super::SurfaceParser;
use crate::surface_model::{SurfaceDecl, Visibility};
use crate::token::TokenKind;

impl SurfaceParser<'_> {
    pub(super) fn parse_fn_decl(
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
        let equals = self.find_significant_token(after_type_parameters, end, |kind| {
            kind == TokenKind::OperatorEquals
        })?;
        let where_index = self.find_raw(after_type_parameters, equals, "where");
        let signature_end = where_index.unwrap_or(equals);
        let constraints = where_index
            .map(|where_index| self.parse_constraint_names(where_index + 1, equals))
            .unwrap_or_default();
        let (parameters, return_type) =
            self.parse_curried_signature(after_type_parameters, signature_end)?;

        Some(SurfaceDecl::Fn {
            visibility,
            name,
            name_span: self.byte_span(name_index)?,
            type_parameters,
            parameters,
            return_type,
            constraints,
            span: self.declaration_span(top_start, end)?,
        })
    }
}
