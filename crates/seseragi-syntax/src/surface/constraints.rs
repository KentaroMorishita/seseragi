use super::{SurfaceConstraint, SurfaceParser, TypeRef};
use crate::token::TokenKind;

impl SurfaceParser<'_> {
    pub(super) fn parse_constraints(&self, start: usize, end: usize) -> Vec<SurfaceConstraint> {
        let mut constraints = Vec::new();
        let mut cursor = start;
        while let Some(next) = self.next_significant_token(cursor, end) {
            let Some((constraint, after_constraint)) = self.parse_type_ref(next, end) else {
                break;
            };
            if let TypeRef::Named {
                name,
                arguments,
                span,
            } = constraint
            {
                constraints.push(SurfaceConstraint {
                    name,
                    arguments,
                    span,
                });
            }
            let Some(separator) = self.next_significant_token(after_constraint, end) else {
                break;
            };
            if self.kind_at(separator) != Some(TokenKind::PunctuationComma) {
                break;
            }
            cursor = separator + 1;
        }
        constraints
    }
}
