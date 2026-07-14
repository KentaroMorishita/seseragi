use super::ExpressionParser;
use crate::surface_model::{ByteSpan, SurfaceExpr};
use crate::token::{Token, TokenKind};

pub(super) fn parse(parser: &mut ExpressionParser<'_>, open: &Token) -> Option<SurfaceExpr> {
    parser.skip_trivia();
    if parser.kind_at_cursor() == Some(TokenKind::PunctuationSquareRight) {
        let close = parser.consume(TokenKind::PunctuationSquareRight)?;
        return Some(SurfaceExpr::Array {
            elements: Vec::new(),
            span: ByteSpan {
                start: open.start,
                end: close.end,
            },
        });
    }

    let mut elements = Vec::new();
    loop {
        elements.push(parser.parse_expr_bp(
            0,
            &[
                TokenKind::PunctuationComma,
                TokenKind::PunctuationSquareRight,
            ],
        )?);
        parser.skip_trivia();
        if parser.kind_at_cursor() != Some(TokenKind::PunctuationComma) {
            break;
        }
        parser.consume(TokenKind::PunctuationComma)?;
        parser.skip_trivia();
        if parser.kind_at_cursor() == Some(TokenKind::PunctuationSquareRight) {
            break;
        }
    }
    let close = parser.consume(TokenKind::PunctuationSquareRight)?;
    Some(SurfaceExpr::Array {
        elements,
        span: ByteSpan {
            start: open.start,
            end: close.end,
        },
    })
}
