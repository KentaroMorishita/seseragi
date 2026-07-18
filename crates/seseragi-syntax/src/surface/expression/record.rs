use super::ExpressionParser;
use crate::surface_model::{ByteSpan, SurfaceExpr, SurfaceRecordItem, TypeRef};
use crate::token::{Token, TokenKind};

pub(super) fn parse(parser: &mut ExpressionParser<'_>, open: &Token) -> Option<SurfaceExpr> {
    let (items, end) = parse_items(parser, false)?;
    Some(SurfaceExpr::Record {
        items,
        span: ByteSpan {
            start: open.start,
            end,
        },
    })
}

pub(super) fn parse_struct(
    parser: &mut ExpressionParser<'_>,
    name: &Token,
    _open: &Token,
    type_arguments: Option<Vec<TypeRef>>,
) -> Option<SurfaceExpr> {
    let (items, end) = parse_items(parser, true)?;
    Some(SurfaceExpr::Struct {
        name: name.raw.clone(),
        name_span: token_span(name),
        type_arguments,
        items,
        span: ByteSpan {
            start: name.start,
            end,
        },
    })
}

fn parse_items(
    parser: &mut ExpressionParser<'_>,
    allow_single_shorthand: bool,
) -> Option<(Vec<SurfaceRecordItem>, usize)> {
    parser.skip_trivia();
    if parser.kind_at_cursor() == Some(TokenKind::PunctuationBraceRight) {
        let close = parser.consume(TokenKind::PunctuationBraceRight)?;
        return Some((Vec::new(), close.end));
    }

    let mut items = Vec::new();
    loop {
        parser.skip_trivia();
        let mut ambiguous_shorthand = false;
        if parser.kind_at_cursor() == Some(TokenKind::PunctuationEllipsis) {
            let spread = parser.tokens.get(parser.cursor)?.clone();
            parser.cursor += 1;
            let value = parser.parse_expr_bp(
                0,
                &[
                    TokenKind::PunctuationComma,
                    TokenKind::PunctuationBraceRight,
                ],
            )?;
            items.push(SurfaceRecordItem::Spread {
                span: ByteSpan {
                    start: spread.start,
                    end: value.span().end,
                },
                value,
            });
        } else {
            let name_token = parser.tokens.get(parser.cursor)?.clone();
            if name_token.kind != TokenKind::IdentifierLower {
                return None;
            }
            parser.cursor += 1;
            parser.skip_trivia();

            let (value, explicit) = if parser.kind_at_cursor() == Some(TokenKind::PunctuationColon)
            {
                parser.cursor += 1;
                let value = parser.parse_expr_bp(
                    0,
                    &[
                        TokenKind::PunctuationComma,
                        TokenKind::PunctuationBraceRight,
                    ],
                )?;
                (value, true)
            } else {
                (
                    SurfaceExpr::Name {
                        name: name_token.raw.clone(),
                        span: token_span(&name_token),
                    },
                    false,
                )
            };
            ambiguous_shorthand = !explicit;
            let field_end = value.span().end;
            let name_span = token_span(&name_token);
            items.push(SurfaceRecordItem::Field {
                name: name_token.raw,
                name_span,
                value,
                span: ByteSpan {
                    start: name_token.start,
                    end: field_end,
                },
            });
        }

        parser.skip_trivia();
        if parser.kind_at_cursor() == Some(TokenKind::PunctuationBraceRight) {
            if ambiguous_shorthand && items.len() == 1 && !allow_single_shorthand {
                return None;
            }
            break;
        }
        parser.consume(TokenKind::PunctuationComma)?;
        parser.skip_trivia();
        if parser.kind_at_cursor() == Some(TokenKind::PunctuationBraceRight) {
            break;
        }
    }

    let close = parser.consume(TokenKind::PunctuationBraceRight)?;
    Some((items, close.end))
}

fn token_span(token: &Token) -> ByteSpan {
    ByteSpan {
        start: token.start,
        end: token.end,
    }
}
