use super::SurfaceParser;
use crate::surface_model::{SurfaceDecl, SurfaceMethod, Visibility};
use crate::token::TokenKind;

impl SurfaceParser<'_> {
    pub(super) fn parse_instance_methods(
        &self,
        body_start: usize,
        end: usize,
    ) -> Vec<SurfaceMethod> {
        self.method_ranges(body_start, end)
            .into_iter()
            .filter_map(|(start, method_end)| {
                let declaration =
                    self.parse_fn_decl(Visibility::Private, start, start, method_end)?;
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
                Some(SurfaceMethod {
                    name,
                    name_span,
                    type_parameters,
                    parameters,
                    return_type,
                    constraints,
                    body,
                    span,
                })
            })
            .collect()
    }

    pub(super) fn parse_trait_methods(&self, body_start: usize, end: usize) -> Vec<SurfaceMethod> {
        self.trait_method_ranges(body_start, end)
            .into_iter()
            .filter_map(|(start, method_end)| self.parse_trait_method(start, method_end))
            .collect()
    }

    pub(super) fn trait_method_ranges(&self, body_start: usize, end: usize) -> Vec<(usize, usize)> {
        self.method_ranges(body_start, end)
            .into_iter()
            .map(|(start, end)| {
                // A deprecation clause belongs to the following method. Only
                // recognize its complete grammar before `fn`, so contextual
                // parameter names such as `deprecated: A` stay in signatures.
                let prefix = (start..end).find(|index| {
                    self.raw_at(*index) == Some("deprecated")
                        && self.is_declaration_boundary(*index)
                        && self.deprecated_method_after(*index, end + 1) == Some(end)
                });
                let end = prefix.unwrap_or(end);
                let end = self
                    .previous_significant_token(end)
                    .filter(|last| self.kind_at(*last) == Some(TokenKind::PunctuationSemicolon))
                    .unwrap_or(end);
                (start, end)
            })
            .collect()
    }

    fn deprecated_method_after(&self, start: usize, end: usize) -> Option<usize> {
        let message = self.next_significant_token(start + 1, end)?;
        if self.kind_at(message) != Some(TokenKind::LiteralString) {
            return None;
        }
        let mut next = self.next_significant_token(message + 1, end)?;
        if self.raw_at(next) == Some("since") {
            let version = self.next_significant_token(next + 1, end)?;
            if self.kind_at(version) != Some(TokenKind::LiteralString) {
                return None;
            }
            next = self.next_significant_token(version + 1, end)?;
        }
        (self.kind_at(next) == Some(TokenKind::KeywordFn)).then_some(next)
    }

    pub(super) fn parse_trait_method(&self, start: usize, end: usize) -> Option<SurfaceMethod> {
        let name_index = self.next_significant_token(start + 1, end)?;
        let name = self.identifier_name_at(name_index)?;
        let (type_parameters, after_type_parameters) =
            self.parse_optional_type_parameters(name_index + 1, end);
        let where_index = self.find_raw(after_type_parameters, end, "where");
        let signature_end = where_index.unwrap_or(end);
        let constraints = where_index
            .map(|where_index| self.parse_constraints(where_index + 1, end))
            .unwrap_or_default();
        let (parameters, return_type) =
            self.parse_curried_signature(after_type_parameters, signature_end)?;
        Some(SurfaceMethod {
            name,
            name_span: self.byte_span(name_index)?,
            type_parameters,
            parameters,
            return_type,
            constraints,
            body: None,
            span: self.declaration_span(start, end)?,
        })
    }

    pub(super) fn method_ranges(&self, body_start: usize, end: usize) -> Vec<(usize, usize)> {
        if self.kind_at(body_start) != Some(TokenKind::PunctuationBraceLeft) {
            return Vec::new();
        }
        let mut starts = Vec::new();
        let mut depth = 1usize;
        let mut body_end = end;
        for index in body_start + 1..end {
            match self.kind_at(index) {
                Some(TokenKind::PunctuationBraceLeft) => depth += 1,
                Some(TokenKind::PunctuationBraceRight) => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        body_end = index;
                        break;
                    }
                }
                Some(TokenKind::KeywordFn) if depth == 1 => starts.push(index),
                _ => {}
            }
        }
        starts
            .iter()
            .enumerate()
            .map(|(position, start)| {
                (
                    *start,
                    starts.get(position + 1).copied().unwrap_or(body_end),
                )
            })
            .collect()
    }
}
