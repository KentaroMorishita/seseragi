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
    message_key: String,
    notes: Vec<String>,
    helps: Vec<String>,
    fixes: Vec<seseragi_syntax::DiagnosticFix>,
    expected_type: Option<String>,
    actual_type: Option<String>,
}

pub fn convert(
    artifact: &DiagnosticArtifact,
    source: &str,
    encoding: PositionEncoding,
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
                related_information: diagnostic
                    .labels()
                    .iter()
                    .map(|label| {
                        Ok(LspRelatedInformation {
                            location: LspLocation {
                                uri: artifact.source.clone(),
                                range: Range {
                                    start: position(
                                        index.try_locate_encoded(label.primary.start, encoding)?,
                                    ),
                                    end: position(
                                        index.try_locate_encoded(label.primary.end, encoding)?,
                                    ),
                                },
                            },
                            message: label.message.clone(),
                        })
                    })
                    .collect::<Result<Vec<_>, LineIndexError>>()?,
                data: LspDiagnosticData {
                    message_key: diagnostic.message_key.clone(),
                    notes: diagnostic.notes(),
                    helps: diagnostic.helps(),
                    fixes: diagnostic.fixes.clone(),
                    expected_type,
                    actual_type,
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
                id: "D1".to_owned(),
                code: "SES-N0001".to_owned(),
                severity: DiagnosticSeverity::Error,
                message_key: "name.unresolved".to_owned(),
                primary: ByteRange { start: 4, end: 11 },
                related: vec![RelatedDiagnostic {
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
}
