use crate::{parse_cst, CstArtifact, CstError, CstMissing};
use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticArtifact {
    pub schema: u32,
    pub source: String,
    #[serde(rename = "positionEncoding")]
    pub position_encoding: String,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub id: String,
    pub code: String,
    pub severity: DiagnosticSeverity,
    #[serde(rename = "messageKey")]
    pub message_key: String,
    pub primary: ByteRange,
    pub related: Vec<RelatedDiagnostic>,
    pub fixes: Vec<DiagnosticFix>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct ByteRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RelatedDiagnostic {
    pub message: String,
    pub primary: ByteRange,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DiagnosticFix {
    pub title: String,
    pub edits: Vec<DiagnosticEdit>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DiagnosticEdit {
    pub range: ByteRange,
    pub replacement: String,
}

pub fn parse_diagnostics(source_name: impl Into<String>, source: &str) -> DiagnosticArtifact {
    diagnostics_from_cst(parse_cst(source_name, source))
}

fn diagnostics_from_cst(cst: CstArtifact) -> DiagnosticArtifact {
    let diagnostics = cst
        .errors
        .iter()
        .enumerate()
        .map(|(index, error)| diagnostic_from_cst_error(index, error, &cst.missing))
        .collect();

    DiagnosticArtifact {
        schema: 1,
        source: cst.source,
        position_encoding: "utf-8".to_owned(),
        diagnostics,
    }
}

fn diagnostic_from_cst_error(index: usize, error: &CstError, missing: &[CstMissing]) -> Diagnostic {
    let primary = primary_range_for_error(error, missing);
    Diagnostic {
        id: format!("d{}", index + 1),
        code: error.code.clone(),
        severity: DiagnosticSeverity::Error,
        message_key: message_key_for_code(&error.code).to_owned(),
        primary,
        related: Vec::new(),
        fixes: Vec::new(),
    }
}

fn primary_range_for_error(error: &CstError, missing: &[CstMissing]) -> ByteRange {
    let start = missing
        .iter()
        .find(|missing| missing.at_token == error.start_token)
        .map(|missing| missing.at_byte)
        .unwrap_or(error.start_token);
    ByteRange { start, end: start }
}

fn message_key_for_code(code: &str) -> &str {
    match code {
        "SES-P0001" => "parser.expected-expression",
        _ => "parser.error",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_source_diagnostics_for_valid_module() {
        let diagnostics = parse_diagnostics("main.ssrg", "pub let answer: Int = 42\n");

        assert_eq!(diagnostics.schema, 1);
        assert_eq!(diagnostics.source, "main.ssrg");
        assert_eq!(diagnostics.position_encoding, "utf-8");
        assert!(diagnostics.diagnostics.is_empty());
    }

    #[test]
    fn reports_missing_let_expression() {
        let diagnostics = parse_diagnostics("main.ssrg", "pub let answer: Int =");

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].id, "d1");
        assert_eq!(diagnostics.diagnostics[0].code, "SES-P0001");
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "parser.expected-expression"
        );
        assert_eq!(
            diagnostics.diagnostics[0].primary,
            ByteRange { start: 21, end: 21 }
        );
    }
}
