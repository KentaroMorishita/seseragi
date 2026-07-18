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
        RecordIssue::SpreadOnNonRecord { spread, actual } => (
            "record.spread-on-non-record",
            *spread,
            format!(
                "record spread requires a record, received {}",
                type_label(actual)
            ),
        ),
        RecordIssue::UnknownStructField {
            field,
            name,
            structure,
        } => (
            "struct.field-unresolved",
            *field,
            format!("struct `{structure}` has no field `{name}`"),
        ),
        RecordIssue::MissingStructField { structure, name } => (
            "struct.field-missing",
            *structure,
            format!("struct construction is missing required field `{name}`"),
        ),
        RecordIssue::StructRepresentationPrivate { structure, name } => (
            "struct.representation-private",
            *structure,
            format!("struct `{name}` representation is private to its defining module"),
        ),
        RecordIssue::StructTypeArgumentsUnresolved { structure, name } => (
            "struct.type-arguments-unresolved",
            *structure,
            format!(
                "struct `{name}` type arguments cannot be inferred from its fields or expected type"
            ),
        ),
        RecordIssue::StructFieldType {
            field,
            name,
            expected,
            actual,
        } => (
            "struct.field-type-mismatch",
            *field,
            format!(
                "struct field `{name}` requires {}, received {}",
                type_label(expected),
                type_label(actual)
            ),
        ),
        RecordIssue::StructSpreadType {
            spread,
            expected,
            actual,
        } => (
            "struct.spread-type-mismatch",
            *spread,
            format!(
                "struct update requires {}, received {}",
                type_label(expected),
                type_label(actual)
            ),
        ),
        RecordIssue::StructSpreadPosition { spread } => (
            "struct.spread-not-first",
            *spread,
            "struct spread must be the first item".to_owned(),
        ),
        RecordIssue::MultipleStructSpreads { spread } => (
            "struct.multiple-spreads",
            *spread,
            "struct construction accepts at most one spread".to_owned(),
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
    fn types_optional_field_access_as_maybe() {
        let artifact = semantic_diagnostics(
            "record-optional.ssrg",
            "pub fn optionalId user: { id?: String } -> Maybe<String> = user.id\n",
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
                "pub fn bad -> { name: String } = { ...42, name: \"A\" }\n",
                "record.spread-on-non-record",
            ),
        ] {
            let artifact = semantic_diagnostics("record-invalid.ssrg", source);
            assert_eq!(artifact.diagnostics.len(), 1, "{source}\n{artifact:#?}");
            assert_eq!(artifact.diagnostics[0].code, "SES-T0101");
            assert_eq!(artifact.diagnostics[0].message_key, expected);
        }
    }

    #[test]
    fn accepts_nominal_struct_construction_update_access_and_pattern() {
        let artifact = semantic_diagnostics(
            "struct-user.ssrg",
            concat!(
                "pub struct User { name: String, score: Int }\n",
                "fn rename user: User -> User = User { ...user, name: \"Mio\" }\n",
                "fn display user: User -> String = match user { User { name } -> name }\n",
                "pub fn answer -> String = User { name: \"Aki\", score: 42 } |> rename |> display\n",
            ),
        );

        assert!(artifact.diagnostics.is_empty(), "{artifact:#?}");
    }

    #[test]
    fn infers_generic_struct_arguments_from_fields_and_spread_updates() {
        let artifact = semantic_diagnostics(
            "struct-box.ssrg",
            concat!(
                "pub struct Box<A> { value: A }\n",
                "pub let inferred = Box { value: 42 }\n",
                "fn replace<A> value: A -> box: Box<A> -> Box<A> = Box { ...box, value }\n",
                "fn unwrap<A> box: Box<A> -> A = box.value\n",
                "pub fn answer -> Int = inferred |> replace 42 |> unwrap\n",
            ),
        );

        assert!(artifact.diagnostics.is_empty(), "{artifact:#?}");
    }

    #[test]
    fn reports_conflicting_or_unresolved_generic_struct_arguments() {
        for (source, expected) in [
            (
                concat!(
                    "pub struct Pair<A> { left: A, right: A }\n",
                    "pub let bad = Pair { left: 1, right: \"two\" }\n",
                ),
                "struct.field-type-mismatch",
            ),
            (
                concat!(
                    "pub struct Marker<A> { label: String }\n",
                    "pub let bad = Marker { label: \"unknown\" }\n",
                ),
                "struct.type-arguments-unresolved",
            ),
        ] {
            let artifact = semantic_diagnostics("struct-generic-invalid.ssrg", source);
            assert_eq!(artifact.diagnostics.len(), 1, "{source}\n{artifact:#?}");
            assert_eq!(artifact.diagnostics[0].code, "SES-T0101");
            assert_eq!(artifact.diagnostics[0].message_key, expected);
        }
    }

    #[test]
    fn reports_nominal_struct_construction_errors() {
        for (source, expected) in [
            (
                "pub struct User { name: String, score: Int }\npub fn bad -> User = User { name: \"A\" }\n",
                "struct.field-missing",
            ),
            (
                "pub struct User { name: String }\npub fn bad -> User = User { name: 42 }\n",
                "struct.field-type-mismatch",
            ),
            (
                "pub struct User { name: String }\npub fn bad -> User = User { age: \"A\", name: \"B\" }\n",
                "struct.field-unresolved",
            ),
            (
                "pub struct User { name: String }\npub struct Team { name: String }\npub fn bad team: Team -> User = User { ...team }\n",
                "struct.spread-type-mismatch",
            ),
        ] {
            let artifact = semantic_diagnostics("struct-invalid.ssrg", source);
            assert_eq!(artifact.diagnostics.len(), 1, "{artifact:#?}");
            assert_eq!(artifact.diagnostics[0].code, "SES-T0101");
            assert_eq!(artifact.diagnostics[0].message_key, expected);
        }
    }
}
