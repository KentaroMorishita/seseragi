use crate::typed::RangeIssue;
use seseragi_syntax::{ByteRange, ByteSpan, Diagnostic, DiagnosticSeverity, RelatedDiagnostic};

use super::type_labels::type_label;

pub(super) fn collect_range_diagnostic(
    issue: Option<&RangeIssue>,
    declaration: ByteSpan,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(issue) = issue else {
        return;
    };
    diagnostics.push(range_diagnostic(issue, declaration));
}

pub(super) fn range_diagnostic(issue: &RangeIssue, declaration: ByteSpan) -> Diagnostic {
    Diagnostic {
        id: String::new(),
        code: "SES-T0101".to_owned(),
        severity: DiagnosticSeverity::Error,
        message_key: "range.endpoint-not-int".to_owned(),
        primary: byte_range(issue.endpoint),
        related: vec![RelatedDiagnostic {
            message: format!(
                "range {} must be Int, received {}",
                issue.position,
                type_label(&issue.actual)
            ),
            primary: byte_range(declaration),
        }],
        fixes: Vec::new(),
    }
}

fn byte_range(span: ByteSpan) -> ByteRange {
    ByteRange {
        start: span.start,
        end: span.end,
    }
}

#[cfg(test)]
mod tests {
    use crate::semantic_diagnostics;

    #[test]
    fn reports_non_int_range_endpoints() {
        for source in [
            "pub fn bad -> Range<Int> = \"one\"..10\n",
            "pub fn bad -> Range<Int> = 1..=\"ten\"\n",
        ] {
            let artifact = semantic_diagnostics("range-endpoint.ssrg", source);

            assert_eq!(artifact.diagnostics.len(), 1, "{artifact:#?}");
            assert_eq!(artifact.diagnostics[0].code, "SES-T0101");
            assert_eq!(
                artifact.diagnostics[0].message_key,
                "range.endpoint-not-int"
            );
        }
    }
}
