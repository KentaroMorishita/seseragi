use serde::Serialize;
use seseragi_source::{EncodedPosition, LineIndex, LineIndexError, PositionEncoding};
use seseragi_syntax::{DiagnosticArtifact, DiagnosticSeverity};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Position {
    line: usize,
    character: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
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
            Ok(LspDiagnostic {
                range: Range {
                    start: position(index.try_locate_encoded(diagnostic.primary.start, encoding)?),
                    end: position(index.try_locate_encoded(diagnostic.primary.end, encoding)?),
                },
                severity: severity(diagnostic.severity),
                code: diagnostic.code.clone(),
                source: "seseragi",
                message: diagnostic.message_key.clone(),
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
    use seseragi_syntax::parse_diagnostics;

    #[test]
    fn converts_byte_ranges_to_utf16_positions() {
        let source = "// 🙂\npub let broken: Int =\n";
        let artifact = parse_diagnostics("memory.ssrg", source);
        let diagnostics = convert(&artifact, source, PositionEncoding::Utf16).unwrap();

        assert!(!diagnostics.is_empty());
        assert!(diagnostics.iter().all(|item| item.source == "seseragi"));
        assert!(diagnostics.iter().all(|item| item.range.start.line >= 1));
    }
}
