use crate::typed::{PureCallIssue, PureFunctionAnalysis};
use seseragi_syntax::{ByteRange, Diagnostic, DiagnosticSeverity, RelatedDiagnostic};

use super::type_labels::type_label;

pub(super) fn collect_pure_function_diagnostics(
    analysis: &PureFunctionAnalysis,
    span: seseragi_syntax::ByteSpan,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some(issue) = &analysis.pure_call_issue {
        diagnostics.push(call_diagnostic(issue.clone(), span));
    }
}

pub(super) fn call_diagnostic(
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
        PureCallIssue::MissingInstance { callee, constraint } => (
            "instance.missing",
            callee,
            format!(
                "no {} instance matches the inferred call arguments",
                constraint.name
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

#[cfg(test)]
mod tests {
    use crate::semantic_diagnostics;

    #[test]
    fn reports_a_non_function_higher_order_argument() {
        let artifact = semantic_diagnostics(
            "higher-order-mismatch.ssrg",
            concat!(
                "fn apply f: (Int -> Int) -> value: Int -> Int = f value\n",
                "fn broken value: Int -> Int = apply value value\n",
            ),
        );

        assert_eq!(artifact.diagnostics.len(), 1);
        assert_eq!(artifact.diagnostics[0].code, "SES-T0101");
        assert_eq!(
            artifact.diagnostics[0].message_key,
            "call.argument-type-mismatch"
        );
    }

    #[test]
    fn reports_missing_reducible_evidence_instead_of_an_unresolved_name() {
        let artifact = semantic_diagnostics(
            "missing-reducible.ssrg",
            "fn broken value: Int -> Int = reduce 0 (+) value\n",
        );

        assert_eq!(artifact.diagnostics.len(), 1);
        assert_eq!(artifact.diagnostics[0].code, "SES-T0101");
        assert_eq!(artifact.diagnostics[0].message_key, "instance.missing");
    }

    #[test]
    fn reports_a_missing_local_trait_instance_at_the_call_site() {
        let artifact = semantic_diagnostics(
            "missing-render.ssrg",
            "type Badge = | Active\n\
             trait Render<A> { fn render value: A -> String }\n\
             fn label value: Badge -> String = render value\n",
        );

        assert_eq!(artifact.diagnostics.len(), 1);
        assert_eq!(artifact.diagnostics[0].code, "SES-T0101");
        assert_eq!(artifact.diagnostics[0].message_key, "instance.missing");
    }
}
