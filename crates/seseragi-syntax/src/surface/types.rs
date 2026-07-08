use super::SurfaceParser;
use crate::surface_model::{SurfaceDecl, SurfaceField, Visibility};
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

    pub(super) fn parse_struct_decl(
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
        let body_start = self.find_significant_token(after_type_parameters, end, |kind| {
            kind == TokenKind::PunctuationBraceLeft
        })?;
        let body_end = self
            .find_significant_token(body_start + 1, end, |kind| {
                kind == TokenKind::PunctuationBraceRight
            })
            .unwrap_or(end);

        Some(SurfaceDecl::Struct {
            visibility,
            name,
            name_span: self.byte_span(name_index)?,
            type_parameters,
            fields: self.parse_struct_fields(body_start + 1, body_end),
            span: self.declaration_span(top_start, end)?,
        })
    }

    fn parse_struct_fields(&self, start: usize, end: usize) -> Vec<SurfaceField> {
        let mut fields = Vec::new();
        let mut cursor = start;
        while let Some(name_index) = self.next_significant_token(cursor, end) {
            let Some(name) = self.identifier_name_at(name_index) else {
                cursor = name_index + 1;
                continue;
            };
            let Some(colon) = self.find_significant_token(name_index + 1, end, |kind| {
                kind == TokenKind::PunctuationColon
            }) else {
                break;
            };
            let Some((type_ref, after_type)) = self.parse_type_ref(colon + 1, end) else {
                break;
            };
            let Some(name_span) = self.byte_span(name_index) else {
                break;
            };
            fields.push(SurfaceField {
                name,
                name_span,
                type_ref,
            });
            cursor = self
                .next_significant_token(after_type, end)
                .filter(|separator| self.kind_at(*separator) == Some(TokenKind::PunctuationComma))
                .map(|separator| separator + 1)
                .unwrap_or(after_type);
        }
        fields
    }
}
