use crate::typed::ArrayIssue;
use seseragi_syntax::{ByteRange, ByteSpan, Diagnostic, DiagnosticSeverity, RelatedDiagnostic};

use super::type_labels::type_label;

pub(super) fn collect_array_diagnostic(
    issue: Option<&ArrayIssue>,
    declaration: ByteSpan,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(issue) = issue else {
        return;
    };
    diagnostics.push(array_diagnostic(issue, declaration));
}

pub(super) fn array_diagnostic(issue: &ArrayIssue, declaration: ByteSpan) -> Diagnostic {
    let (message_key, primary, message) = match issue {
        ArrayIssue::EmptyWithoutExpectedType { array } => (
            "array.empty-element-type-unknown",
            *array,
            "add an Array element type annotation".to_owned(),
        ),
        ArrayIssue::ElementTypeMismatch {
            element,
            index,
            expected,
            actual,
        } => (
            "array.element-type-mismatch",
            *element,
            format!(
                "element {} expected {}, received {}",
                index + 1,
                type_label(expected),
                type_label(actual)
            ),
        ),
    };
    Diagnostic {
        id: String::new(),
        code: "SES-T0101".to_owned(),
        severity: DiagnosticSeverity::Error,
        message_key: message_key.to_owned(),
        primary: byte_range(primary),
        related: vec![RelatedDiagnostic {
            message,
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
    fn accepts_empty_array_with_a_contextual_element_type() {
        let artifact =
            semantic_diagnostics("array-empty.ssrg", "pub fn empty -> Array<Int> = []\n");

        assert!(artifact.diagnostics.is_empty(), "{artifact:#?}");
    }

    #[test]
    fn reports_a_heterogeneous_array_element() {
        let artifact = semantic_diagnostics(
            "array-mismatch.ssrg",
            "pub fn arrays -> Array<Int> = [1, \"bad\"]\n",
        );

        assert_eq!(artifact.diagnostics.len(), 1);
        assert_eq!(artifact.diagnostics[0].code, "SES-T0101");
        assert_eq!(
            artifact.diagnostics[0].message_key,
            "array.element-type-mismatch"
        );
    }

    #[test]
    fn reports_an_array_issue_inside_an_effect_do_let() {
        let artifact = semantic_diagnostics(
            "array-effect.ssrg",
            concat!(
                "pub effect fn main -> Unit\n",
                "with Console\n",
                "fails ConsoleError =\n",
                "  do {\n",
                "    let values = [1, \"bad\"]\n",
                "    println \"done\"\n",
                "  }\n",
            ),
        );

        assert_eq!(artifact.diagnostics.len(), 1);
        assert_eq!(
            artifact.diagnostics[0].message_key,
            "array.element-type-mismatch"
        );
    }
}
