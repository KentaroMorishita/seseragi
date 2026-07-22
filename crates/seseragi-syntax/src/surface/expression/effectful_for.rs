use super::{find_matching_brace, parse_expression_range, ExpressionParser};
use crate::surface::pattern::parse_pattern_range;
use crate::surface_model::{ByteSpan, SurfaceExpr};
use crate::token::{Token, TokenKind};

pub(super) fn parse(parser: &mut ExpressionParser<'_>, for_token: &Token) -> Option<SurfaceExpr> {
    parser.skip_trivia();
    let pattern_start = parser.cursor;
    let bind = top_level_token(
        parser.tokens,
        pattern_start,
        parser.end,
        TokenKind::OperatorBind,
    )?;
    let pattern = parse_pattern_range(parser.tokens, pattern_start, bind);
    let body_open = top_level_token(
        parser.tokens,
        bind + 1,
        parser.end,
        TokenKind::PunctuationBraceLeft,
    )?;
    let source = parse_expression_range(parser.tokens, bind + 1, body_open)?;
    let body_close = find_matching_brace(parser.tokens, body_open, parser.end)?;
    let body = parse_expression_range(parser.tokens, body_open + 1, body_close)?;
    parser.cursor = body_close + 1;

    let end = parser.tokens[body_close].end;
    Some(SurfaceExpr::EffectfulFor {
        pattern,
        source: Box::new(source),
        body: Box::new(body),
        span: ByteSpan {
            start: for_token.start,
            end,
        },
    })
}

fn top_level_token(
    tokens: &[Token],
    start: usize,
    end: usize,
    expected: TokenKind,
) -> Option<usize> {
    let mut parens = 0usize;
    let mut squares = 0usize;
    let mut braces = 0usize;
    for (index, token) in tokens.iter().enumerate().take(end).skip(start) {
        if token.kind == expected && parens == 0 && squares == 0 && braces == 0 {
            return Some(index);
        }
        match token.kind {
            TokenKind::PunctuationParenLeft => parens += 1,
            TokenKind::PunctuationParenRight => parens = parens.saturating_sub(1),
            TokenKind::PunctuationListLeft | TokenKind::PunctuationSquareLeft => squares += 1,
            TokenKind::PunctuationSquareRight => squares = squares.saturating_sub(1),
            TokenKind::PunctuationBraceLeft => braces += 1,
            TokenKind::PunctuationBraceRight => braces = braces.saturating_sub(1),
            _ => {}
        }
    }
    None
}
