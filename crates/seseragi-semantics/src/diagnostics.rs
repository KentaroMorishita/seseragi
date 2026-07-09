use seseragi_syntax::{
    lex, parse_diagnostics, parse_surface_ast, ByteRange, Diagnostic, DiagnosticArtifact,
    DiagnosticSeverity, RelatedDiagnostic, SurfaceDecl, Token, TokenKind,
};

pub fn semantic_diagnostics(source_name: impl Into<String>, source: &str) -> DiagnosticArtifact {
    let source_name = source_name.into();
    let mut artifact = parse_diagnostics(source_name.clone(), source);
    if !artifact.diagnostics.is_empty() {
        return artifact;
    }

    let surface = parse_surface_ast(artifact.source.clone(), source);
    let tokens = lex(artifact.source.clone(), source).tokens;
    let mut diagnostics = Vec::new();

    for declaration in &surface.declarations {
        collect_decl_diagnostics(declaration, &tokens, &mut diagnostics);
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

    if !inferred_contract {
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

    if operation.raw == "println" {
        return;
    }
    if operation.kind == TokenKind::KeywordDo {
        match compact_do_statement_operation(tokens, *span, operation) {
            None => return,
            Some(statement) if statement.raw == "println" => return,
            Some(statement) => {
                push_compact_body_not_effect_diagnostic(diagnostics, statement, *span);
                return;
            }
        }
    }

    push_compact_body_not_effect_diagnostic(diagnostics, operation, *span);
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

fn compact_do_statement_operation<'tokens>(
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
        .find(|token| token.end <= right_brace.start && is_significant(token))
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
}
