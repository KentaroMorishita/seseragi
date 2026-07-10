use crate::cst::parse_cst_from_tokens;
use crate::lexer::lex;
use crate::surface::parse_surface_ast;
use crate::surface_model::SurfaceDecl;
use crate::{CstArtifact, CstError, CstMissing, CstNode, Token, TokenKind};
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
    let source_name = source_name.into();
    let tokens = lex(source_name.clone(), source);
    let literal_diagnostics = integer_literal_diagnostics(&tokens.tokens);
    let source_tokens = tokens.tokens.clone();
    let surface = parse_surface_ast(source_name, source);
    let cst = parse_cst_from_tokens(tokens);
    let mut artifact = diagnostics_from_cst(&cst, &source_tokens);
    let surface_declaration_diagnostics = missing_surface_declaration_diagnostics(
        &cst.root,
        &surface.declarations,
        &source_tokens,
        &artifact.diagnostics,
    );
    append_diagnostics(&mut artifact, surface_declaration_diagnostics);
    let surface_diagnostics = missing_surface_body_diagnostics(
        &surface.declarations,
        &source_tokens,
        &artifact.diagnostics,
    );
    append_diagnostics(&mut artifact, surface_diagnostics);
    append_diagnostics(&mut artifact, literal_diagnostics);
    artifact
}

fn append_diagnostics(artifact: &mut DiagnosticArtifact, diagnostics: Vec<Diagnostic>) {
    let next_id = artifact.diagnostics.len() + 1;
    artifact
        .diagnostics
        .extend(
            diagnostics
                .into_iter()
                .enumerate()
                .map(|(index, mut diagnostic)| {
                    diagnostic.id = format!("d{}", next_id + index);
                    diagnostic
                }),
        );
}

fn missing_surface_declaration_diagnostics(
    root: &CstNode,
    declarations: &[SurfaceDecl],
    tokens: &[Token],
    existing: &[Diagnostic],
) -> Vec<Diagnostic> {
    let surface_type_starts = declarations
        .iter()
        .filter_map(|declaration| match declaration {
            SurfaceDecl::Type { span, .. } => Some(span.start),
            _ => None,
        })
        .collect::<Vec<_>>();

    root.children
        .iter()
        .filter(|top| top.children.iter().any(|child| child.kind == "type-decl"))
        .filter_map(|top| {
            let range = byte_range_for_node(top, tokens)?;
            if surface_type_starts.contains(&range.start)
                || existing.iter().any(|diagnostic| {
                    diagnostic.primary.start >= range.start && diagnostic.primary.start <= range.end
                })
            {
                return None;
            }
            Some(Diagnostic {
                id: String::new(),
                code: "SES-P0001".to_owned(),
                severity: DiagnosticSeverity::Error,
                message_key: "parser.invalid-type-declaration".to_owned(),
                primary: range,
                related: Vec::new(),
                fixes: Vec::new(),
            })
        })
        .collect()
}

fn byte_range_for_node(node: &CstNode, tokens: &[Token]) -> Option<ByteRange> {
    let source_tokens = tokens.get(node.start_token..node.end_token)?;
    let start = source_tokens
        .iter()
        .find(|token| !is_trivia(token.kind))?
        .start;
    let end = source_tokens
        .iter()
        .rev()
        .find(|token| !is_trivia(token.kind))
        .map(|token| token.end)
        .unwrap_or(start);
    Some(ByteRange { start, end })
}

fn missing_surface_body_diagnostics(
    declarations: &[SurfaceDecl],
    tokens: &[Token],
    existing: &[Diagnostic],
) -> Vec<Diagnostic> {
    declarations
        .iter()
        .filter_map(|declaration| {
            let span = match declaration {
                SurfaceDecl::Let {
                    body: None, span, ..
                }
                | SurfaceDecl::Fn {
                    body: None, span, ..
                }
                | SurfaceDecl::EffectFn {
                    body: None, span, ..
                } => *span,
                _ => return None,
            };
            if existing.iter().any(|diagnostic| {
                diagnostic.code == "SES-P0001"
                    && diagnostic.primary.start >= span.start
                    && diagnostic.primary.start <= span.end
            }) {
                return None;
            }
            let primary = malformed_tuple_body_range(tokens, span.start, span.end)?;
            Some(Diagnostic {
                id: String::new(),
                code: "SES-P0001".to_owned(),
                severity: DiagnosticSeverity::Error,
                message_key: "parser.expected-expression".to_owned(),
                primary,
                related: Vec::new(),
                fixes: Vec::new(),
            })
        })
        .collect()
}

fn malformed_tuple_body_range(
    tokens: &[Token],
    declaration_start: usize,
    declaration_end: usize,
) -> Option<ByteRange> {
    let equals = tokens.iter().find(|token| {
        token.kind == TokenKind::OperatorEquals
            && token.start >= declaration_start
            && token.end <= declaration_end
    });
    let equals = equals?;
    let body = tokens
        .iter()
        .filter(|token| {
            token.start >= equals.end
                && token.end <= declaration_end
                && token.kind != TokenKind::Eof
                && !is_trivia(token.kind)
        })
        .collect::<Vec<_>>();
    match (body.first(), body.last()) {
        (Some(first), Some(last))
            if first.kind == TokenKind::PunctuationParenLeft
                && last.kind == TokenKind::PunctuationParenRight
                && body
                    .iter()
                    .rev()
                    .nth(1)
                    .is_some_and(|token| token.kind == TokenKind::PunctuationComma) =>
        {
            Some(ByteRange {
                start: first.start,
                end: last.end,
            })
        }
        _ => None,
    }
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

fn diagnostics_from_cst(cst: &CstArtifact, tokens: &[Token]) -> DiagnosticArtifact {
    let diagnostics = cst
        .errors
        .iter()
        .enumerate()
        .map(|(index, error)| diagnostic_from_cst_error(index, error, &cst.missing, tokens))
        .collect();

    DiagnosticArtifact {
        schema: 1,
        source: cst.source.clone(),
        position_encoding: "utf-8".to_owned(),
        diagnostics,
    }
}

fn diagnostic_from_cst_error(
    index: usize,
    error: &CstError,
    missing: &[CstMissing],
    tokens: &[Token],
) -> Diagnostic {
    let primary = primary_range_for_error(error, missing, tokens);
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

fn primary_range_for_error(
    error: &CstError,
    missing: &[CstMissing],
    tokens: &[Token],
) -> ByteRange {
    if let Some(missing) = missing
        .iter()
        .find(|missing| missing.at_token == error.start_token)
    {
        return ByteRange {
            start: missing.at_byte,
            end: missing.at_byte,
        };
    }

    let start = tokens
        .get(error.start_token)
        .map(|token| token.start)
        .unwrap_or_else(|| tokens.last().map(|token| token.end).unwrap_or(0));
    let end = error
        .end_token
        .checked_sub(1)
        .and_then(|index| tokens.get(index))
        .map(|token| token.end)
        .unwrap_or(start)
        .max(start);
    ByteRange { start, end }
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
    fn reports_malformed_tuple_expressions_instead_of_silently_dropping_the_body() {
        for source in ["pub let singleton = (1,)\n", "pub let trailing = (1, 2,)\n"] {
            let diagnostics = parse_diagnostics("main.ssrg", source);

            assert_eq!(diagnostics.diagnostics.len(), 1);
            assert_eq!(diagnostics.diagnostics[0].code, "SES-P0001");
            assert!(
                diagnostics.diagnostics[0].primary.start < diagnostics.diagnostics[0].primary.end
            );
        }
    }

    #[test]
    fn reports_invalid_adt_names_at_the_source_token() {
        for (source, expected) in [
            ("type bad = | Rock\n", ByteRange { start: 5, end: 8 }),
            ("type Bad = | rock\n", ByteRange { start: 13, end: 17 }),
        ] {
            let diagnostics = parse_diagnostics("main.ssrg", source);

            assert_eq!(diagnostics.diagnostics.len(), 1, "{source:?}");
            assert_eq!(diagnostics.diagnostics[0].code, "SES-P0001");
            assert_eq!(diagnostics.diagnostics[0].primary, expected);
        }
    }

    #[test]
    fn reports_an_empty_adt_at_the_missing_variant_position() {
        let source = "type Empty =\n";
        let diagnostics = parse_diagnostics("main.ssrg", source);

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-P0001");
        assert_eq!(
            diagnostics.diagnostics[0].primary,
            ByteRange {
                start: source.find('\n').expect("fixture contains a newline"),
                end: source.find('\n').expect("fixture contains a newline"),
            }
        );
    }

    #[test]
    fn reports_an_adt_payload_that_surface_syntax_cannot_normalize() {
        let source = "type Bad = | Good Int extra\npub let answer: Int = 42\n";
        let diagnostics = parse_diagnostics("main.ssrg", source);

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-P0001");
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "parser.invalid-type-declaration"
        );
        assert_eq!(
            diagnostics.diagnostics[0].primary,
            ByteRange { start: 0, end: 27 }
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
