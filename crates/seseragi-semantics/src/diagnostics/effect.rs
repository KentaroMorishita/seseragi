use crate::typed::{analyze_effect_function, EffectFunctionIssue, TypedResolution};
use seseragi_syntax::{
    ByteRange, ByteSpan, Diagnostic, DiagnosticSeverity, RelatedDiagnostic, SurfaceDecl, Token,
};

use super::{type_difference::type_difference, type_labels::type_label};

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
            type_difference: None,
            id: String::new(),
            code: "SES-P0001".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "effect.compact-contract-clause".to_owned(),
            primary: byte_range(primary),
            related: vec![related("compact inferred effect function", function)],
            fixes: Vec::new(),
        },
        EffectFunctionIssue::MissingDoResult { primary } => Diagnostic {
            type_difference: None,
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
            type_difference: None,
            id: String::new(),
            code: "SES-E0001".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "effect.compact-failure-conflict".to_owned(),
            primary: byte_range(primary),
            related: failures
                .into_iter()
                .map(|failure| {
                    related(
                        &format!("operation can fail with {}", type_label(&failure.failure)),
                        failure.origin,
                    )
                })
                .collect(),
            fixes: Vec::new(),
        },
        EffectFunctionIssue::ExplicitFailureMismatch {
            primary,
            declared,
            failures,
        } => {
            let actual = failures
                .first()
                .map(|failure| failure.failure.clone())
                .unwrap_or_else(|| declared.clone());
            let mut related_diagnostics = vec![related(
                &format!("declared failure type is {}", type_label(&declared)),
                primary,
            )];
            related_diagnostics.extend(failures.into_iter().map(|failure| {
                related(
                    &format!("operation can fail with {}", type_label(&failure.failure)),
                    failure.origin,
                )
            }));
            Diagnostic {
                type_difference: type_difference(&declared, &actual),
                id: String::new(),
                code: "SES-E0001".to_owned(),
                severity: DiagnosticSeverity::Error,
                message_key: "effect.explicit-failure-mismatch".to_owned(),
                primary: byte_range(primary),
                related: related_diagnostics,
                fixes: Vec::new(),
            }
        }
        EffectFunctionIssue::ExplicitSuccessMismatch {
            primary,
            declared,
            actual,
            origin,
        } => Diagnostic {
            type_difference: type_difference(&declared, &actual),
            id: String::new(),
            code: "SES-E0001".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "effect.explicit-success-mismatch".to_owned(),
            primary: byte_range(primary),
            related: vec![
                related(
                    &format!("declared success type is {}", type_label(&declared)),
                    primary,
                ),
                related(
                    &format!("body succeeds with {}", type_label(&actual)),
                    origin,
                ),
            ],
            fixes: Vec::new(),
        },
        EffectFunctionIssue::ExplicitEnvironmentMismatch {
            primary,
            declared,
            operations,
        } => {
            let actual = operations
                .first()
                .map(|operation| operation.environment.clone())
                .unwrap_or_else(|| declared.clone());
            let mut related_diagnostics = vec![related(
                &format!("declared environment is {}", type_label(&declared)),
                primary,
            )];
            related_diagnostics.extend(operations.into_iter().map(|operation| {
                related(
                    &format!(
                        "operation requires environment {}",
                        type_label(&operation.environment)
                    ),
                    operation.origin,
                )
            }));
            Diagnostic {
                type_difference: type_difference(&declared, &actual),
                id: String::new(),
                code: "SES-E0001".to_owned(),
                severity: DiagnosticSeverity::Error,
                message_key: "effect.explicit-environment-mismatch".to_owned(),
                primary: byte_range(primary),
                related: related_diagnostics,
                fixes: Vec::new(),
            }
        }
        EffectFunctionIssue::DoStatementNotEffect { primary } => Diagnostic {
            type_difference: None,
            id: String::new(),
            code: "SES-T0101".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "effect.do-statement-not-effect".to_owned(),
            primary: byte_range(primary),
            related: vec![related("explicit effect function", function)],
            fixes: Vec::new(),
        },
        EffectFunctionIssue::BindValueNotEffect { primary, bind } => Diagnostic {
            type_difference: None,
            id: String::new(),
            code: "SES-T0101".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "effect.bind-value-not-effect".to_owned(),
            primary: byte_range(primary),
            related: vec![related("do bind statement", bind)],
            fixes: Vec::new(),
        },
        EffectFunctionIssue::CompactBodyNotEffect { primary } => Diagnostic {
            type_difference: None,
            id: String::new(),
            code: "SES-T0101".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "effect.compact-body-not-effect".to_owned(),
            primary: byte_range(primary),
            related: vec![related("compact inferred effect function", function)],
            fixes: Vec::new(),
        },
        EffectFunctionIssue::MapErrorMapperNotFunction { primary, actual } => Diagnostic {
            type_difference: None,
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
            type_difference: None,
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
            type_difference: None,
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
            type_difference: None,
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
            type_difference: None,
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
        EffectFunctionIssue::Record(issue) => super::record::record_diagnostic(&issue, function),
        EffectFunctionIssue::Range(issue) => super::range::range_diagnostic(&issue, function),
        EffectFunctionIssue::Pattern(issue) => super::match_expression::diagnostic(&issue),
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
