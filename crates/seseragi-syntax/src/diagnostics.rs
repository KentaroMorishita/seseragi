use crate::cst::parse_cst_from_tokens;
use crate::decode_string_literal;
use crate::lexer::lex;
use crate::literal::decode_template_text;
use crate::surface::parse_surface_ast;
use crate::surface_model::SurfaceDecl;
use crate::template::{scan_template, TemplateChunk};
use crate::{CstArtifact, CstError, CstMissing, CstNode, Token, TokenKind};
use serde::Serialize;

mod surface_errors;

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
#[serde(into = "DiagnosticWire")]
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
    pub type_difference: Option<TypeDifference>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DiagnosticWire {
    id: String,
    code: String,
    severity: DiagnosticSeverity,
    message_key: String,
    message: String,
    primary: ByteRange,
    related: Vec<RelatedDiagnostic>,
    labels: Vec<RelatedDiagnostic>,
    notes: Vec<String>,
    helps: Vec<String>,
    fixes: Vec<DiagnosticFix>,
    expected_type: Option<String>,
    actual_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    type_difference: Option<TypeDifference>,
}

impl From<Diagnostic> for DiagnosticWire {
    fn from(diagnostic: Diagnostic) -> Self {
        let (expected_type, actual_type) = diagnostic.expected_actual_types();
        Self {
            message: diagnostic.message(),
            labels: diagnostic.related.clone(),
            notes: diagnostic.notes(),
            helps: diagnostic.helps(),
            expected_type,
            actual_type,
            type_difference: diagnostic.type_difference.clone(),
            id: diagnostic.id,
            code: diagnostic.code,
            severity: diagnostic.severity,
            message_key: diagnostic.message_key,
            primary: diagnostic.primary,
            related: diagnostic.related,
            fixes: diagnostic.fixes,
        }
    }
}

impl Diagnostic {
    pub fn message(&self) -> String {
        match self.message_key.as_str() {
            "name.unresolved" => "Name could not be resolved".to_owned(),
            "call.arity-mismatch" => {
                "Function was called with the wrong number of arguments".to_owned()
            }
            "call.argument-type-mismatch" => {
                "Argument type does not match the parameter type".to_owned()
            }
            "literal.invalid-escape" => {
                "Literal contains an invalid or unsupported escape sequence".to_owned()
            }
            "literal.int-outside-range" => {
                "Integer literal is outside the Int safe range".to_owned()
            }
            "function.return-type-mismatch"
            | "let.type-mismatch"
            | "if.branch-type-mismatch"
            | "match.branch-type-mismatch"
            | "struct.field-type-mismatch" => {
                "Expression type does not match the expected type".to_owned()
            }
            "instance.missing" => "A required trait instance is not available".to_owned(),
            "record.field-unresolved" => "This record has no such field".to_owned(),
            "struct.field-unresolved" => "This struct has no such field".to_owned(),
            "effect.do-statement-not-effect"
            | "effect.bind-value-not-effect"
            | "effect.compact-body-not-effect"
            | "effect.map-error-source-not-effect"
            | "for.body-not-effect" => "This position requires an Effect value".to_owned(),
            "effect.explicit-failure-mismatch" => {
                "Effect body failure does not match the declared failure type".to_owned()
            }
            "effect.explicit-success-mismatch" => {
                "Effect body success does not match the declared return type".to_owned()
            }
            "effect.explicit-environment-mismatch" => {
                "Effect body requires an environment not provided by with".to_owned()
            }
            "match.non-exhaustive" => "This match does not cover every possible value".to_owned(),
            "parser.expected-expression" => "Expected an expression here".to_owned(),
            "parser.error" => "Could not parse this syntax".to_owned(),
            "alias.arity-mismatch" => {
                "Type alias was used with the wrong number of arguments".to_owned()
            }
            "alias.kind-mismatch" => "Type alias argument has the wrong kind".to_owned(),
            "alias.cycle" => "Type alias expands recursively into itself".to_owned(),
            "alias.private-type-exposure" => "Public type alias exposes a private type".to_owned(),
            "module.initialization-order" => {
                "Top-level initializer reads a value before it is initialized".to_owned()
            }
            "module.initialization-cycle" => {
                "Top-level initialization depends recursively on itself".to_owned()
            }
            "web.html.void-children" => "Void HTML elements cannot receive children".to_owned(),
            "web.html.missing-required-prop" => {
                "This HTML tag is missing a required prop".to_owned()
            }
            "web.html.unknown-prop" => "This HTML tag has no such standard prop".to_owned(),
            "web.html.event-control-without-handler" => {
                "This event control has no matching handler".to_owned()
            }
            _ => humanize_message_key(&self.message_key),
        }
    }

    pub fn labels(&self) -> &[RelatedDiagnostic] {
        &self.related
    }

    pub fn notes(&self) -> Vec<String> {
        match self.message_key.as_str() {
            "instance.missing" => vec![
                "Trait instances are selected from the current lexical and import scope."
                    .to_owned(),
            ],
            "alias.cycle" => vec![
                "Type aliases are transparent and therefore cannot define recursive types."
                    .to_owned(),
            ],
            "module.initialization-order" => vec![
                "Top-level let values are initialized once in source order.".to_owned(),
            ],
            "module.initialization-cycle" => vec![
                "Calling a function from an initializer also evaluates the values read by that call."
                    .to_owned(),
            ],
            "call.argument-type-mismatch"
            | "function.return-type-mismatch"
            | "let.type-mismatch"
            | "if.branch-type-mismatch"
            | "match.branch-type-mismatch"
            | "struct.field-type-mismatch" => vec![
                "Seseragi does not insert an implicit conversion between these types.".to_owned(),
            ],
            _ => Vec::new(),
        }
    }

    pub fn helps(&self) -> Vec<String> {
        let help = match self.message_key.as_str() {
            "name.unresolved" => {
                "Check the spelling, or define or import the name before using it."
            }
            "literal.int-outside-range" => {
                "Use a value from -9007199254740991 through 9007199254740991, or use BigInt."
            }
            "call.arity-mismatch" => {
                "Add or remove arguments so the call matches the function signature."
            }
            "call.argument-type-mismatch"
            | "function.return-type-mismatch"
            | "let.type-mismatch"
            | "if.branch-type-mismatch"
            | "match.branch-type-mismatch"
            | "struct.field-type-mismatch" => {
                "Change the expression or its annotation so the two types agree."
            }
            "instance.missing" => {
                "Define or import a matching instance, or use a type with an available instance."
            }
            "alias.arity-mismatch" => {
                "Add or remove type arguments so the alias application matches its declaration."
            }
            "alias.kind-mismatch" => {
                "Pass a type or type constructor with the kind required by the alias parameter."
            }
            "alias.cycle" => {
                "Break the expansion cycle, or use an ADT, struct, or newtype for recursive data."
            }
            "module.initialization-order" => {
                "Move the required value before this binding, or defer the read inside a function."
            }
            "module.initialization-cycle" => {
                "Break the eager call cycle, or defer the call until after module initialization."
            }
            "alias.private-type-exposure" => {
                "Make the exposed type public, or keep the alias private."
            }
            "web.html.void-children" => {
                "Remove `children`, or use a non-void element that can contain content."
            }
            "web.html.missing-required-prop" => {
                "Add the required prop to this record literal."
            }
            "web.html.unknown-prop" => {
                "Use the suggested standard prop, or put a validated custom Attribute in `attributes`."
            }
            "web.html.event-control-without-handler" => {
                "Add `onClick`, or remove the unused event control prop."
            }
            "record.field-unresolved" | "struct.field-unresolved" => {
                "Check the field spelling and the type of the value being accessed."
            }
            "effect.do-statement-not-effect"
            | "effect.bind-value-not-effect"
            | "effect.compact-body-not-effect"
            | "effect.map-error-source-not-effect"
            | "for.body-not-effect" => {
                "Use an Effect-producing operation here, or bind a pure value with let."
            }
            "effect.explicit-failure-mismatch" => {
                "Use mapError to convert each operation failure to the declared failure type."
            }
            "effect.explicit-success-mismatch" => {
                "Return the declared success type, or change the effect function return type."
            }
            "effect.explicit-environment-mismatch" => {
                "Add the required environment field to with, using the operation's canonical type."
            }
            "match.non-exhaustive" => {
                "Add the missing pattern arms, or add a final _ arm when a catch-all is intended."
            }
            "parser.expected-expression" | "parser.error" => {
                "Complete the expression at the highlighted location."
            }
            "literal.invalid-escape" => {
                "Use a supported escape such as \\n, \\r, \\t, \\\\, a delimiter escape, or \\u{...}."
            }
            _ => return Vec::new(),
        };
        vec![help.to_owned()]
    }

    pub fn expected_actual_types(&self) -> (Option<String>, Option<String>) {
        if let Some(difference) = &self.type_difference {
            return (
                Some(difference.expected_type.clone()),
                Some(difference.actual_type.clone()),
            );
        }
        if self.message_key == "match.branch-type-mismatch" && self.related.len() >= 2 {
            return (
                self.related[0]
                    .message
                    .strip_prefix("first branch has type ")
                    .map(str::to_owned),
                self.related[1]
                    .message
                    .strip_prefix("this branch has type ")
                    .map(str::to_owned),
            );
        }
        let carries_type_pair = self.message_key.contains("type-mismatch")
            || self.message_key.contains("type-argument")
            || self.message_key.ends_with("-not-bool")
            || self.message_key == "range.endpoint-not-int"
            || self.message_key == "call.argument-type-mismatch";
        if !carries_type_pair {
            return (None, None);
        }
        for label in &self.related {
            for (left, separator) in [
                (" expected ", ", received "),
                ("expected ", ", received "),
                (" requires ", ", received "),
                ("declared ", ", body produces "),
                ("context expects ", ", annotation declares "),
                ("context expects ", ", body produces "),
            ] {
                if let Some((expected, actual)) = split_pair(&label.message, left, separator) {
                    if expected.parse::<usize>().is_err() || actual.parse::<usize>().is_err() {
                        return (Some(expected), Some(actual));
                    }
                }
            }
        }
        (None, None)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeDifference {
    pub expected_type: String,
    pub actual_type: String,
    pub entries: Vec<TypeDifferenceEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeDifferenceEntry {
    pub path: Vec<TypeDifferencePathSegment>,
    pub kind: TypeDifferenceKind,
    pub message: String,
    pub expected_type: Option<String>,
    pub actual_type: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypeDifferencePathSegment {
    RecordField { name: String },
    FunctionParameter { index: usize },
    FunctionResult,
    TypeArgument { name: String, index: usize },
    TupleElement { index: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TypeDifferenceKind {
    TypeMismatch,
    MissingRecordField,
    ExtraRecordField,
    FieldOptionality,
    MissingFunctionParameter,
    ExtraFunctionParameter,
}

fn split_pair(message: &str, left: &str, separator: &str) -> Option<(String, String)> {
    let (_, tail) = message.split_once(left)?;
    let (expected, actual) = tail.split_once(separator)?;
    Some((expected.trim().to_owned(), actual.trim().to_owned()))
}

fn humanize_message_key(message_key: &str) -> String {
    let mut words = message_key
        .split(['.', '-', '_'])
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    if let Some(first) = words.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
    words
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
    let literal_diagnostics = literal_diagnostics(&tokens.tokens);
    let template_diagnostics = template_literal_diagnostics(&tokens.tokens);
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
    let mut surface_error_context = artifact.diagnostics.clone();
    surface_error_context.extend(template_diagnostics.iter().cloned());
    let nested_surface_diagnostics =
        surface_errors::diagnostics(&surface.declarations, &surface_error_context);
    append_diagnostics(&mut artifact, nested_surface_diagnostics);
    append_diagnostics(&mut artifact, template_diagnostics);
    append_diagnostics(&mut artifact, literal_diagnostics);
    artifact
}

fn template_literal_diagnostics(tokens: &[Token]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for token in tokens
        .iter()
        .filter(|token| token.kind == TokenKind::LiteralTemplate)
    {
        for chunk in scan_template(&token.raw).chunks {
            let TemplateChunk::Text(range) = chunk else {
                continue;
            };
            let Err(error) = decode_template_text(&token.raw[range.clone()]) else {
                continue;
            };
            diagnostics.push(Diagnostic {
                type_difference: None,
                id: String::new(),
                code: "SES-P0201".to_owned(),
                severity: DiagnosticSeverity::Error,
                message_key: "literal.invalid-escape".to_owned(),
                primary: ByteRange {
                    start: token.start + range.start + error.range.start,
                    end: token.start + range.start + error.range.end,
                },
                related: Vec::new(),
                fixes: Vec::new(),
            });
        }
    }
    diagnostics
}

fn string_literal_diagnostics(tokens: &[Token]) -> Vec<Diagnostic> {
    tokens
        .iter()
        .filter(|token| {
            token.kind == TokenKind::LiteralString
                && token.raw.starts_with('"')
                && token.raw.ends_with('"')
                && token.raw.len() >= 2
        })
        .filter_map(|token| {
            let error = decode_string_literal(&token.raw).err()?;
            Some(Diagnostic {
                type_difference: None,
                id: String::new(),
                code: "SES-P0201".to_owned(),
                severity: DiagnosticSeverity::Error,
                message_key: "literal.invalid-escape".to_owned(),
                primary: ByteRange {
                    start: token.start + error.range.start,
                    end: token.start + error.range.end,
                },
                related: Vec::new(),
                fixes: Vec::new(),
            })
        })
        .collect()
}

fn literal_diagnostics(tokens: &[Token]) -> Vec<Diagnostic> {
    let mut diagnostics = integer_literal_diagnostics(tokens);
    diagnostics.extend(float_literal_diagnostics(tokens));
    diagnostics.extend(string_literal_diagnostics(tokens));
    diagnostics
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
                type_difference: None,
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
            let (span, effect_body) = match declaration {
                SurfaceDecl::Let {
                    body: None, span, ..
                }
                | SurfaceDecl::Fn {
                    body: None, span, ..
                } => (*span, false),
                SurfaceDecl::EffectFn {
                    body: None, span, ..
                } => (*span, true),
                _ => return None,
            };
            if existing.iter().any(|diagnostic| {
                diagnostic.code == "SES-P0001"
                    && diagnostic.primary.start >= span.start
                    && diagnostic.primary.start <= span.end
            }) {
                return None;
            }
            let primary = if effect_body {
                invalid_body_range(tokens, span.start, span.end)
            } else {
                malformed_pure_body_range(tokens, span.start, span.end)
            }?;
            Some(Diagnostic {
                type_difference: None,
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

fn invalid_body_range(
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
        (Some(first), Some(last)) => Some(ByteRange {
            start: first.start,
            end: last.end,
        }),
        _ => Some(ByteRange {
            start: equals.end,
            end: equals.end,
        }),
    }
}

fn malformed_pure_body_range(
    tokens: &[Token],
    declaration_start: usize,
    declaration_end: usize,
) -> Option<ByteRange> {
    let equals = tokens.iter().find(|token| {
        token.kind == TokenKind::OperatorEquals
            && token.start >= declaration_start
            && token.end <= declaration_end
    })?;
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
        (Some(first), Some(last)) if first.kind == TokenKind::OperatorLambda => Some(ByteRange {
            start: first.start,
            end: last.end,
        }),
        _ => None,
    }
}

fn integer_literal_diagnostics(tokens: &[Token]) -> Vec<Diagnostic> {
    tokens
        .iter()
        .enumerate()
        .filter(|(_, token)| token.kind == TokenKind::LiteralInteger)
        .filter(|(_, token)| !integer_literal_is_in_range(&token.raw))
        .map(|(_, token)| Diagnostic {
            type_difference: None,
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

fn integer_literal_is_in_range(raw: &str) -> bool {
    if !valid_decimal_digits(raw) || (raw.starts_with('0') && raw.len() > 1) {
        return false;
    }
    let normalized = raw.replace('_', "");
    let Ok(value) = normalized.parse::<u128>() else {
        return false;
    };
    const MAX_SAFE_INTEGER: u128 = 9_007_199_254_740_991;
    value <= MAX_SAFE_INTEGER
}

fn float_literal_diagnostics(tokens: &[Token]) -> Vec<Diagnostic> {
    tokens
        .iter()
        .filter(|token| token.kind == TokenKind::LiteralFloat)
        .filter(|token| !float_literal_is_valid(&token.raw))
        .map(|token| Diagnostic {
            type_difference: None,
            id: String::new(),
            code: "SES-P0203".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "literal.float-invalid".to_owned(),
            primary: ByteRange {
                start: token.start,
                end: token.end,
            },
            related: Vec::new(),
            fixes: Vec::new(),
        })
        .collect()
}

fn float_literal_is_valid(raw: &str) -> bool {
    let (mantissa, exponent) = raw.find(['e', 'E']).map_or((raw, None), |index| {
        (&raw[..index], Some(&raw[index + 1..]))
    });
    let mantissa_valid = if let Some(index) = mantissa.find('.') {
        if mantissa[index + 1..].contains('.') {
            return false;
        }
        let integer = &mantissa[..index];
        let fraction = &mantissa[index + 1..];
        valid_decimal_integer_part(integer) && valid_decimal_digits(fraction)
    } else {
        valid_decimal_integer_part(mantissa)
    };
    let exponent_valid = exponent.is_none_or(|exponent| {
        let digits = exponent
            .strip_prefix('+')
            .or_else(|| exponent.strip_prefix('-'))
            .unwrap_or(exponent);
        valid_decimal_digits(digits)
    });
    if !mantissa_valid || !exponent_valid {
        return false;
    }
    raw.replace('_', "")
        .parse::<f64>()
        .is_ok_and(f64::is_finite)
}

fn valid_decimal_integer_part(raw: &str) -> bool {
    valid_decimal_digits(raw) && (!raw.starts_with('0') || raw == "0")
}

fn valid_decimal_digits(raw: &str) -> bool {
    let mut previous_was_digit = false;
    let mut saw_digit = false;
    for char in raw.chars() {
        if char.is_ascii_digit() {
            previous_was_digit = true;
            saw_digit = true;
        } else if char == '_' && previous_was_digit {
            previous_was_digit = false;
        } else {
            return false;
        }
    }
    saw_digit && previous_was_digit
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
        type_difference: None,
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
        "SES-P0201" => "literal.invalid-escape",
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
    fn reports_an_invalid_template_escape() {
        let diagnostics = parse_diagnostics(
            "main.ssrg",
            r#"pub let message: String = `bad\qescape`
"#,
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-P0201");
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "literal.invalid-escape"
        );
        assert_eq!(
            diagnostics.diagnostics[0].primary,
            ByteRange { start: 30, end: 32 }
        );
    }

    #[test]
    fn reports_an_invalid_string_escape_with_human_guidance() {
        let diagnostics = parse_diagnostics(
            "main.ssrg",
            r#"pub let message: String = "bad\qescape"
"#,
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        let diagnostic = &diagnostics.diagnostics[0];
        assert_eq!(diagnostic.code, "SES-P0201");
        assert_eq!(diagnostic.message_key, "literal.invalid-escape");
        assert_eq!(diagnostic.primary, ByteRange { start: 30, end: 32 });
        assert_eq!(
            diagnostic.message(),
            "Literal contains an invalid or unsupported escape sequence"
        );
        assert_eq!(diagnostic.helps().len(), 1);
    }

    #[test]
    fn accepts_every_supported_string_escape() {
        let diagnostics = parse_diagnostics(
            "main.ssrg",
            r#"pub let message: String = "line\nreturn\rtab\tzero\0slash\\quote\"lambda\u{03BB}"
"#,
        );

        assert!(diagnostics.diagnostics.is_empty(), "{diagnostics:?}");
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
    fn reports_a_malformed_lambda_instead_of_dropping_the_body() {
        let source = "pub let broken = \\ -> 42\n";
        let diagnostics = parse_diagnostics("main.ssrg", source);

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-P0001");
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "parser.expected-expression"
        );
        assert_eq!(
            diagnostics.diagnostics[0].primary,
            ByteRange { start: 17, end: 24 }
        );
    }

    #[test]
    fn reports_operator_sections_that_cannot_be_function_values() {
        for operator in [
            "&&", "||", "??", "|>", "$", ":=", "!", "..", "..=", "<", "<=", ">", ">=", ":", "^",
        ] {
            let source = format!("pub let operation = ({operator})\n");
            let diagnostics = parse_diagnostics("main.ssrg", &source);
            let start = source.find(operator).unwrap();

            assert_eq!(diagnostics.diagnostics.len(), 1, "{operator}");
            assert_eq!(diagnostics.diagnostics[0].code, "SES-P0001");
            assert_eq!(
                diagnostics.diagnostics[0].message_key,
                "parser.expected-expression"
            );
            assert_eq!(
                diagnostics.diagnostics[0].primary,
                ByteRange {
                    start,
                    end: start + operator.len(),
                }
            );
        }
    }

    #[test]
    fn reports_and_recovers_a_missing_match_arm_expression() {
        let source = "type Label =\n  | Missing\n  | Present String\n\nfn recover value: Label -> String =\n  match value {\n    Missing ->\n    Present item -> item\n    Missing -> \"missing\"\n  }\n";
        let diagnostics = parse_diagnostics("main.ssrg", source);
        let missing_body = source.find("->\n").expect("fixture has a missing arm body") + 2;

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-P0001");
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "parser.expected-expression"
        );
        assert_eq!(
            diagnostics.diagnostics[0].primary,
            ByteRange {
                start: missing_body,
                end: missing_body,
            }
        );

        let surface = parse_surface_ast("main.ssrg", source);
        let SurfaceDecl::Fn {
            body: Some(crate::SurfaceExpr::Match { arms, .. }),
            ..
        } = &surface.declarations[1]
        else {
            panic!("recovery must preserve the surrounding match")
        };
        assert_eq!(arms.len(), 3);
        assert!(matches!(arms[0].body, crate::SurfaceExpr::Error { .. }));
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
    fn reports_integer_literal_outside_safe_integer_range() {
        let diagnostics =
            parse_diagnostics("main.ssrg", "pub let tooLarge: Int = 9007199254740992\n");

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-P0203");
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "literal.int-outside-range"
        );
        assert_eq!(
            diagnostics.diagnostics[0].message(),
            "Integer literal is outside the Int safe range"
        );
        assert_eq!(
            diagnostics.diagnostics[0].helps(),
            vec![
                "Use a value from -9007199254740991 through 9007199254740991, or use BigInt."
                    .to_owned()
            ]
        );
        assert_eq!(
            diagnostics.diagnostics[0].primary,
            ByteRange { start: 24, end: 40 }
        );
    }

    #[test]
    fn rejects_malformed_and_overflowing_float_literals() {
        for source in [
            "pub let value: Float = 1.\n",
            "pub let value: Float = .5\n",
            "pub let value: Float = 1e\n",
            "pub let value: Float = 1e999\n",
        ] {
            let diagnostics = parse_diagnostics("main.ssrg", source);
            assert!(diagnostics
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "SES-P0203"));
        }
    }

    #[test]
    fn accepts_valid_float_literal_spellings() {
        let diagnostics = parse_diagnostics(
            "main.ssrg",
            "let values: Array<Float> = [1.0, 6.022e23, 1e9, 1.0e-9, -0.0, 1_000.25_0]\n",
        );

        assert!(diagnostics.diagnostics.is_empty());
    }

    #[test]
    fn accepts_an_effectful_for_expression() {
        let source = "pub effect fn main = for n <- 1..=3 { println $ `${n}` }\n";
        let diagnostics = parse_diagnostics("main.ssrg", source);

        assert!(diagnostics.diagnostics.is_empty());
    }

    #[test]
    fn accepts_safe_integer_literal_boundaries() {
        let diagnostics = parse_diagnostics(
            "main.ssrg",
            "let maximum: Int = 9007199254740991\nlet minimum: Int = -9007199254740991\n",
        );

        assert!(diagnostics.diagnostics.is_empty());
    }
}
