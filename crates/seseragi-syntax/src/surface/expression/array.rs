use super::ExpressionParser;
use crate::surface::expression::parse_expression_range;
use crate::surface::pattern::parse_pattern_range;
use crate::surface_model::{ByteSpan, SurfaceComprehensionClause, SurfaceExpr};
use crate::token::{Token, TokenKind};

pub(super) fn parse(parser: &mut ExpressionParser<'_>, open: &Token) -> Option<SurfaceExpr> {
    let is_list = open.kind == TokenKind::PunctuationListLeft;
    parser.skip_trivia();
    if parser.kind_at_cursor() == Some(TokenKind::PunctuationSquareRight) {
        let close = parser.consume(TokenKind::PunctuationSquareRight)?;
        return Some(collection_literal(
            is_list,
            Vec::new(),
            open.start,
            close.end,
        ));
    }

    let first = parser.parse_expr_bp(
        0,
        &[
            TokenKind::PunctuationComma,
            TokenKind::PunctuationSquareRight,
        ],
    )?;
    parser.skip_trivia();
    if parser
        .tokens
        .get(parser.cursor)
        .is_some_and(|token| token.kind == TokenKind::OperatorCustom && token.raw == "|")
    {
        parser.cursor += 1;
        return parse_comprehension(parser, open, first);
    }

    let mut elements = vec![first];
    loop {
        parser.skip_trivia();
        if parser.kind_at_cursor() != Some(TokenKind::PunctuationComma) {
            break;
        }
        parser.consume(TokenKind::PunctuationComma)?;
        parser.skip_trivia();
        if parser.kind_at_cursor() == Some(TokenKind::PunctuationSquareRight) {
            break;
        }
        elements.push(parser.parse_expr_bp(
            0,
            &[
                TokenKind::PunctuationComma,
                TokenKind::PunctuationSquareRight,
            ],
        )?);
    }
    let close = parser.consume(TokenKind::PunctuationSquareRight)?;
    Some(collection_literal(is_list, elements, open.start, close.end))
}

fn collection_literal(
    is_list: bool,
    elements: Vec<SurfaceExpr>,
    start: usize,
    end: usize,
) -> SurfaceExpr {
    let span = ByteSpan { start, end };
    if is_list {
        SurfaceExpr::List { elements, span }
    } else {
        SurfaceExpr::Array { elements, span }
    }
}

fn parse_comprehension(
    parser: &mut ExpressionParser<'_>,
    open: &Token,
    element: SurfaceExpr,
) -> Option<SurfaceExpr> {
    let mut clauses = Vec::new();
    loop {
        parser.skip_trivia();
        let start = parser.cursor;
        let end = clause_end(parser.tokens, start, parser.end)?;
        if start == end {
            return None;
        }
        let clause = if let Some(bind) = top_level_bind(parser.tokens, start, end) {
            let pattern = parse_pattern_range(parser.tokens, start, bind);
            let source = parse_expression_range(parser.tokens, bind + 1, end)?;
            SurfaceComprehensionClause::Generator {
                span: ByteSpan {
                    start: pattern.span().start,
                    end: source.span().end,
                },
                pattern,
                source,
            }
        } else {
            let condition = parse_expression_range(parser.tokens, start, end)?;
            SurfaceComprehensionClause::Guard {
                span: condition.span(),
                condition,
            }
        };
        clauses.push(clause);
        parser.cursor = end;
        parser.skip_trivia();
        if parser.kind_at_cursor() != Some(TokenKind::PunctuationComma) {
            break;
        }
        parser.cursor += 1;
    }
    let close = parser.consume(TokenKind::PunctuationSquareRight)?;
    Some(SurfaceExpr::ArrayComprehension {
        element: Box::new(element),
        clauses,
        span: ByteSpan {
            start: open.start,
            end: close.end,
        },
    })
}

fn clause_end(tokens: &[Token], start: usize, end: usize) -> Option<usize> {
    let mut braces = 0usize;
    let mut parens = 0usize;
    let mut squares = 0usize;
    for (index, token) in tokens.iter().enumerate().take(end).skip(start) {
        if braces == 0 && parens == 0 && squares == 0 {
            if token.kind == TokenKind::PunctuationComma
                || token.kind == TokenKind::PunctuationSquareRight
            {
                return Some(index);
            }
        }
        match token.kind {
            TokenKind::PunctuationBraceLeft => braces += 1,
            TokenKind::PunctuationBraceRight => braces = braces.saturating_sub(1),
            TokenKind::PunctuationParenLeft => parens += 1,
            TokenKind::PunctuationParenRight => parens = parens.saturating_sub(1),
            TokenKind::PunctuationListLeft | TokenKind::PunctuationSquareLeft => squares += 1,
            TokenKind::PunctuationSquareRight => squares = squares.saturating_sub(1),
            _ => {}
        }
    }
    None
}

fn top_level_bind(tokens: &[Token], start: usize, end: usize) -> Option<usize> {
    let mut braces = 0usize;
    let mut parens = 0usize;
    let mut squares = 0usize;
    for (index, token) in tokens.iter().enumerate().take(end).skip(start) {
        if token.kind == TokenKind::OperatorBind && braces == 0 && parens == 0 && squares == 0 {
            return Some(index);
        }
        match token.kind {
            TokenKind::PunctuationBraceLeft => braces += 1,
            TokenKind::PunctuationBraceRight => braces = braces.saturating_sub(1),
            TokenKind::PunctuationParenLeft => parens += 1,
            TokenKind::PunctuationParenRight => parens = parens.saturating_sub(1),
            TokenKind::PunctuationListLeft | TokenKind::PunctuationSquareLeft => squares += 1,
            TokenKind::PunctuationSquareRight => squares = squares.saturating_sub(1),
            _ => {}
        }
    }
    None
}
