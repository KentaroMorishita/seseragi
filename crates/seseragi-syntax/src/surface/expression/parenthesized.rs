use super::ExpressionParser;
use crate::surface_model::{ByteSpan, SurfaceExpr};
use crate::token::{Token, TokenKind};

pub(super) fn parse(parser: &mut ExpressionParser<'_>, open_token: &Token) -> Option<SurfaceExpr> {
    parser.skip_trivia();
    if parser.kind_at_cursor() == Some(TokenKind::PunctuationParenRight) {
        let close = parser.tokens.get(parser.cursor)?;
        parser.cursor += 1;
        return Some(SurfaceExpr::Unit {
            span: ByteSpan {
                start: open_token.start,
                end: close.end,
            },
        });
    }

    if let Some(operator) = parser.infix_operator_occurrence().filter(|operator| {
        crate::standard_operator(&operator.token.raw).is_some()
            || super::is_unresolved_infix_operator(&operator.token)
    }) {
        parser.cursor = operator.next;
        parser.skip_trivia();
        if parser.kind_at_cursor() == Some(TokenKind::PunctuationParenRight) {
            let close = parser.tokens.get(parser.cursor)?;
            parser.cursor += 1;
            return Some(SurfaceExpr::Grouped {
                value: Box::new(SurfaceExpr::Name {
                    name: operator.token.raw,
                    span: ByteSpan {
                        start: operator.token.start,
                        end: operator.token.end,
                    },
                }),
                span: ByteSpan {
                    start: open_token.start,
                    end: close.end,
                },
            });
        }
        return None;
    }

    let first = parser.parse_expr_bp(
        0,
        &[
            TokenKind::PunctuationComma,
            TokenKind::PunctuationParenRight,
        ],
    )?;
    parser.skip_trivia();
    if parser.kind_at_cursor() != Some(TokenKind::PunctuationComma) {
        let close = parser.consume(TokenKind::PunctuationParenRight)?;
        return Some(SurfaceExpr::Grouped {
            value: Box::new(first),
            span: ByteSpan {
                start: open_token.start,
                end: close.end,
            },
        });
    }

    let mut elements = vec![first];
    loop {
        parser.consume(TokenKind::PunctuationComma)?;
        parser.skip_trivia();
        if parser.kind_at_cursor() == Some(TokenKind::PunctuationParenRight) {
            return None;
        }
        elements.push(parser.parse_expr_bp(
            0,
            &[
                TokenKind::PunctuationComma,
                TokenKind::PunctuationParenRight,
            ],
        )?);
        parser.skip_trivia();
        if parser.kind_at_cursor() != Some(TokenKind::PunctuationComma) {
            break;
        }
    }
    let close = parser.consume(TokenKind::PunctuationParenRight)?;
    Some(SurfaceExpr::Tuple {
        elements,
        span: ByteSpan {
            start: open_token.start,
            end: close.end,
        },
    })
}
