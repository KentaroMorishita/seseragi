use crate::typed::{PureCallIssue, PureFunctionAnalysis};
use seseragi_syntax::{ByteRange, Diagnostic, DiagnosticSeverity, RelatedDiagnostic};

use super::type_labels::type_label;

pub(super) fn collect_pure_function_diagnostics(
    analysis: &PureFunctionAnalysis,
    span: seseragi_syntax::ByteSpan,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some(issue) = &analysis.pure_call_issue {
        diagnostics.push(pure_call_diagnostic(issue.clone(), span));
    }
}

fn pure_call_diagnostic(
    issue: PureCallIssue,
    function_span: seseragi_syntax::ByteSpan,
) -> Diagnostic {
    let (message_key, primary, related_message) = match issue {
        PureCallIssue::Arity {
            callee,
            expected,
            actual,
        } => (
            "call.arity-mismatch",
            callee,
            format!(
                "expected {} {}, received {actual}",
                expected,
                argument_word(expected)
            ),
        ),
        PureCallIssue::ArgumentType {
            argument,
            index,
            expected,
            actual,
        } => (
            "call.argument-type-mismatch",
            argument,
            format!(
                "argument {} expected {}, received {}",
                index + 1,
                type_label(&expected),
                type_label(&actual)
            ),
        ),
    };
    Diagnostic {
        id: String::new(),
        code: "SES-T0101".to_owned(),
        severity: DiagnosticSeverity::Error,
        message_key: message_key.to_owned(),
        primary: ByteRange {
            start: primary.start,
            end: primary.end,
        },
        related: vec![RelatedDiagnostic {
            message: related_message,
            primary: ByteRange {
                start: function_span.start,
                end: function_span.end,
            },
        }],
        fixes: Vec::new(),
    }
}

fn argument_word(count: usize) -> &'static str {
    if count == 1 {
        "argument"
    } else {
        "arguments"
    }
}
