use super::SurfaceParser;
use crate::surface_model::{SurfaceDecl, Visibility};
use crate::token::TokenKind;

impl SurfaceParser<'_> {
    pub(super) fn parse_newtype_decl(
        &self,
        visibility: Visibility,
        top_start: usize,
        decl_start: usize,
        end: usize,
    ) -> Option<SurfaceDecl> {
        let name_index = self.next_significant_token(decl_start + 1, end)?;
        let name = self.identifier_name_at(name_index)?;
        let equals = self.find_significant_token(name_index + 1, end, |kind| {
            kind == TokenKind::OperatorEquals
        })?;
        let representation = self.parse_type_name(equals + 1, end)?;

        Some(SurfaceDecl::Newtype {
            visibility,
            name,
            name_span: self.byte_span(name_index)?,
            representation,
            span: self.declaration_span(top_start, end)?,
        })
    }

    pub(super) fn parse_alias_decl(
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
        let target = self.parse_type_name(equals + 1, end)?;

        Some(SurfaceDecl::Alias {
            visibility,
            name,
            name_span: self.byte_span(name_index)?,
            type_parameters,
            target,
            span: self.declaration_span(top_start, end)?,
        })
    }
}
