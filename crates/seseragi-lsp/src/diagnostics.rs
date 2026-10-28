use serde::Serialize;
use seseragi_source::{EncodedPosition, LineIndex, LineIndexError, PositionEncoding};
use seseragi_syntax::{DiagnosticArtifact, DiagnosticSeverity};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    line: usize,
    character: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Range {
    start: Position,
    end: Position,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LspDiagnostic {
    range: Range,
    severity: u8,
    code: String,
    source: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<u8>,
    related_information: Vec<LspRelatedInformation>,
    data: LspDiagnosticData,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct LspRelatedInformation {
    location: LspLocation,
    message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct LspLocation {
    uri: String,
    range: Range,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct LspDiagnosticData {
    #[serde(skip_serializing_if = "Option::is_none")]
    lint: Option<seseragi_syntax::LintEvidence>,
    message_key: String,
    notes: Vec<String>,
    helps: Vec<String>,
    fixes: Vec<seseragi_syntax::DiagnosticFix>,
    expected_type: Option<String>,
    actual_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    type_difference: Option<seseragi_syntax::TypeDifference>,
    /// Preserve foreign UTF-8 byte locations even if that document is currently
    /// unavailable. They must never be interpreted in the primary text.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    related_sources: Vec<seseragi_syntax::RelatedDiagnostic>,
}

#[cfg(test)]
fn convert(
    artifact: &DiagnosticArtifact,
    source: &str,
    encoding: PositionEncoding,
) -> Result<Vec<LspDiagnostic>, LineIndexError> {
    convert_with_sources(artifact, source, encoding, |_| None)
}

pub(crate) struct RelatedSource<'a> {
    pub uri: &'a str,
    pub text: &'a str,
}

pub(crate) fn convert_with_sources<'a>(
    artifact: &DiagnosticArtifact,
    source: &str,
    encoding: PositionEncoding,
    related_source: impl Fn(&str) -> Option<RelatedSource<'a>>,
) -> Result<Vec<LspDiagnostic>, LineIndexError> {
    let index = LineIndex::new(source);
    artifact
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let (expected_type, actual_type) = diagnostic.expected_actual_types();
            Ok(LspDiagnostic {
                range: Range {
                    start: position(index.try_locate_encoded(diagnostic.primary.start, encoding)?),
                    end: position(index.try_locate_encoded(diagnostic.primary.end, encoding)?),
                },
                severity: severity(diagnostic.severity),
                code: diagnostic.code.clone(),
                source: "seseragi",
                message: diagnostic.message(),
                tags: if diagnostic.code == "SES-L0201" {
                    vec![2]
                } else {
                    Vec::new()
                },
                related_information: diagnostic
                    .labels()
                    .iter()
                    .filter_map(|label| {
                        let (uri, text) = match label.source.as_deref() {
                            Some(name) => match related_source(name) {
                                Some(related) => (related.uri, related.text),
                                None if name == artifact.source => {
                                    (artifact.source.as_str(), source)
                                }
                                None => return None,
                            },
                            None => (artifact.source.as_str(), source),
                        };
                        let index = LineIndex::new(text);
                        Some((|| {
                            Ok(LspRelatedInformation {
                                location: LspLocation {
                                    uri: uri.to_owned(),
                                    range: Range {
                                        start: position(
                                            index.try_locate_encoded(
                                                label.primary.start,
                                                encoding,
                                            )?,
                                        ),
                                        end: position(
                                            index
                                                .try_locate_encoded(label.primary.end, encoding)?,
                                        ),
                                    },
                                },
                                message: label.message.clone(),
                            })
                        })())
                    })
                    .collect::<Result<Vec<_>, LineIndexError>>()?,
                data: LspDiagnosticData {
                    lint: diagnostic.lint_evidence(),
                    message_key: diagnostic.message_key.clone(),
                    notes: diagnostic.notes(),
                    helps: diagnostic.helps(),
                    fixes: diagnostic.fixes.clone(),
                    expected_type,
                    actual_type,
                    type_difference: diagnostic.type_difference.clone(),
                    related_sources: diagnostic
                        .related
                        .iter()
                        .filter(|label| {
                            label
                                .source
                                .as_deref()
                                .is_some_and(|name| name != artifact.source)
                        })
                        .cloned()
                        .collect(),
                },
            })
        })
        .collect()
}

fn position(position: EncodedPosition) -> Position {
    Position {
        line: position.line,
        character: position.character,
    }
}

fn severity(severity: DiagnosticSeverity) -> u8 {
    match severity {
        DiagnosticSeverity::Error => 1,
        DiagnosticSeverity::Warning => 2,
        DiagnosticSeverity::Information => 3,
        DiagnosticSeverity::Hint => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use seseragi_syntax::{
        parse_diagnostics, ByteRange, Diagnostic, DiagnosticArtifact, DiagnosticSeverity,
        RelatedDiagnostic,
    };

    #[test]
    fn converts_byte_ranges_to_utf16_positions() {
        let source = "// 🙂\npub let broken: Int =\n";
        let artifact = parse_diagnostics("memory.ssrg", source);
        let diagnostics = convert(&artifact, source, PositionEncoding::Utf16).unwrap();

        assert!(!diagnostics.is_empty());
        assert!(diagnostics.iter().all(|item| item.source == "seseragi"));
        assert!(diagnostics.iter().all(|item| item.range.start.line >= 1));
        assert!(diagnostics
            .iter()
            .all(|item| !item.message.contains("parser.")));
        assert!(diagnostics.iter().all(|item| !item.data.helps.is_empty()));
    }

    #[test]
    fn converts_unicode_primary_and_related_ranges_without_byte_drift() {
        let source = "🙂missing";
        let artifact = DiagnosticArtifact {
            schema: 1,
            source: "file:///unicode.ssrg".to_owned(),
            position_encoding: "utf-8-byte-offset".to_owned(),
            diagnostics: vec![Diagnostic {
                type_difference: None,
                id: "D1".to_owned(),
                code: "SES-N0001".to_owned(),
                severity: DiagnosticSeverity::Error,
                message_key: "name.unresolved".to_owned(),
                primary: ByteRange { start: 4, end: 11 },
                related: vec![RelatedDiagnostic {
                    source: None,
                    message: "unresolved value name".to_owned(),
                    primary: ByteRange { start: 0, end: 4 },
                }],
                fixes: Vec::new(),
            }],
        };

        let utf16 = convert(&artifact, source, PositionEncoding::Utf16).unwrap();
        assert_eq!(utf16[0].range.start.character, 2);
        assert_eq!(utf16[0].range.end.character, 9);
        assert_eq!(
            utf16[0].related_information[0].location.range.end.character,
            2
        );

        let utf8 = convert(&artifact, source, PositionEncoding::Utf8).unwrap();
        assert_eq!(utf8[0].range.start.character, 4);
        assert_eq!(utf8[0].range.end.character, 11);
    }

    #[test]
    fn external_related_source_is_never_encoded_using_primary_text() {
        let source = "old";
        let provider = "// 元の宣言 🙂\npub deprecated \"Use modern\" fn old -> Int = 1\n";
        let begin = provider.find("deprecated").unwrap();
        let artifact = DiagnosticArtifact {
            schema: 1,
            source: "file:///consumer.ssrg".to_owned(),
            position_encoding: "utf-8-byte-offset".to_owned(),
            diagnostics: vec![Diagnostic {
                id: "l1".to_owned(),
                code: "SES-L0201".to_owned(),
                severity: DiagnosticSeverity::Warning,
                message_key: "lint.deprecated-symbol".to_owned(),
                primary: ByteRange { start: 0, end: 3 },
                related: vec![RelatedDiagnostic {
                    source: Some("/provider/model.ssrg".to_owned()),
                    message: "Use modern".to_owned(),
                    primary: ByteRange {
                        start: begin,
                        end: begin + "deprecated".len(),
                    },
                }],
                fixes: Vec::new(),
                type_difference: None,
            }],
        };
        let unavailable = convert(&artifact, source, PositionEncoding::Utf16).unwrap();
        assert!(unavailable[0].related_information.is_empty());
        assert_eq!(
            unavailable[0].data.related_sources[0].source.as_deref(),
            Some("/provider/model.ssrg")
        );
        assert_eq!(unavailable[0].tags, vec![2]);
        for encoding in [PositionEncoding::Utf8, PositionEncoding::Utf16] {
            let converted = convert_with_sources(&artifact, source, encoding, |name| {
                assert_eq!(name, "/provider/model.ssrg");
                Some(RelatedSource {
                    uri: "file:///provider/model.ssrg",
                    text: provider,
                })
            })
            .unwrap();
            let location = &converted[0].related_information[0].location;
            assert_eq!(location.uri, "file:///provider/model.ssrg");
            let expected = LineIndex::new(provider)
                .try_locate_encoded(begin, encoding)
                .unwrap();
            assert_eq!(location.range.start, position(expected));
        }
    }
}
