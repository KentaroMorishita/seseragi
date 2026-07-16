use super::ExpressionParser;
use crate::surface_model::{ByteSpan, SurfaceExpr, SurfaceRecordField};
use crate::token::{Token, TokenKind};

pub(super) fn parse(parser: &mut ExpressionParser<'_>, open: &Token) -> Option<SurfaceExpr> {
    parser.skip_trivia();
    if parser.kind_at_cursor() == Some(TokenKind::PunctuationBraceRight) {
        let close = parser.consume(TokenKind::PunctuationBraceRight)?;
        return Some(SurfaceExpr::Record {
            fields: Vec::new(),
            span: ByteSpan {
                start: open.start,
                end: close.end,
            },
        });
    }

    let mut fields = Vec::new();
    loop {
        parser.skip_trivia();
        let name_token = parser.tokens.get(parser.cursor)?.clone();
        if name_token.kind != TokenKind::IdentifierLower {
            return None;
        }
        parser.cursor += 1;
        parser.skip_trivia();

        let (value, explicit) = if parser.kind_at_cursor() == Some(TokenKind::PunctuationColon) {
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
        let field_end = value.span().end;
        let name_span = token_span(&name_token);
        fields.push(SurfaceRecordField {
            name: name_token.raw,
            name_span,
            value,
            span: ByteSpan {
                start: name_token.start,
                end: field_end,
            },
        });

        parser.skip_trivia();
        if parser.kind_at_cursor() == Some(TokenKind::PunctuationBraceRight) {
            if !explicit && fields.len() == 1 {
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
    Some(SurfaceExpr::Record {
        fields,
        span: ByteSpan {
            start: open.start,
            end: close.end,
        },
    })
}

fn token_span(token: &Token) -> ByteSpan {
    ByteSpan {
        start: token.start,
        end: token.end,
    }
}
