use super::SurfaceParser;
use crate::surface_model::{SurfaceDecl, SurfaceField, SurfaceVariant, Visibility};
use crate::token::TokenKind;

impl SurfaceParser<'_> {
    pub(super) fn parse_newtype_decl(
        &self,
        visibility: Visibility,
        opaque: bool,
        top_start: usize,
        decl_start: usize,
        end: usize,
    ) -> Option<SurfaceDecl> {
        let name_index = self.next_significant_token(decl_start + 1, end)?;
        let name = self.identifier_name_at(name_index)?;
        let (type_parameters, after_type_parameters) =
            self.parse_optional_type_parameters(name_index + 1, end);
        let (deriving, after_deriving) = self.parse_optional_deriving(after_type_parameters, end);
        let equals = self.find_significant_token(after_deriving, end, |kind| {
            kind == TokenKind::OperatorEquals
        })?;
        let representation = self.parse_type_name(equals + 1, end)?;

        Some(SurfaceDecl::Newtype {
            visibility,
            opaque,
            name,
            name_span: self.byte_span(name_index)?,
            type_parameters,
            deriving,
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

    pub(super) fn parse_type_decl(
        &self,
        visibility: Visibility,
        opaque: bool,
        top_start: usize,
        decl_start: usize,
        end: usize,
    ) -> Option<SurfaceDecl> {
        let name_index = self.next_significant_token(decl_start + 1, end)?;
        let name = self.identifier_name_at(name_index)?;
        let (type_parameters, after_type_parameters) =
            self.parse_optional_type_parameters(name_index + 1, end);
        let (deriving, after_deriving) = self.parse_optional_deriving(after_type_parameters, end);
        let equals = self.find_significant_token(after_deriving, end, |kind| {
            kind == TokenKind::OperatorEquals
        })?;

        Some(SurfaceDecl::Type {
            visibility,
            opaque,
            name,
            name_span: self.byte_span(name_index)?,
            type_parameters,
            deriving,
            variants: self.parse_type_variants(equals + 1, end),
            span: self.declaration_span(top_start, end)?,
        })
    }

    fn parse_type_variants(&self, start: usize, end: usize) -> Vec<SurfaceVariant> {
        let mut variants = Vec::new();
        let mut cursor = start;
        while let Some(pipe) = self.find_raw(cursor, end, "|") {
            let Some(name_index) = self.next_significant_token(pipe + 1, end) else {
                break;
            };
            let Some(name) = self.identifier_name_at(name_index) else {
                cursor = name_index + 1;
                continue;
            };
            let Some(name_span) = self.byte_span(name_index) else {
                break;
            };
            let after_name = name_index + 1;
            let next = self.next_significant_token(after_name, end);
            let payload = next
                .filter(|index| self.raw_at(*index) != Some("|"))
                .and_then(|index| self.parse_type_ref(index, end))
                .map(|(type_ref, _)| type_ref);
            variants.push(SurfaceVariant {
                name,
                name_span,
                payload,
            });
            cursor = after_name;
        }
        variants
    }

    pub(super) fn parse_struct_decl(
        &self,
        visibility: Visibility,
        opaque: bool,
        top_start: usize,
        decl_start: usize,
        end: usize,
    ) -> Option<SurfaceDecl> {
        let name_index = self.next_significant_token(decl_start + 1, end)?;
        let name = self.identifier_name_at(name_index)?;
        let (type_parameters, after_type_parameters) =
            self.parse_optional_type_parameters(name_index + 1, end);
        let (deriving, after_deriving) = self.parse_optional_deriving(after_type_parameters, end);
        let body_start = self.find_significant_token(after_deriving, end, |kind| {
            kind == TokenKind::PunctuationBraceLeft
        })?;
        let body_end = self
            .find_significant_token(body_start + 1, end, |kind| {
                kind == TokenKind::PunctuationBraceRight
            })
            .unwrap_or(end);

        Some(SurfaceDecl::Struct {
            visibility,
            opaque,
            name,
            name_span: self.byte_span(name_index)?,
            type_parameters,
            deriving,
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

    fn parse_optional_deriving(&self, start: usize, end: usize) -> (Vec<String>, usize) {
        let Some(deriving_keyword) = self.next_significant_token(start, end) else {
            return (Vec::new(), start);
        };
        if self.raw_at(deriving_keyword) != Some("deriving") {
            return (Vec::new(), start);
        }

        let mut deriving = Vec::new();
        let mut cursor = deriving_keyword + 1;
        loop {
            let Some(name_index) = self.next_significant_token(cursor, end) else {
                return (deriving, cursor);
            };
            let Some(name) = self.identifier_name_at(name_index) else {
                return (deriving, cursor);
            };
            deriving.push(name);

            let Some(separator) = self.next_significant_token(name_index + 1, end) else {
                return (deriving, name_index + 1);
            };
            if self.kind_at(separator) != Some(TokenKind::PunctuationComma) {
                return (deriving, separator);
            }
            cursor = separator + 1;
        }
    }
}
