use crate::effect_ops::known_effect_operation_by_surface;
use seseragi_syntax::{
    ByteRange, ByteSpan, Diagnostic, DiagnosticSeverity, RelatedDiagnostic, SurfaceDecl, Token,
    TokenKind,
};

pub(super) fn collect_effect_fn_diagnostics(
    declaration: &SurfaceDecl,
    tokens: &[Token],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let SurfaceDecl::EffectFn {
        inferred_contract,
        span,
        ..
    } = declaration
    else {
        return;
    };

    let diagnostic_count = diagnostics.len();
    collect_invalid_do_bind_diagnostics(tokens, *span, diagnostics);
    if diagnostics.len() != diagnostic_count {
        return;
    }
    if collect_missing_do_result_diagnostic(tokens, *span, diagnostics) {
        return;
    }

    if !inferred_contract {
        collect_invalid_explicit_do_statement_diagnostics(tokens, *span, diagnostics);
        return;
    }

    if let Some(clause) = compact_contract_clause(tokens, *span) {
        diagnostics.push(Diagnostic {
            id: String::new(),
            code: "SES-T0002".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "effect.compact-contract-clause".to_owned(),
            primary: ByteRange {
                start: clause.start,
                end: clause.end,
            },
            related: vec![RelatedDiagnostic {
                message: "compact inferred effect function".to_owned(),
                primary: byte_range(*span),
            }],
            fixes: Vec::new(),
        });
        return;
    }

    let Some(operation) = compact_effect_body_operation(tokens, *span) else {
        return;
    };

    if is_known_effect_surface_operation(operation) {
        collect_compact_failure_conflict(tokens, *span, diagnostics);
        return;
    }
    if operation.kind == TokenKind::KeywordDo {
        match compact_do_unknown_statement_operation(tokens, *span, operation) {
            None => {
                collect_compact_failure_conflict(tokens, *span, diagnostics);
                return;
            }
            Some(statement) => {
                push_compact_body_not_effect_diagnostic(diagnostics, statement, *span);
                return;
            }
        }
    }

    push_compact_body_not_effect_diagnostic(diagnostics, operation, *span);
}

fn collect_missing_do_result_diagnostic(
    tokens: &[Token],
    span: ByteSpan,
    diagnostics: &mut Vec<Diagnostic>,
) -> bool {
    let Some(do_index) = tokens.iter().position(|token| {
        token.start >= span.start && token.end <= span.end && token.kind == TokenKind::KeywordDo
    }) else {
        return false;
    };
    let Some(left_brace_index) = tokens[do_index + 1..]
        .iter()
        .position(|token| token.end <= span.end && token.raw == "{")
        .map(|index| do_index + 1 + index)
    else {
        return false;
    };
    let Some(right_brace_index) = tokens[left_brace_index + 1..]
        .iter()
        .position(|token| token.end <= span.end && token.raw == "}")
        .map(|index| left_brace_index + 1 + index)
    else {
        return false;
    };
    let contents = &tokens[left_brace_index + 1..right_brace_index];
    let significant = contents
        .iter()
        .filter(|token| is_significant(token))
        .collect::<Vec<_>>();
    let final_operation_index = contents.iter().rposition(is_known_effect_surface_operation);
    let final_operation_is_bound = final_operation_index.is_some_and(|operation_index| {
        let line_start = contents[..operation_index]
            .iter()
            .rposition(|token| token.kind == TokenKind::TriviaNewline)
            .map_or(0, |index| index + 1);
        contents[line_start..operation_index]
            .iter()
            .any(|token| token.kind == TokenKind::OperatorBind)
    });
    let missing_final_expression = significant.is_empty() || final_operation_is_bound;
    if !missing_final_expression {
        return false;
    }
    let origin = ByteSpan {
        start: tokens[right_brace_index].start,
        end: tokens[right_brace_index].start,
    };
    diagnostics.push(Diagnostic {
        id: String::new(),
        code: "SES-P0001".to_owned(),
        severity: DiagnosticSeverity::Error,
        message_key: "effect.do-missing-final-expression".to_owned(),
        primary: byte_range(origin),
        related: vec![RelatedDiagnostic {
            message: "do block requires a final monadic expression".to_owned(),
            primary: byte_range(span),
        }],
        fixes: Vec::new(),
    });
    true
}

fn collect_compact_failure_conflict(
    tokens: &[Token],
    span: ByteSpan,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(equals_index) = tokens
        .iter()
        .position(|token| token.start >= span.start && token.end <= span.end && token.raw == "=")
    else {
        return;
    };
    let mut failures = Vec::new();
    for token in tokens[equals_index + 1..]
        .iter()
        .take_while(|token| token.end <= span.end)
    {
        let Some(operation) = known_effect_operation_by_surface(token.raw.as_str()) else {
            continue;
        };
        if operation.failure_type == "Never"
            || failures
                .iter()
                .any(|(failure_type, _)| *failure_type == operation.failure_type)
        {
            continue;
        }
        failures.push((operation.failure_type, token));
    }
    let Some((_, conflicting_operation)) = failures.get(1) else {
        return;
    };
    diagnostics.push(Diagnostic {
        id: String::new(),
        code: "SES-T0003".to_owned(),
        severity: DiagnosticSeverity::Error,
        message_key: "effect.compact-failure-conflict".to_owned(),
        primary: ByteRange {
            start: conflicting_operation.start,
            end: conflicting_operation.end,
        },
        related: failures
            .into_iter()
            .map(|(failure_type, operation)| RelatedDiagnostic {
                message: format!("operation can fail with {failure_type}"),
                primary: ByteRange {
                    start: operation.start,
                    end: operation.end,
                },
            })
            .collect(),
        fixes: Vec::new(),
    });
}

fn collect_invalid_explicit_do_statement_diagnostics(
    tokens: &[Token],
    span: ByteSpan,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(do_index) = tokens.iter().position(|token| {
        token.start >= span.start && token.end <= span.end && token.kind == TokenKind::KeywordDo
    }) else {
        return;
    };
    let Some(left_brace_index) = tokens[do_index + 1..]
        .iter()
        .position(|token| token.end <= span.end && token.raw == "{")
        .map(|index| do_index + 1 + index)
    else {
        return;
    };
    let Some(right_brace_index) = tokens[left_brace_index + 1..]
        .iter()
        .position(|token| token.end <= span.end && token.raw == "}")
        .map(|index| left_brace_index + 1 + index)
    else {
        return;
    };

    let mut line_start = left_brace_index + 1;
    for line_end in (left_brace_index + 1..=right_brace_index).filter(|index| {
        *index == right_brace_index || tokens[*index].kind == TokenKind::TriviaNewline
    }) {
        collect_invalid_explicit_do_line(tokens, span, line_start, line_end, diagnostics);
        line_start = line_end + 1;
    }
}

fn collect_invalid_explicit_do_line(
    tokens: &[Token],
    span: ByteSpan,
    start: usize,
    end: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let significant = tokens[start..end]
        .iter()
        .filter(|token| is_significant(token))
        .collect::<Vec<_>>();
    let Some(first) = significant.first().copied() else {
        return;
    };
    if significant
        .iter()
        .any(|token| token.kind == TokenKind::OperatorBind)
        || is_known_effect_surface_operation(first)
    {
        return;
    }
    if first.kind != TokenKind::IdentifierLower {
        return;
    }
    diagnostics.push(Diagnostic {
        id: String::new(),
        code: "SES-T0103".to_owned(),
        severity: DiagnosticSeverity::Error,
        message_key: "effect.do-statement-not-effect".to_owned(),
        primary: ByteRange {
            start: first.start,
            end: first.end,
        },
        related: vec![RelatedDiagnostic {
            message: "explicit effect function".to_owned(),
            primary: byte_range(span),
        }],
        fixes: Vec::new(),
    });
}

fn collect_invalid_do_bind_diagnostics(
    tokens: &[Token],
    span: ByteSpan,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (index, bind) in tokens.iter().enumerate().filter(|(_, token)| {
        token.start >= span.start && token.end <= span.end && token.kind == TokenKind::OperatorBind
    }) {
        let operation = tokens[index + 1..]
            .iter()
            .find(|token| token.end <= span.end && token.kind == TokenKind::IdentifierLower);
        let Some(operation) = operation else {
            continue;
        };
        if is_known_effect_surface_operation(operation) {
            continue;
        }
        diagnostics.push(Diagnostic {
            id: String::new(),
            code: "SES-T0102".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "effect.bind-value-not-effect".to_owned(),
            primary: ByteRange {
                start: operation.start,
                end: operation.end,
            },
            related: vec![RelatedDiagnostic {
                message: "do bind statement".to_owned(),
                primary: ByteRange {
                    start: bind.start,
                    end: operation.end,
                },
            }],
            fixes: Vec::new(),
        });
    }
}

fn push_compact_body_not_effect_diagnostic(
    diagnostics: &mut Vec<Diagnostic>,
    operation: &Token,
    span: ByteSpan,
) {
    diagnostics.push(Diagnostic {
        id: String::new(),
        code: "SES-T0001".to_owned(),
        severity: DiagnosticSeverity::Error,
        message_key: "effect.compact-body-not-effect".to_owned(),
        primary: ByteRange {
            start: operation.start,
            end: operation.end,
        },
        related: vec![RelatedDiagnostic {
            message: "compact inferred effect function".to_owned(),
            primary: byte_range(span),
        }],
        fixes: Vec::new(),
    });
}

fn compact_effect_body_operation(tokens: &[Token], span: ByteSpan) -> Option<&Token> {
    let equals_index = tokens
        .iter()
        .position(|token| token.start >= span.start && token.end <= span.end && token.raw == "=")?;
    tokens[equals_index + 1..]
        .iter()
        .find(|token| token.end <= span.end && is_significant(token))
}

fn compact_contract_clause(tokens: &[Token], span: ByteSpan) -> Option<&Token> {
    let equals_index = tokens
        .iter()
        .position(|token| token.start >= span.start && token.end <= span.end && token.raw == "=")?;
    tokens
        .iter()
        .take(equals_index)
        .find(|token| token.start >= span.start && is_compact_contract_clause_token(token))
}

fn is_compact_contract_clause_token(token: &Token) -> bool {
    matches!(token.kind, TokenKind::KeywordWith | TokenKind::KeywordFails) || token.raw == "where"
}

fn is_known_effect_surface_operation(token: &Token) -> bool {
    known_effect_operation_by_surface(token.raw.as_str()).is_some()
}

fn compact_do_unknown_statement_operation<'tokens>(
    tokens: &'tokens [Token],
    span: ByteSpan,
    do_token: &Token,
) -> Option<&'tokens Token> {
    let left_brace = tokens
        .iter()
        .skip_while(|token| token.start <= do_token.start)
        .find(|token| token.end <= span.end && token.raw == "{")?;
    let right_brace = tokens
        .iter()
        .skip_while(|token| token.start <= do_token.start)
        .find(|token| token.end <= span.end && token.raw == "}")?;
    tokens
        .iter()
        .skip_while(|token| token.start <= left_brace.start)
        .find(|token| {
            token.end <= right_brace.start
                && token.kind == TokenKind::IdentifierLower
                && !is_known_effect_surface_operation(token)
                && !is_do_bind_target(tokens, token)
        })
}

fn is_do_bind_target(tokens: &[Token], candidate: &Token) -> bool {
    tokens
        .iter()
        .skip_while(|token| token.start <= candidate.start)
        .find(|token| is_significant(token))
        .is_some_and(|token| token.kind == TokenKind::OperatorBind)
}

fn is_significant(token: &Token) -> bool {
    !matches!(
        token.kind,
        TokenKind::TriviaComment
            | TokenKind::TriviaNewline
            | TokenKind::TriviaSpace
            | TokenKind::Eof
    )
}

fn byte_range(span: ByteSpan) -> ByteRange {
    ByteRange {
        start: span.start,
        end: span.end,
    }
}
