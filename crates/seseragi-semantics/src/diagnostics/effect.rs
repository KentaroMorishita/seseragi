use crate::typed::{analyze_effect_function, EffectFunctionIssue, TypedResolution};
use seseragi_syntax::{
    ByteRange, ByteSpan, Diagnostic, DiagnosticSeverity, RelatedDiagnostic, SurfaceDecl, Token,
};

use super::type_labels::type_label;

pub(super) fn collect_effect_fn_diagnostics(
    declaration: &SurfaceDecl,
    tokens: &[Token],
    resolution: &TypedResolution<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let SurfaceDecl::EffectFn { span, .. } = declaration else {
        return;
    };
    diagnostics.extend(
        analyze_effect_function(declaration, tokens, resolution)
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
        EffectFunctionIssue::MapErrorMapperNotFunction { primary, actual } => Diagnostic {
            id: String::new(),
            code: "SES-T0101".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "effect.map-error-mapper-not-function".to_owned(),
            primary: byte_range(primary),
            related: vec![related(
                &format!(
                    "expected a failure mapper, received {}",
                    type_label(&actual)
                ),
                function,
            )],
            fixes: Vec::new(),
        },
        EffectFunctionIssue::MapErrorSourceNotEffect { primary } => Diagnostic {
            id: String::new(),
            code: "SES-T0101".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "effect.map-error-source-not-effect".to_owned(),
            primary: byte_range(primary),
            related: vec![related("mapError requires an Effect value", function)],
            fixes: Vec::new(),
        },
        EffectFunctionIssue::MapErrorFailureMismatch {
            primary,
            expected,
            actual,
        } => Diagnostic {
            id: String::new(),
            code: "SES-E0001".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "effect.map-error-failure-mismatch".to_owned(),
            primary: byte_range(primary),
            related: vec![related(
                &format!(
                    "mapper accepts {}, but source fails with {}",
                    type_label(&actual),
                    type_label(&expected)
                ),
                function,
            )],
            fixes: Vec::new(),
        },
        EffectFunctionIssue::IntrinsicArityMismatch {
            primary,
            expected,
            actual,
        } => Diagnostic {
            id: String::new(),
            code: "SES-T0101".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "call.arity-mismatch".to_owned(),
            primary: byte_range(primary),
            related: vec![related(
                &format!(
                    "expected {} {}, received {actual}",
                    expected,
                    if expected == 1 {
                        "argument"
                    } else {
                        "arguments"
                    }
                ),
                function,
            )],
            fixes: Vec::new(),
        },
        EffectFunctionIssue::FromEitherSourceNotEither { primary, actual } => Diagnostic {
            id: String::new(),
            code: "SES-T0101".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "effect.from-either-source-not-either".to_owned(),
            primary: byte_range(primary),
            related: vec![related(
                &format!("expected Either<E, A>, received {}", type_label(&actual)),
                function,
            )],
            fixes: Vec::new(),
        },
        EffectFunctionIssue::Call(issue) => super::pure_call::call_diagnostic(issue, function),
        EffectFunctionIssue::Array(issue) => super::array::array_diagnostic(&issue, function),
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
