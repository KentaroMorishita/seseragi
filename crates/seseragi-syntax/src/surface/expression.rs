use super::SurfaceParser;
use crate::surface_model::{ByteSpan, SurfaceExpr, SurfaceInfixStep, SurfaceLambdaParameter};
use crate::token::{Token, TokenKind};

mod array;
mod do_block;
mod match_expression;
mod parenthesized;
mod record;
mod template;
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
        if min_bp == 0 {
            let saved = self.cursor;
            if let Some(chain) = self.parse_unresolved_infix_chain(stops) {
                return Some(chain);
            }
            self.cursor = saved;
        }

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

            if self.kind_at_cursor() == Some(TokenKind::PunctuationDot) {
                const MEMBER_BP: u8 = 90;
                if MEMBER_BP < min_bp {
                    break;
                }
                let saved = self.cursor;
                self.cursor += 1;
                self.skip_trivia();
                let Some(field) = self.tokens.get(self.cursor).cloned() else {
                    self.cursor = saved;
                    break;
                };
                if field.kind != TokenKind::IdentifierLower
                    && field.kind != TokenKind::IdentifierUpper
                {
                    self.cursor = saved;
                    break;
                }
                self.cursor += 1;
                let span = ByteSpan {
                    start: left.span().start,
                    end: field.end,
                };
                let field_span = token_span(&field);
                left = SurfaceExpr::Member {
                    receiver: Box::new(left),
                    field: field.raw,
                    field_span,
                    span,
                };
                continue;
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
                ParsedOperator::Pipeline => SurfaceExpr::Application {
                    function: Box::new(right),
                    argument: Box::new(left),
                    span,
                },
                ParsedOperator::TraitMethod {
                    method,
                    method_operand_sources,
                } => trait_method_application(
                    method,
                    token_span(operator),
                    left,
                    right,
                    method_operand_sources,
                    span,
                ),
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

    /// Retains a complete infix sequence whenever it contains an operator
    /// whose fixity must be supplied by module semantics. Known operators are
    /// kept in the same flat sequence so a later stage can reassociate the
    /// whole expression rather than inheriting an arbitrary parser choice.
    /// Expressions containing only language-defined operators continue down
    /// the existing Pratt path unchanged.
    fn parse_unresolved_infix_chain(&mut self, stops: &[TokenKind]) -> Option<SurfaceExpr> {
        const INFIX_OPERAND_BP: u8 = 70;

        self.skip_trivia();
        if self.at_stop(stops) {
            return None;
        }
        let first = self.parse_expr_bp(INFIX_OPERAND_BP, stops)?;
        let mut steps = Vec::new();
        let mut contains_unresolved = false;

        loop {
            self.skip_trivia();
            if self.cursor >= self.end || self.at_stop(stops) {
                break;
            }
            let Some(operator) = self.infix_operator_occurrence() else {
                break;
            };
            if !is_infix_operator(&operator.token) {
                break;
            }
            contains_unresolved |= is_unresolved_infix_operator(&operator.token);
            self.cursor = operator.next;
            let operand = self.parse_expr_bp(INFIX_OPERAND_BP, stops)?;
            let operator_span = token_span(&operator.token);
            steps.push(SurfaceInfixStep {
                operator: operator.token.raw,
                operator_span,
                operand,
            });
        }

        if !contains_unresolved {
            return None;
        }
        let end = steps.last()?.operand.span().end;
        Some(SurfaceExpr::InfixChain {
            span: ByteSpan {
                start: first.span().start,
                end,
            },
            first: Box::new(first),
            steps,
        })
    }

    fn infix_operator_occurrence(&self) -> Option<InfixOperatorOccurrence> {
        let first = self.tokens.get(self.cursor)?;
        if first.kind != TokenKind::PunctuationColon
            && !super::operators::is_operator_spelling_token(first.kind)
        {
            return None;
        }

        let mut next = self.cursor + 1;
        let mut end = first.end;
        let mut spelling = first.raw.clone();
        while next < self.end {
            let token = self.tokens.get(next)?;
            if token.start != end || !super::operators::is_operator_spelling_token(token.kind) {
                break;
            }
            spelling.push_str(&token.raw);
            end = token.end;
            next += 1;
        }

        Some(InfixOperatorOccurrence {
            token: Token {
                kind: if next == self.cursor + 1 {
                    first.kind
                } else {
                    TokenKind::OperatorCustom
                },
                start: first.start,
                end,
                raw: spelling,
            },
            next,
        })
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
            TokenKind::LiteralTemplate => template::parse(token),
            TokenKind::LiteralBoolean => Some(SurfaceExpr::Boolean {
                value: token.raw == "True",
                span: token_span(token),
            }),
            TokenKind::IdentifierUpper => {
                if let Some((type_arguments, open)) = self.consume_struct_head() {
                    record::parse_struct(self, token, &open, type_arguments)
                } else {
                    Some(SurfaceExpr::Name {
                        name: token.raw.clone(),
                        span: token_span(token),
                    })
                }
            }
            TokenKind::IdentifierLower => Some(SurfaceExpr::Name {
                name: token.raw.clone(),
                span: token_span(token),
            }),
            TokenKind::OperatorLambda => self.parse_lambda(token, stops),
            TokenKind::KeywordIf => self.parse_if(token, stops),
            TokenKind::KeywordMatch => match_expression::parse(self, token),
            TokenKind::KeywordDo => self.parse_do(token),
            TokenKind::PunctuationParenLeft => parenthesized::parse(self, token),
            TokenKind::PunctuationBraceLeft => record::parse(self, token),
            TokenKind::PunctuationListLeft | TokenKind::PunctuationSquareLeft => {
                array::parse(self, token)
            }
            TokenKind::Unknown => Some(SurfaceExpr::Error {
                span: token_span(token),
            }),
            _ => None,
        }
    }

    fn consume_struct_head(&mut self) -> Option<(Option<Vec<crate::TypeRef>>, Token)> {
        self.skip_trivia();
        if self.kind_at_cursor() == Some(TokenKind::PunctuationBraceLeft) {
            let open = self.tokens.get(self.cursor)?.clone();
            self.cursor += 1;
            return Some((None, open));
        }

        let open_angle = self.cursor;
        if self
            .tokens
            .get(open_angle)
            .is_none_or(|token| token.kind != TokenKind::OperatorComparison || token.raw != "<")
        {
            return None;
        }
        let parser = SurfaceParser {
            tokens: self.tokens,
            non_eof_token_count: self.end,
        };
        let (type_arguments, closing_angle) =
            parser.parse_type_arguments(open_angle + 1, self.end)?;
        let open_brace = parser.next_significant_token(closing_angle + 1, self.end)?;
        if parser.kind_at(open_brace) != Some(TokenKind::PunctuationBraceLeft) {
            return None;
        }

        let open = self.tokens.get(open_brace)?.clone();
        self.cursor = open_brace + 1;
        Some((Some(type_arguments), open))
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
                    | TokenKind::LiteralTemplate
                    | TokenKind::LiteralBoolean
                    | TokenKind::IdentifierLower
                    | TokenKind::IdentifierUpper
                    | TokenKind::OperatorLambda
                    | TokenKind::KeywordIf
                    | TokenKind::KeywordMatch
                    | TokenKind::KeywordDo
                    | TokenKind::PunctuationParenLeft
                    | TokenKind::PunctuationBraceLeft
                    | TokenKind::PunctuationListLeft
                    | TokenKind::PunctuationSquareLeft
            )
        )
    }

    fn parse_lambda(
        &mut self,
        lambda_token: &Token,
        inherited_stops: &[TokenKind],
    ) -> Option<SurfaceExpr> {
        let mut parameters = Vec::new();
        loop {
            self.skip_trivia();
            if self.kind_at_cursor() == Some(TokenKind::OperatorArrow) {
                break;
            }
            let parameter = self.tokens.get(self.cursor)?.clone();
            if parameter.kind != TokenKind::IdentifierLower {
                return None;
            }
            self.cursor += 1;
            self.skip_trivia();
            let type_ref = if self.kind_at_cursor() == Some(TokenKind::PunctuationColon) {
                self.cursor += 1;
                let parser = SurfaceParser {
                    tokens: self.tokens,
                    non_eof_token_count: self.end,
                };
                let (type_ref, after_type) = parser.parse_type_atom(self.cursor, self.end)?;
                self.cursor = after_type;
                Some(type_ref)
            } else {
                None
            };
            let name_span = token_span(&parameter);
            parameters.push(SurfaceLambdaParameter {
                name: parameter.raw,
                name_span,
                type_ref,
            });
        }
        if parameters.is_empty() {
            return None;
        }
        self.consume(TokenKind::OperatorArrow)?;
        let body = self.parse_expr_bp(0, inherited_stops)?;
        let body_end = body.span().end;
        let parameter_count = parameters.len();
        Some(
            parameters
                .into_iter()
                .rev()
                .enumerate()
                .fold(body, |body, (index, parameter)| SurfaceExpr::Lambda {
                    span: ByteSpan {
                        start: if index + 1 == parameter_count {
                            lambda_token.start
                        } else {
                            parameter.name_span.start
                        },
                        end: body_end,
                    },
                    parameter,
                    body: Box::new(body),
                }),
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

struct InfixOperatorOccurrence {
    token: Token,
    next: usize,
}

#[derive(Clone, Copy)]
enum ParsedOperator {
    Apply,
    Pipeline,
    TraitMethod {
        method: &'static str,
        method_operand_sources: [usize; 2],
    },
    Binary,
}

fn binary_binding_power(token: &Token) -> Option<(u8, u8, ParsedOperator)> {
    let (precedence, right_associative, kind) = match (token.kind, token.raw.as_str()) {
        (TokenKind::OperatorApply, "$") => (5, true, ParsedOperator::Apply),
        (TokenKind::OperatorPipeline, "|>") => (10, false, ParsedOperator::Pipeline),
        (TokenKind::OperatorCustom, spelling)
            if crate::standard_trait_operator(spelling).is_some() =>
        {
            let operator = crate::standard_trait_operator(spelling)
                .expect("matched standard trait operator must remain registered");
            (
                operator.parser_precedence,
                operator.associativity == crate::OperatorAssociativity::Right,
                ParsedOperator::TraitMethod {
                    method: operator.method_name,
                    method_operand_sources: operator.method_operand_sources,
                },
            )
        }
        (TokenKind::OperatorComparison, _) => (30, false, ParsedOperator::Binary),
        (TokenKind::OperatorRangeExclusive | TokenKind::OperatorRangeInclusive, _) => {
            (35, false, ParsedOperator::Binary)
        }
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

fn is_infix_operator(token: &Token) -> bool {
    binary_binding_power(token).is_some() || is_unresolved_infix_operator(token)
}

fn is_unresolved_infix_operator(token: &Token) -> bool {
    token.kind == TokenKind::OperatorCustom
        && token.raw != "|"
        && binary_binding_power(token).is_none()
}

fn trait_method_application(
    method: &str,
    operator_span: ByteSpan,
    left: SurfaceExpr,
    right: SurfaceExpr,
    method_operand_sources: [usize; 2],
    span: ByteSpan,
) -> SurfaceExpr {
    let mut source_operands = [Some(left), Some(right)];
    let [first_source, second_source] = method_operand_sources;
    let first = source_operands[first_source]
        .take()
        .expect("trait operator method order must be a permutation");
    let second = source_operands[second_source]
        .take()
        .expect("trait operator method order must be a permutation");
    let first_span = ByteSpan {
        start: operator_span.start.min(first.span().start),
        end: operator_span.end.max(first.span().end),
    };
    SurfaceExpr::Application {
        function: Box::new(SurfaceExpr::Application {
            function: Box::new(SurfaceExpr::Name {
                name: method.to_owned(),
                span: operator_span,
            }),
            argument: Box::new(first),
            span: first_span,
        }),
        argument: Box::new(second),
        span,
    }
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
