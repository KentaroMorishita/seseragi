use crate::typed::{analyze_effect_function, EffectFunctionIssue};
use seseragi_syntax::{
    ByteRange, ByteSpan, Diagnostic, DiagnosticSeverity, RelatedDiagnostic, SurfaceDecl, Token,
};

pub(super) fn collect_effect_fn_diagnostics(
    declaration: &SurfaceDecl,
    tokens: &[Token],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let SurfaceDecl::EffectFn { span, .. } = declaration else {
        return;
    };
    diagnostics.extend(
        analyze_effect_function(declaration, tokens)
            .into_iter()
            .map(|issue| diagnostic_from_issue(issue, *span)),
    );
}

fn diagnostic_from_issue(issue: EffectFunctionIssue, function: ByteSpan) -> Diagnostic {
    match issue {
        EffectFunctionIssue::CompactContractClause { primary } => Diagnostic {
            id: String::new(),
            code: "SES-P0001".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "effect.compact-contract-clause".to_owned(),
            primary: byte_range(primary),
            related: vec![related("compact inferred effect function", function)],
            fixes: Vec::new(),
        },
        EffectFunctionIssue::MissingDoResult { primary } => Diagnostic {
            id: String::new(),
            code: "SES-P0001".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "effect.do-missing-final-expression".to_owned(),
            primary: byte_range(primary),
            related: vec![related(
                "do block requires a final monadic expression",
                function,
            )],
            fixes: Vec::new(),
        },
        EffectFunctionIssue::CompactFailureConflict { primary, failures } => Diagnostic {
            id: String::new(),
            code: "SES-E0001".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "effect.compact-failure-conflict".to_owned(),
            primary: byte_range(primary),
            related: failures
                .into_iter()
                .map(|failure| {
                    related(
                        &format!("operation can fail with {}", failure.failure_type),
                        failure.origin,
                    )
                })
                .collect(),
            fixes: Vec::new(),
        },
        EffectFunctionIssue::DoStatementNotEffect { primary } => Diagnostic {
            id: String::new(),
            code: "SES-T0101".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "effect.do-statement-not-effect".to_owned(),
            primary: byte_range(primary),
            related: vec![related("explicit effect function", function)],
            fixes: Vec::new(),
        },
        EffectFunctionIssue::BindValueNotEffect { primary, bind } => Diagnostic {
            id: String::new(),
            code: "SES-T0101".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "effect.bind-value-not-effect".to_owned(),
            primary: byte_range(primary),
            related: vec![related("do bind statement", bind)],
            fixes: Vec::new(),
        },
        EffectFunctionIssue::CompactBodyNotEffect { primary } => Diagnostic {
            id: String::new(),
            code: "SES-T0101".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "effect.compact-body-not-effect".to_owned(),
            primary: byte_range(primary),
            related: vec![related("compact inferred effect function", function)],
            fixes: Vec::new(),
        },
    }
}

fn related(message: &str, span: ByteSpan) -> RelatedDiagnostic {
    RelatedDiagnostic {
        message: message.to_owned(),
        primary: byte_range(span),
    }
}

fn byte_range(span: ByteSpan) -> ByteRange {
    ByteRange {
        start: span.start,
        end: span.end,
    }
}
