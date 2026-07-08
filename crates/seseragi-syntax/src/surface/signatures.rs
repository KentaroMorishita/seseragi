use super::{SurfaceParser, TypeRef};
use crate::surface_model::SurfaceParameter;
use crate::token::TokenKind;

impl SurfaceParser<'_> {
    pub(super) fn parse_curried_signature(
        &self,
        start: usize,
        end: usize,
    ) -> Option<(Vec<SurfaceParameter>, TypeRef)> {
        let mut parameters = Vec::new();
        let mut cursor = start;

        loop {
            let name_index = self.next_significant_token(cursor, end)?;
            let name = self.identifier_name_at(name_index)?;
            let colon = self.find_significant_token(name_index + 1, end, |kind| {
                kind == TokenKind::PunctuationColon
            })?;
            let (type_ref, after_type) = self.parse_type_ref(colon + 1, end)?;
            let arrow = self.next_significant_token(after_type, end)?;
            if self.kind_at(arrow) != Some(TokenKind::OperatorArrow) {
                return None;
            }
            parameters.push(SurfaceParameter {
                name,
                name_span: self.byte_span(name_index)?,
                type_ref,
            });
            cursor = arrow + 1;

            let next = self.next_significant_token(cursor, end)?;
            let next_after_type = self.parse_type_ref(next, end);
            let Some((return_type, after_return_type)) = next_after_type else {
                continue;
            };
            if self
                .next_significant_token(after_return_type, end)
                .is_none()
            {
                return Some((parameters, return_type));
            }
        }
    }
}
