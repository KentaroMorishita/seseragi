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
                let prefix = crate::declaration::declaration_prefix_in(
                    self.tokens,
                    start,
                    method_end,
                    crate::declaration::DeclarationContext::Member,
                )
                .ok()?;
                // An instance implements a contract; it cannot introduce an
                // independent API migration notice for that contract method.
                if prefix.deprecation.is_some() || prefix.public {
                    return None;
                }
                let declaration =
                    self.parse_fn_decl(Visibility::Private, start, prefix.keyword, method_end)?;
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
                let end = self
                    .previous_significant_token(end)
                    .filter(|last| self.kind_at(*last) == Some(TokenKind::PunctuationSemicolon))
                    .unwrap_or(end);
                (start, end)
            })
            .collect()
    }

    pub(super) fn parse_trait_method(&self, start: usize, end: usize) -> Option<SurfaceMethod> {
        let prefix = crate::declaration::declaration_prefix_in(
            self.tokens,
            start,
            end,
            crate::declaration::DeclarationContext::Member,
        )
        .ok()?;
        if prefix.public || self.kind_at(prefix.keyword) != Some(TokenKind::KeywordFn) {
            return None;
        }
        let name_index = self.next_significant_token(prefix.keyword + 1, end)?;
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
        let body_end = self.find_matching_brace(body_start, end).unwrap_or(end);
        let starts = self.declaration_member_starts(
            body_start + 1,
            body_end,
            crate::declaration::DeclarationContext::Member,
        );
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

    /// Delimit members with the same closed modifier grammar as declarations.
    /// A clause starts the following member instead of extending its predecessor.
    pub(super) fn declaration_member_starts(
        &self,
        start: usize,
        end: usize,
        context: crate::declaration::DeclarationContext,
    ) -> Vec<usize> {
        use crate::declaration::{declaration_prefix_in, DeclarationContext};
        let mut starts = Vec::new();
        let mut depth = 0usize;
        let mut prefix_end = None;
        for index in start..end {
            match self.kind_at(index) {
                Some(TokenKind::PunctuationBraceLeft) => depth += 1,
                Some(TokenKind::PunctuationBraceRight) => {
                    depth = depth.saturating_sub(1);
                }
                _ if depth == 0 && !prefix_end.is_some_and(|end| index <= end) => {
                    let raw = self.raw_at(index).unwrap_or_default();
                    let contextual_prefix = raw == "deprecated"
                        && self
                            .next_significant_token(index + 1, end)
                            .is_none_or(|next| {
                                self.kind_at(next) != Some(TokenKind::PunctuationColon)
                            });
                    let candidate = contextual_prefix
                        || match context {
                            DeclarationContext::Foreign => {
                                matches!(raw, "pure" | "task" | "opaque" | "namespace")
                            }
                            _ => matches!(raw, "pub" | "fn" | "operator"),
                        };
                    let boundary = self.is_declaration_boundary(index)
                        || self
                            .previous_significant_token(index)
                            .is_some_and(|previous| {
                                self.kind_at(previous) == Some(TokenKind::PunctuationBraceLeft)
                            });
                    if candidate && boundary {
                        starts.push(index);
                        prefix_end = declaration_prefix_in(self.tokens, index, end, context)
                            .ok()
                            .map(|prefix| prefix.keyword);
                    }
                }
                _ => {}
            }
        }
        starts
    }
}
