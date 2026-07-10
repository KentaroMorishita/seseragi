use crate::typed::{
    find_value_tokens, is_known_top_level_pure_call, is_supported_top_level_pure_call,
    top_level_pure_call_issue, typed_parameters_from_surface, PureCallIssue, TopLevelPureFunction,
};
use seseragi_syntax::{
    ByteRange, Diagnostic, DiagnosticSeverity, RelatedDiagnostic, SurfaceParameter, Token,
    TokenKind,
};
use std::collections::{BTreeMap, BTreeSet};

use super::type_labels::type_label;

pub(super) fn collect_pure_function_diagnostics(
    tokens: &[Token],
    span: seseragi_syntax::ByteSpan,
    parameters: &[SurfaceParameter],
    declared_values: &BTreeSet<String>,
    top_level_values: &BTreeMap<String, crate::TypedType>,
    top_level_functions: &BTreeMap<String, TopLevelPureFunction>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(equals_index) = tokens
        .iter()
        .position(|token| token.start >= span.start && token.end <= span.end && token.raw == "=")
    else {
        return;
    };
    let parameter_names = parameters
        .iter()
        .map(|parameter| parameter.name.as_str())
        .collect::<BTreeSet<_>>();
    let body_tokens = tokens[equals_index + 1..]
        .iter()
        .take_while(|token| token.end <= span.end)
        .filter(|token| token.kind == TokenKind::IdentifierLower)
        .collect::<Vec<_>>();
    let typed_parameters = typed_parameters_from_surface(parameters);
    let value_tokens = find_value_tokens(tokens, span);
    let has_supported_call = is_supported_top_level_pure_call(
        &value_tokens,
        &typed_parameters,
        top_level_values,
        top_level_functions,
    );
    let has_known_call = is_known_top_level_pure_call(
        &value_tokens,
        &typed_parameters,
        top_level_values,
        top_level_functions,
    );
    if let Some(issue) = top_level_pure_call_issue(
        &value_tokens,
        &typed_parameters,
        top_level_values,
        top_level_functions,
    ) {
        diagnostics.push(pure_call_diagnostic(issue, span));
    }
    for (index, token) in body_tokens.into_iter().enumerate() {
        if index == 0 && (has_supported_call || has_known_call) {
            continue;
        }
        if parameter_names.contains(token.raw.as_str()) || declared_values.contains(&token.raw) {
            continue;
        }
        diagnostics.push(Diagnostic {
            id: String::new(),
            code: "SES-T0101".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "name.unresolved".to_owned(),
            primary: ByteRange {
                start: token.start,
                end: token.end,
            },
            related: vec![RelatedDiagnostic {
                message: "pure function body".to_owned(),
                primary: ByteRange {
                    start: span.start,
                    end: span.end,
                },
            }],
            fixes: Vec::new(),
        });
    }
}

fn pure_call_diagnostic(
    issue: PureCallIssue,
    function_span: seseragi_syntax::ByteSpan,
) -> Diagnostic {
    let (message_key, primary, related_message) = match issue {
        PureCallIssue::Arity {
            callee,
            expected,
            actual,
        } => (
            "call.arity-mismatch",
            callee,
            format!(
                "expected {} {}, received {actual}",
                expected,
                argument_word(expected)
            ),
        ),
        PureCallIssue::ArgumentType {
            argument,
            index,
            expected,
            actual,
        } => (
            "call.argument-type-mismatch",
            argument,
            format!(
                "argument {} expected {}, received {}",
                index + 1,
                type_label(&expected),
                type_label(&actual)
            ),
        ),
    };
    Diagnostic {
        id: String::new(),
        code: "SES-T0101".to_owned(),
        severity: DiagnosticSeverity::Error,
        message_key: message_key.to_owned(),
        primary: ByteRange {
            start: primary.start,
            end: primary.end,
        },
        related: vec![RelatedDiagnostic {
            message: related_message,
            primary: ByteRange {
                start: function_span.start,
                end: function_span.end,
            },
        }],
        fixes: Vec::new(),
    }
}

fn argument_word(count: usize) -> &'static str {
    if count == 1 {
        "argument"
    } else {
        "arguments"
    }
}
