use crate::typed::RecordIssue;
use seseragi_syntax::{ByteRange, ByteSpan, Diagnostic, DiagnosticSeverity, RelatedDiagnostic};

use super::type_labels::type_label;

pub(super) fn collect_record_diagnostic(
    issue: Option<&RecordIssue>,
    declaration: ByteSpan,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(issue) = issue else {
        return;
    };
    diagnostics.push(record_diagnostic(issue, declaration));
}

pub(super) fn record_diagnostic(issue: &RecordIssue, declaration: ByteSpan) -> Diagnostic {
    let (message_key, primary, message) = match issue {
        RecordIssue::DuplicateField { field, name } => (
            "record.duplicate-field",
            *field,
            format!("record field `{name}` is declared more than once"),
        ),
        RecordIssue::MissingField { field, name } => (
            "record.field-unresolved",
            *field,
            format!("record type has no required field `{name}`"),
        ),
        RecordIssue::AccessOnNonRecord { receiver, actual } => (
            "record.access-on-non-record",
            *receiver,
            format!(
                "field access requires a record, received {}",
                type_label(actual)
            ),
        ),
        RecordIssue::OptionalAccessUnsupported { field, name } => (
            "record.optional-access-not-connected",
            *field,
            format!("optional field `{name}` access is not connected to Maybe yet"),
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
    fn accepts_record_width_subtyping_and_required_field_access() {
        let artifact = semantic_diagnostics(
            "record-width.ssrg",
            concat!(
                "fn displayName user: { name: String } -> String = user.name\n",
                "pub fn answer -> String = displayName { name: \"Mio\", score: 42 }\n",
            ),
        );

        assert!(artifact.diagnostics.is_empty(), "{artifact:#?}");
    }

    #[test]
    fn reports_invalid_record_construction_and_access() {
        for (source, expected) in [
            (
                "pub fn bad -> { name: String } = { name: \"A\", name: \"B\" }\n",
                "record.duplicate-field",
            ),
            (
                "pub fn bad user: { name: String } -> String = user.age\n",
                "record.field-unresolved",
            ),
            (
                "pub fn bad -> String = 42.name\n",
                "record.access-on-non-record",
            ),
            (
                "pub fn bad user: { name?: String } -> String = user.name\n",
                "record.optional-access-not-connected",
            ),
        ] {
            let artifact = semantic_diagnostics("record-invalid.ssrg", source);
            assert_eq!(artifact.diagnostics.len(), 1, "{artifact:#?}");
            assert_eq!(artifact.diagnostics[0].code, "SES-T0101");
            assert_eq!(artifact.diagnostics[0].message_key, expected);
        }
    }
}
