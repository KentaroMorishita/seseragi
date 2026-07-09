use crate::effect_ops::known_effect_operation_by_surface;
use seseragi_syntax::{
    lex, parse_diagnostics, parse_surface_ast, ByteRange, Diagnostic, DiagnosticArtifact,
    DiagnosticSeverity, RelatedDiagnostic, SurfaceDecl, Token, TokenKind,
};
use std::collections::BTreeSet;

pub fn semantic_diagnostics(source_name: impl Into<String>, source: &str) -> DiagnosticArtifact {
    let source_name = source_name.into();
    let mut artifact = parse_diagnostics(source_name.clone(), source);
    if !artifact.diagnostics.is_empty() {
        return artifact;
    }

    let surface = parse_surface_ast(artifact.source.clone(), source);
    let tokens = lex(artifact.source.clone(), source).tokens;
    let declared_values = declared_value_names(&surface.declarations);
    let mut diagnostics = Vec::new();

    for declaration in &surface.declarations {
        collect_decl_diagnostics(declaration, &tokens, &declared_values, &mut diagnostics);
    }

    artifact.diagnostics = diagnostics
        .into_iter()
        .enumerate()
        .map(|(index, mut diagnostic)| {
            diagnostic.id = format!("d{}", index + 1);
            diagnostic
        })
        .collect();
    artifact
}

fn collect_decl_diagnostics(
    declaration: &SurfaceDecl,
    tokens: &[Token],
    declared_values: &BTreeSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let SurfaceDecl::Fn {
        parameters, span, ..
    } = declaration
    {
        collect_unknown_pure_function_names(
            tokens,
            *span,
            parameters,
            declared_values,
            diagnostics,
        );
        return;
    }

    let SurfaceDecl::EffectFn {
        inferred_contract,
        span,
        ..
    } = declaration
    else {
        return;
    };

    collect_invalid_do_bind_diagnostics(tokens, *span, diagnostics);

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
                primary: ByteRange {
                    start: span.start,
                    end: span.end,
                },
            }],
            fixes: Vec::new(),
        });
        return;
    }

    let Some(operation) = compact_effect_body_operation(tokens, *span) else {
        return;
    };

    if is_known_effect_surface_operation(operation) {
        return;
    }
    if operation.kind == TokenKind::KeywordDo {
        match compact_do_unknown_statement_operation(tokens, *span, operation) {
            None => return,
            Some(statement) => {
                push_compact_body_not_effect_diagnostic(diagnostics, statement, *span);
                return;
            }
        }
    }

    push_compact_body_not_effect_diagnostic(diagnostics, operation, *span);
}

fn collect_invalid_explicit_do_statement_diagnostics(
    tokens: &[Token],
    span: seseragi_syntax::ByteSpan,
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
    span: seseragi_syntax::ByteSpan,
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
            primary: ByteRange {
                start: span.start,
                end: span.end,
            },
        }],
        fixes: Vec::new(),
    });
}

fn collect_invalid_do_bind_diagnostics(
    tokens: &[Token],
    span: seseragi_syntax::ByteSpan,
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

fn declared_value_names(declarations: &[SurfaceDecl]) -> BTreeSet<String> {
    declarations
        .iter()
        .filter_map(|declaration| match declaration {
            SurfaceDecl::Let { name, .. } => Some(name.clone()),
            SurfaceDecl::Fn { .. }
            | SurfaceDecl::EffectFn { .. }
            | SurfaceDecl::Newtype { .. }
            | SurfaceDecl::Alias { .. }
            | SurfaceDecl::Type { .. }
            | SurfaceDecl::Struct { .. }
            | SurfaceDecl::Trait { .. }
            | SurfaceDecl::Operator { .. }
            | SurfaceDecl::Instance { .. } => None,
        })
        .collect()
}

fn collect_unknown_pure_function_names(
    tokens: &[Token],
    span: seseragi_syntax::ByteSpan,
    parameters: &[seseragi_syntax::SurfaceParameter],
    declared_values: &BTreeSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(equals_index) = tokens
        .iter()
        .position(|token| token.start >= span.start && token.end <= span.end && token.raw == "=")
    else {
        return;
    };
    let parameter_names = parameters
        .iter()
        .map(|parameter| parameter.name.as_str())
        .collect::<BTreeSet<_>>();
    for token in tokens[equals_index + 1..]
        .iter()
        .take_while(|token| token.end <= span.end)
        .filter(|token| token.kind == TokenKind::IdentifierLower)
    {
        if parameter_names.contains(token.raw.as_str()) || declared_values.contains(&token.raw) {
            continue;
        }
        diagnostics.push(Diagnostic {
            id: String::new(),
            code: "SES-T0101".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "name.unresolved".to_owned(),
            primary: ByteRange {
                start: token.start,
                end: token.end,
            },
            related: vec![RelatedDiagnostic {
                message: "pure function body".to_owned(),
                primary: ByteRange {
                    start: span.start,
                    end: span.end,
                },
            }],
            fixes: Vec::new(),
        });
    }
}

fn push_compact_body_not_effect_diagnostic(
    diagnostics: &mut Vec<Diagnostic>,
    operation: &Token,
    span: seseragi_syntax::ByteSpan,
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
            primary: ByteRange {
                start: span.start,
                end: span.end,
            },
        }],
        fixes: Vec::new(),
    });
}

fn compact_effect_body_operation(
    tokens: &[Token],
    span: seseragi_syntax::ByteSpan,
) -> Option<&Token> {
    let equals_index = tokens
        .iter()
        .position(|token| token.start >= span.start && token.end <= span.end && token.raw == "=")?;
    tokens[equals_index + 1..]
        .iter()
        .find(|token| token.end <= span.end && is_significant(token))
}

fn compact_contract_clause(tokens: &[Token], span: seseragi_syntax::ByteSpan) -> Option<&Token> {
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
    span: seseragi_syntax::ByteSpan,
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
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_compact_effect_body_that_is_not_effect() {
        let diagnostics = semantic_diagnostics(
            "artifact/effect-compact-not-effect/main.ssrg",
            "pub effect fn greet name: String = name\n",
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].id, "d1");
        assert_eq!(diagnostics.diagnostics[0].code, "SES-T0001");
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "effect.compact-body-not-effect"
        );
        assert_eq!(
            diagnostics.diagnostics[0].primary,
            ByteRange { start: 35, end: 39 }
        );
        assert_eq!(diagnostics.diagnostics[0].related.len(), 1);
        assert_eq!(
            diagnostics.diagnostics[0].related[0].primary,
            ByteRange { start: 0, end: 39 }
        );
    }

    #[test]
    fn reports_unknown_compact_do_statement() {
        let diagnostics = semantic_diagnostics(
            "artifact/effect-compact-do-not-effect/main.ssrg",
            "pub effect fn greet =\n  do { name }\n",
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-T0001");
        assert_eq!(
            diagnostics.diagnostics[0].primary,
            ByteRange { start: 29, end: 33 }
        );
        assert_eq!(
            diagnostics.diagnostics[0].related[0].primary,
            ByteRange { start: 0, end: 35 }
        );
    }

    #[test]
    fn reports_unknown_compact_do_statement_after_known_statement() {
        let diagnostics = semantic_diagnostics(
            "artifact/effect-compact-do-late-not-effect/main.ssrg",
            "pub effect fn greet =\n  do { println \"hello\" name }\n",
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-T0001");
        assert_eq!(
            diagnostics.diagnostics[0].primary,
            ByteRange { start: 45, end: 49 }
        );
    }

    #[test]
    fn reports_contract_clause_in_compact_effect_fn() {
        let diagnostics = semantic_diagnostics(
            "artifact/effect-compact-with-clause/main.ssrg",
            "pub effect fn main with Console =\n  println \"hello\"\n",
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-T0002");
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "effect.compact-contract-clause"
        );
        assert_eq!(
            diagnostics.diagnostics[0].primary,
            ByteRange { start: 19, end: 23 }
        );
        assert_eq!(
            diagnostics.diagnostics[0].related[0].primary,
            ByteRange { start: 0, end: 51 }
        );
    }

    #[test]
    fn reports_unresolved_name_in_pure_function_body() {
        let diagnostics = semantic_diagnostics(
            "artifact/unknown-pure-name/main.ssrg",
            "pub fn useMissing value: Int -> Int = missing\n",
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-T0101");
        assert_eq!(diagnostics.diagnostics[0].message_key, "name.unresolved");
        assert_eq!(
            diagnostics.diagnostics[0].primary,
            ByteRange { start: 38, end: 45 }
        );
    }

    #[test]
    fn accepts_top_level_binding_in_pure_function_body() {
        let diagnostics = semantic_diagnostics(
            "artifact/top-level-binding/main.ssrg",
            "pub let answer: Int = 42\npub fn answerValue unit: Unit -> Int = answer\n",
        );

        assert!(diagnostics.diagnostics.is_empty());
    }

    #[test]
    fn rejects_function_value_reference_until_function_values_are_typed() {
        let diagnostics = semantic_diagnostics(
            "artifact/function-value-reference/main.ssrg",
            "fn source unit: Unit -> Int = 1\nfn alias unit: Unit -> Int = source\n",
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-T0101");
        assert_eq!(
            diagnostics.diagnostics[0].primary,
            ByteRange { start: 61, end: 67 }
        );
    }

    #[test]
    fn reports_non_effect_value_in_explicit_do_bind() {
        let diagnostics = semantic_diagnostics(
            "artifact/invalid-do-bind/main.ssrg",
            "effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  do { ignored <- missing }\n",
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-T0102");
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "effect.bind-value-not-effect"
        );
    }

    #[test]
    fn reports_non_effect_statement_in_explicit_do_block() {
        let diagnostics = semantic_diagnostics(
            "artifact/invalid-explicit-do/main.ssrg",
            "effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  do { missing }\n",
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-T0103");
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "effect.do-statement-not-effect"
        );
    }
}
