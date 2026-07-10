use crate::typed::FunctionBodyIssue;
use seseragi_syntax::{ByteRange, Diagnostic, DiagnosticSeverity, RelatedDiagnostic};

use super::type_labels::type_label;

pub(super) fn collect_function_body_diagnostics(
    issue: Option<&FunctionBodyIssue>,
    span: seseragi_syntax::ByteSpan,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(FunctionBodyIssue {
        body,
        expected,
        actual,
    }) = issue
    else {
        return;
    };

    diagnostics.push(Diagnostic {
        id: String::new(),
        code: "SES-T0101".to_owned(),
        severity: DiagnosticSeverity::Error,
        message_key: "function.return-type-mismatch".to_owned(),
        primary: byte_range(*body),
        related: vec![RelatedDiagnostic {
            message: format!(
                "declared {}, body produces {}",
                type_label(expected),
                type_label(actual)
            ),
            primary: byte_range(span),
        }],
        fixes: Vec::new(),
    });
}

fn byte_range(span: seseragi_syntax::ByteSpan) -> ByteRange {
    ByteRange {
        start: span.start,
        end: span.end,
    }
}
