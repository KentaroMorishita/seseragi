use super::{ByteSpan, SurfaceParser, TypeRef};
use crate::surface_model::TypeRecordField;
use crate::token::TokenKind;

impl SurfaceParser<'_> {
    pub(super) fn parse_type_after_colon(&self, start: usize, end: usize) -> Option<TypeRef> {
        self.find_significant_token(start, end, |kind| kind == TokenKind::PunctuationColon)
            .and_then(|colon| self.parse_type_name(colon + 1, end))
    }

    pub(super) fn parse_type_name(&self, start: usize, end: usize) -> Option<TypeRef> {
        self.parse_type_ref(start, end)
            .map(|(type_ref, _)| type_ref)
    }

    pub(super) fn parse_type_ref(&self, start: usize, end: usize) -> Option<(TypeRef, usize)> {
        let (parameter, after_parameter) = self.parse_type_atom(start, end)?;
        let next = self.next_significant_token(after_parameter, end);
        if next.is_some_and(|index| self.kind_at(index) == Some(TokenKind::OperatorArrow)) {
            let arrow = next?;
            let (result, after_result) = self.parse_type_ref(arrow + 1, end)?;
            let span = ByteSpan {
                start: self.type_ref_span(&parameter)?.start,
                end: self.type_ref_span(&result)?.end,
            };
            return Some((
                TypeRef::Function {
                    parameter: Box::new(parameter),
                    result: Box::new(result),
                    span,
                },
                after_result,
            ));
        }
        Some((parameter, after_parameter))
    }

    pub(super) fn parse_type_atom(&self, start: usize, end: usize) -> Option<(TypeRef, usize)> {
        let type_index = self.next_significant_token(start, end)?;
        if self.kind_at(type_index) == Some(TokenKind::PunctuationBraceLeft) {
            return self.parse_record_type_ref(type_index, end);
        }
        if self.kind_at(type_index) == Some(TokenKind::PunctuationParenLeft) {
            return self.parse_parenthesized_type_ref(type_index, end);
        }
        if !matches!(
            self.kind_at(type_index),
            Some(TokenKind::IdentifierUpper | TokenKind::IdentifierLower)
        ) {
            return None;
        }

        let (name, name_span, after_name) = self.parse_qualified_type_name(type_index, end)?;
        let next = self.next_significant_token(after_name, end);
        let (arguments, next_index, span_end) =
            if next.is_some_and(|index| self.is_angle_left(index)) {
                self.parse_type_arguments(next? + 1, end)
                    .map(|(arguments, closing_angle)| {
                        let span_end = self
                            .tokens
                            .get(closing_angle)
                            .map(|token| token.end)
                            .unwrap_or(name_span.end);
                        (arguments, closing_angle + 1, span_end)
                    })
                    .unwrap_or_else(|| (Vec::new(), after_name, name_span.end))
            } else {
                (Vec::new(), after_name, name_span.end)
            };

        Some((
            TypeRef::Named {
                name,
                arguments,
                span: ByteSpan {
                    start: name_span.start,
                    end: span_end,
                },
            },
            next_index,
        ))
    }

    fn type_ref_span(&self, type_ref: &TypeRef) -> Option<ByteSpan> {
        match type_ref {
            TypeRef::Named { span, .. }
            | TypeRef::Record { span, .. }
            | TypeRef::Tuple { span, .. }
            | TypeRef::Function { span, .. } => Some(*span),
        }
    }

    fn parse_qualified_type_name(
        &self,
        start: usize,
        end: usize,
    ) -> Option<(String, ByteSpan, usize)> {
        let mut parts = vec![self.identifier_name_at(start)?];
        let mut last = start;
        let mut cursor = start + 1;

        while let Some(dot) = self.next_significant_token(cursor, end) {
            if self.kind_at(dot) != Some(TokenKind::PunctuationDot) {
                break;
            }
            let Some(next_name) = self.next_significant_token(dot + 1, end) else {
                break;
            };
            let Some(name) = self.identifier_name_at(next_name) else {
                break;
            };
            parts.push(name);
            last = next_name;
            cursor = next_name + 1;
        }

        let start_span = self.byte_span(start)?;
        let end_span = self.byte_span(last)?;
        Some((
            parts.join("."),
            ByteSpan {
                start: start_span.start,
                end: end_span.end,
            },
            cursor,
        ))
    }

    pub(super) fn parse_type_arguments(
        &self,
        start: usize,
        end: usize,
    ) -> Option<(Vec<TypeRef>, usize)> {
        let mut arguments = Vec::new();
        let mut cursor = start;

        loop {
            let next = self.next_significant_token(cursor, end)?;
            if self.is_angle_right(next) {
                return Some((arguments, next));
            }

            let (argument, after_argument) = self.parse_type_ref(next, end)?;
            arguments.push(argument);

            let separator = self.next_significant_token(after_argument, end)?;
            if self.is_angle_right(separator) {
                return Some((arguments, separator));
            }
            if self.kind_at(separator) != Some(TokenKind::PunctuationComma) {
                return None;
            }
            cursor = separator + 1;
        }
    }

    fn parse_record_type_ref(&self, open_brace: usize, end: usize) -> Option<(TypeRef, usize)> {
        let close_brace = self.find_matching_brace(open_brace, end)?;
        let fields = self.parse_record_type_fields(open_brace + 1, close_brace);
        let span = ByteSpan {
            start: self.tokens.get(open_brace)?.start,
            end: self.tokens.get(close_brace)?.end,
        };
        Some((
            TypeRef::Record {
                closed: true,
                fields,
                span,
            },
            close_brace + 1,
        ))
    }

    fn parse_parenthesized_type_ref(
        &self,
        open_paren: usize,
        end: usize,
    ) -> Option<(TypeRef, usize)> {
        let close_paren = self.find_matching_paren(open_paren, end)?;
        let mut elements = Vec::new();
        let mut saw_comma = false;
        let mut cursor = open_paren + 1;
        let mut needs_element = false;

        while let Some(next) = self.next_significant_token(cursor, close_paren) {
            needs_element = false;
            let (element, after_element) = self.parse_type_ref(next, close_paren)?;
            elements.push(element);
            let separator = self.next_significant_token(after_element, close_paren);
            if separator
                .is_some_and(|index| self.kind_at(index) == Some(TokenKind::PunctuationComma))
            {
                saw_comma = true;
                needs_element = true;
                let comma = separator?;
                cursor = comma + 1;
                continue;
            }
            if separator.is_some() {
                return None;
            }
            break;
        }

        if !saw_comma {
            let inner = elements.into_iter().next()?;
            return Some((inner, close_paren + 1));
        }
        if needs_element || elements.len() < 2 {
            return None;
        }

        let span = ByteSpan {
            start: self.tokens.get(open_paren)?.start,
            end: self.tokens.get(close_paren)?.end,
        };
        Some((TypeRef::Tuple { elements, span }, close_paren + 1))
    }

    fn parse_record_type_fields(&self, start: usize, end: usize) -> Vec<TypeRecordField> {
        let mut fields = Vec::new();
        let mut cursor = start;
        while let Some(name_index) = self.next_significant_token(cursor, end) {
            let Some(name) = self.identifier_name_at(name_index) else {
                cursor = name_index + 1;
                continue;
            };
            let Some(name_span) = self.byte_span(name_index) else {
                break;
            };
            let after_name = self.next_significant_token(name_index + 1, end);
            let optional =
                after_name.is_some_and(|index| matches!(self.raw_at(index), Some("?" | "?:")));
            let colon = if after_name.is_some_and(|index| self.raw_at(index) == Some("?:")) {
                after_name
            } else {
                let colon_start = if optional {
                    after_name.map(|index| index + 1).unwrap_or(name_index + 1)
                } else {
                    name_index + 1
                };
                self.find_significant_token(colon_start, end, |kind| {
                    kind == TokenKind::PunctuationColon
                })
            };
            let Some(colon) = colon else { break };
            let Some((type_ref, after_type)) = self.parse_type_ref(colon + 1, end) else {
                break;
            };
            fields.push(TypeRecordField {
                name,
                name_span,
                optional,
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

    fn find_matching_brace(&self, open_brace: usize, end: usize) -> Option<usize> {
        let mut depth = 0usize;
        for index in open_brace..end {
            match self.kind_at(index) {
                Some(TokenKind::PunctuationBraceLeft) => depth += 1,
                Some(TokenKind::PunctuationBraceRight) => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return Some(index);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn find_matching_paren(&self, open_paren: usize, end: usize) -> Option<usize> {
        let mut depth = 0usize;
        for index in open_paren..end {
            match self.kind_at(index) {
                Some(TokenKind::PunctuationParenLeft) => depth += 1,
                Some(TokenKind::PunctuationParenRight) => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return Some(index);
                    }
                }
                _ => {}
            }
        }
        None
    }
}
