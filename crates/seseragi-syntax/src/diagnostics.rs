use crate::cst::parse_cst_from_tokens;
use crate::lexer::lex;
use crate::{CstArtifact, CstError, CstMissing, Token, TokenKind};
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
    let tokens = lex(source_name, source);
    let literal_diagnostics = integer_literal_diagnostics(&tokens.tokens);
    let mut artifact = diagnostics_from_cst(parse_cst_from_tokens(tokens));
    let next_id = artifact.diagnostics.len() + 1;
    artifact
        .diagnostics
        .extend(
            literal_diagnostics
                .into_iter()
                .enumerate()
                .map(|(index, mut diagnostic)| {
                    diagnostic.id = format!("d{}", next_id + index);
                    diagnostic
                }),
        );
    artifact
}

fn integer_literal_diagnostics(tokens: &[Token]) -> Vec<Diagnostic> {
    tokens
        .iter()
        .enumerate()
        .filter(|(_, token)| token.kind == TokenKind::LiteralInteger)
        .filter(|(index, token)| !integer_literal_is_in_range(tokens, *index, &token.raw))
        .map(|(_, token)| Diagnostic {
            id: String::new(),
            code: "SES-P0203".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "literal.int-outside-range".to_owned(),
            primary: ByteRange {
                start: token.start,
                end: token.end,
            },
            related: Vec::new(),
            fixes: Vec::new(),
        })
        .collect()
}

fn integer_literal_is_in_range(tokens: &[Token], index: usize, raw: &str) -> bool {
    let Ok(value) = raw.parse::<u128>() else {
        return false;
    };
    if value <= i64::MAX as u128 {
        return true;
    }
    value == (i64::MAX as u128) + 1 && has_unary_minus(tokens, index)
}

fn has_unary_minus(tokens: &[Token], index: usize) -> bool {
    let significant = tokens[..index]
        .iter()
        .filter(|token| !is_trivia(token.kind))
        .collect::<Vec<_>>();
    let Some(minus) = significant.last() else {
        return false;
    };
    if minus.raw != "-" {
        return false;
    }
    significant
        .get(significant.len().saturating_sub(2))
        .is_none_or(|previous| {
            matches!(
                previous.kind,
                TokenKind::OperatorEquals
                    | TokenKind::OperatorArithmetic
                    | TokenKind::OperatorComparison
                    | TokenKind::OperatorApply
                    | TokenKind::OperatorBind
                    | TokenKind::OperatorPipeline
                    | TokenKind::PunctuationBraceLeft
                    | TokenKind::PunctuationComma
                    | TokenKind::PunctuationParenLeft
                    | TokenKind::PunctuationSquareLeft
                    | TokenKind::KeywordThen
                    | TokenKind::KeywordElse
            )
        })
}

fn is_trivia(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::TriviaComment | TokenKind::TriviaNewline | TokenKind::TriviaSpace
    )
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
        "SES-P0203" => "literal.int-outside-range",
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

    #[test]
    fn reports_integer_literal_outside_signed_64_bit_range() {
        let diagnostics =
            parse_diagnostics("main.ssrg", "pub let tooLarge: Int = 9223372036854775808\n");

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-P0203");
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "literal.int-outside-range"
        );
        assert_eq!(
            diagnostics.diagnostics[0].primary,
            ByteRange { start: 24, end: 43 }
        );
    }

    #[test]
    fn accepts_signed_64_bit_literal_boundaries() {
        let diagnostics = parse_diagnostics(
            "main.ssrg",
            "let maximum: Int = 9223372036854775807\nlet minimum: Int = -9223372036854775808\n",
        );

        assert!(diagnostics.diagnostics.is_empty());
    }
}
