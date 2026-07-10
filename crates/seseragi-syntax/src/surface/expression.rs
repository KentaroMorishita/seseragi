use super::SurfaceParser;
use crate::surface_model::{ByteSpan, SurfaceExpr};
use crate::token::{Token, TokenKind};

mod do_block;
mod parenthesized;
#[cfg(test)]
mod tests;

impl SurfaceParser<'_> {
    pub(super) fn parse_expression(&self, start: usize, end: usize) -> Option<SurfaceExpr> {
        parse_expression_range(self.tokens, start, end)
    }
}

pub(super) fn parse_expression_range(
    tokens: &[Token],
    start: usize,
    end: usize,
) -> Option<SurfaceExpr> {
    let mut parser = ExpressionParser {
        tokens,
        cursor: start,
        end,
    };
    let expression = parser.parse_expr_bp(0, &[])?;
    parser.skip_trivia();
    if parser.cursor < end
        && parser
            .tokens
            .get(parser.cursor)
            .is_some_and(|token| token.kind != TokenKind::PunctuationSemicolon)
    {
        return None;
    }
    Some(expression)
}

struct ExpressionParser<'tokens> {
    tokens: &'tokens [Token],
    cursor: usize,
    end: usize,
}

impl ExpressionParser<'_> {
    fn parse_expr_bp(&mut self, min_bp: u8, stops: &[TokenKind]) -> Option<SurfaceExpr> {
        self.skip_trivia();
        if self.at_stop(stops) {
            return None;
        }
        let mut left = self.parse_prefix(stops)?;

        loop {
            self.skip_trivia();
            if self.cursor >= self.end || self.at_stop(stops) {
                break;
            }

            if self.can_start_application_argument() {
                const APPLICATION_BP: u8 = 80;
                if APPLICATION_BP < min_bp {
                    break;
                }
                let argument = self.parse_expr_bp(APPLICATION_BP + 1, stops)?;
                let span = ByteSpan {
                    start: left.span().start,
                    end: argument.span().end,
                };
                left = SurfaceExpr::Application {
                    function: Box::new(left),
                    argument: Box::new(argument),
                    span,
                };
                continue;
            }

            let operator_index = self.cursor;
            let operator = self.tokens.get(operator_index)?;
            let Some((left_bp, right_bp, operator_kind)) = binary_binding_power(operator) else {
                break;
            };
            if left_bp < min_bp {
                break;
            }
            self.cursor = operator_index + 1;
            let right = self.parse_expr_bp(right_bp, stops)?;
            let span = ByteSpan {
                start: left.span().start,
                end: right.span().end,
            };
            left = match operator_kind {
                ParsedOperator::Apply => SurfaceExpr::Application {
                    function: Box::new(left),
                    argument: Box::new(right),
                    span,
                },
                ParsedOperator::Binary => SurfaceExpr::Binary {
                    operator: operator.raw.clone(),
                    operator_span: token_span(operator),
                    left: Box::new(left),
                    right: Box::new(right),
                    span,
                },
            };
        }

        Some(left)
    }

    fn parse_prefix(&mut self, stops: &[TokenKind]) -> Option<SurfaceExpr> {
        self.skip_trivia();
        let index = self.cursor;
        let token = self.tokens.get(index)?;
        if index >= self.end || stops.contains(&token.kind) {
            return None;
        }
        self.cursor += 1;

        match token.kind {
            TokenKind::LiteralInteger => Some(SurfaceExpr::Integer {
                raw: token.raw.clone(),
                span: token_span(token),
            }),
            TokenKind::LiteralString => Some(SurfaceExpr::String {
                raw: token.raw.clone(),
                span: token_span(token),
            }),
            TokenKind::LiteralBoolean => Some(SurfaceExpr::Boolean {
                value: token.raw == "True",
                span: token_span(token),
            }),
            TokenKind::IdentifierLower | TokenKind::IdentifierUpper => Some(self.parse_name(index)),
            TokenKind::KeywordIf => self.parse_if(token, stops),
            TokenKind::KeywordDo => self.parse_do(token),
            TokenKind::PunctuationParenLeft => parenthesized::parse(self, token),
            TokenKind::Unknown => Some(SurfaceExpr::Error {
                span: token_span(token),
            }),
            _ => None,
        }
    }

    fn parse_name(&mut self, first: usize) -> SurfaceExpr {
        let mut name = self.tokens[first].raw.clone();
        let mut end = self.tokens[first].end;
        loop {
            let saved = self.cursor;
            self.skip_trivia();
            if self.kind_at_cursor() != Some(TokenKind::PunctuationDot) {
                self.cursor = saved;
                break;
            }
            self.cursor += 1;
            self.skip_trivia();
            let Some(next) = self.tokens.get(self.cursor) else {
                self.cursor = saved;
                break;
            };
            if !matches!(
                next.kind,
                TokenKind::IdentifierLower | TokenKind::IdentifierUpper
            ) {
                self.cursor = saved;
                break;
            }
            name.push('.');
            name.push_str(&next.raw);
            end = next.end;
            self.cursor += 1;
        }
        SurfaceExpr::Name {
            name,
            span: ByteSpan {
                start: self.tokens[first].start,
                end,
            },
        }
    }

    fn parse_if(&mut self, if_token: &Token, inherited_stops: &[TokenKind]) -> Option<SurfaceExpr> {
        let condition = self.parse_expr_bp(0, &[TokenKind::KeywordThen])?;
        self.consume(TokenKind::KeywordThen)?;
        let then_branch = self.parse_expr_bp(0, &[TokenKind::KeywordElse])?;
        self.consume(TokenKind::KeywordElse)?;
        let else_branch = self.parse_expr_bp(0, inherited_stops)?;
        Some(SurfaceExpr::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            span: ByteSpan {
                start: if_token.start,
                end: else_branch.span().end,
            },
            else_branch: Box::new(else_branch),
        })
    }

    fn parse_do(&mut self, do_token: &Token) -> Option<SurfaceExpr> {
        self.skip_trivia();
        let open = self.cursor;
        if self.kind_at_cursor() != Some(TokenKind::PunctuationBraceLeft) {
            return None;
        }
        let close = find_matching_brace(self.tokens, open, self.end)?;
        let (items, result) = do_block::parse_do_contents(self.tokens, open, close);
        self.cursor = close + 1;
        Some(SurfaceExpr::Do {
            items,
            result: result.map(Box::new),
            span: ByteSpan {
                start: do_token.start,
                end: self.tokens[close].end,
            },
        })
    }

    fn can_start_application_argument(&self) -> bool {
        matches!(
            self.kind_at_cursor(),
            Some(
                TokenKind::LiteralInteger
                    | TokenKind::LiteralString
                    | TokenKind::LiteralBoolean
                    | TokenKind::IdentifierLower
                    | TokenKind::IdentifierUpper
                    | TokenKind::KeywordIf
                    | TokenKind::KeywordDo
                    | TokenKind::PunctuationParenLeft
            )
        )
    }

    fn consume(&mut self, expected: TokenKind) -> Option<&Token> {
        self.skip_trivia();
        let token = self.tokens.get(self.cursor)?;
        if self.cursor >= self.end || token.kind != expected {
            return None;
        }
        self.cursor += 1;
        Some(token)
    }

    fn at_stop(&self, stops: &[TokenKind]) -> bool {
        self.kind_at_cursor()
            .is_some_and(|kind| stops.contains(&kind))
    }

    fn kind_at_cursor(&self) -> Option<TokenKind> {
        (self.cursor < self.end)
            .then(|| self.tokens.get(self.cursor).map(|token| token.kind))
            .flatten()
    }

    fn skip_trivia(&mut self) {
        while self.cursor < self.end
            && self.tokens.get(self.cursor).is_some_and(|token| {
                matches!(
                    token.kind,
                    TokenKind::TriviaComment | TokenKind::TriviaNewline | TokenKind::TriviaSpace
                )
            })
        {
            self.cursor += 1;
        }
    }
}

#[derive(Clone, Copy)]
enum ParsedOperator {
    Apply,
    Binary,
}

fn binary_binding_power(token: &Token) -> Option<(u8, u8, ParsedOperator)> {
    let (precedence, right_associative, kind) = match (token.kind, token.raw.as_str()) {
        (TokenKind::OperatorApply, "$") => (5, true, ParsedOperator::Apply),
        (TokenKind::OperatorPipeline, "|>") => (10, false, ParsedOperator::Binary),
        (TokenKind::OperatorComparison, _) => (30, false, ParsedOperator::Binary),
        (TokenKind::OperatorArithmetic, "+" | "-") => (40, false, ParsedOperator::Binary),
        (TokenKind::OperatorArithmetic, "*" | "/" | "%") => (50, false, ParsedOperator::Binary),
        (TokenKind::OperatorArithmetic, "**") => (60, true, ParsedOperator::Binary),
        _ => return None,
    };
    Some((
        precedence,
        if right_associative {
            precedence
        } else {
            precedence + 1
        },
        kind,
    ))
}

fn find_matching_brace(tokens: &[Token], open: usize, end: usize) -> Option<usize> {
    let mut depth = 0usize;
    for index in open..end {
        match tokens.get(index)?.kind {
            TokenKind::PunctuationBraceLeft => depth += 1,
            TokenKind::PunctuationBraceRight => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn token_span(token: &Token) -> ByteSpan {
    ByteSpan {
        start: token.start,
        end: token.end,
    }
}
