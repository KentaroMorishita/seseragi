use super::SurfaceParser;
use crate::surface_model::{SurfaceDecl, SurfaceInstanceMethod, Visibility};
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

    fn parse_instance_methods(&self, body_start: usize, end: usize) -> Vec<SurfaceInstanceMethod> {
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
                Some(TokenKind::KeywordFn) if depth == 1 => {
                    starts.push(index);
                }
                _ => {}
            }
        }
        starts
            .iter()
            .enumerate()
            .filter_map(|(position, start)| {
                let method_end = starts.get(position + 1).copied().unwrap_or(body_end);
                let declaration =
                    self.parse_fn_decl(Visibility::Private, *start, *start, method_end)?;
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
                Some(SurfaceInstanceMethod {
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
}
