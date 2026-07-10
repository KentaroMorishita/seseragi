use crate::{TypedExpr, TypedParameter, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceParameter, Token, TokenKind, TypeRef};
use std::collections::{BTreeMap, BTreeSet};

use super::call::{top_level_pure_call_issue, PureCallIssue};
use super::conditional::{conditional_issue, ConditionalIssue};
use super::expr::{find_parameter, find_value_tokens, typed_fn_body_from_tokens};
use super::function_body::{function_body_issue, FunctionBodyIssue};
use super::functions::{typed_parameters_from_surface, TopLevelPureFunction};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UnresolvedNameIssue {
    pub(crate) origin: ByteSpan,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PureFunctionAnalysis {
    pub(crate) conditional_issue: Option<ConditionalIssue>,
    pub(crate) function_body_issue: Option<FunctionBodyIssue>,
    pub(crate) pure_call_issue: Option<PureCallIssue>,
    pub(crate) unresolved_names: Vec<UnresolvedNameIssue>,
}

pub(crate) fn analyze_pure_function(
    tokens: &[Token],
    span: ByteSpan,
    parameters: &[SurfaceParameter],
    return_type: &TypeRef,
    declared_values: &BTreeSet<String>,
    top_level_values: &BTreeMap<String, TypedType>,
    top_level_functions: &BTreeMap<String, TopLevelPureFunction>,
) -> PureFunctionAnalysis {
    let value_tokens = find_value_tokens(tokens, span);
    let typed_parameters = typed_parameters_from_surface(parameters);
    let body = typed_fn_body_from_tokens(
        &value_tokens,
        &typed_parameters,
        top_level_values,
        top_level_functions,
    );
    let conditional_issue = invalid_conditional_issue(
        &value_tokens,
        body.as_ref(),
        &typed_parameters,
        top_level_values,
        top_level_functions,
    );
    let known_call = known_call_signature(
        &value_tokens,
        &typed_parameters,
        top_level_values,
        top_level_functions,
    );
    let pure_call_issue = known_call.and_then(|signature| {
        if body_matches_known_call(body.as_ref(), signature) {
            None
        } else {
            top_level_pure_call_issue(
                &value_tokens,
                &typed_parameters,
                top_level_values,
                top_level_functions,
            )
        }
    });
    let body_span = token_slice_span(&value_tokens);
    let function_body_issue = (conditional_issue.is_none() && pure_call_issue.is_none())
        .then(|| function_body_issue(body.as_ref(), body_span, return_type))
        .flatten();
    let unresolved_names = unresolved_names(
        &value_tokens,
        parameters,
        declared_values,
        known_call.is_some(),
    );

    PureFunctionAnalysis {
        conditional_issue,
        function_body_issue,
        pure_call_issue,
        unresolved_names,
    }
}

fn invalid_conditional_issue(
    tokens: &[&Token],
    body: Option<&TypedExpr>,
    parameters: &[TypedParameter],
    top_level_values: &BTreeMap<String, TypedType>,
    top_level_functions: &BTreeMap<String, TopLevelPureFunction>,
) -> Option<ConditionalIssue> {
    if tokens.first()?.kind != TokenKind::KeywordIf || matches!(body, Some(TypedExpr::If { .. })) {
        return None;
    }
    conditional_issue(tokens, parameters, top_level_values, top_level_functions)
}

fn known_call_signature<'a>(
    tokens: &[&Token],
    parameters: &[TypedParameter],
    top_level_values: &BTreeMap<String, TypedType>,
    top_level_functions: &'a BTreeMap<String, TopLevelPureFunction>,
) -> Option<&'a TopLevelPureFunction> {
    let callee = *tokens.first()?;
    if callee.kind != TokenKind::IdentifierLower
        || find_parameter(callee, parameters).is_some()
        || top_level_values.contains_key(&callee.raw)
    {
        return None;
    }
    top_level_functions.get(&callee.raw)
}

fn body_matches_known_call(body: Option<&TypedExpr>, signature: &TopLevelPureFunction) -> bool {
    matches!(
        body,
        Some(TypedExpr::Call { callee, .. })
            | Some(TypedExpr::Variable { name: callee, .. })
            if callee == &signature.symbol
    )
}

fn unresolved_names(
    tokens: &[&Token],
    parameters: &[SurfaceParameter],
    declared_values: &BTreeSet<String>,
    first_is_known_call: bool,
) -> Vec<UnresolvedNameIssue> {
    let parameter_names = parameters
        .iter()
        .map(|parameter| parameter.name.as_str())
        .collect::<BTreeSet<_>>();
    tokens
        .iter()
        .filter(|token| token.kind == TokenKind::IdentifierLower)
        .enumerate()
        .filter(|(index, token)| {
            !(parameter_names.contains(token.raw.as_str())
                || declared_values.contains(&token.raw)
                || *index == 0 && first_is_known_call)
        })
        .map(|(_, token)| UnresolvedNameIssue {
            origin: ByteSpan {
                start: token.start,
                end: token.end,
            },
        })
        .collect()
}

fn token_slice_span(tokens: &[&Token]) -> Option<ByteSpan> {
    Some(ByteSpan {
        start: tokens.first()?.start,
        end: tokens.last()?.end,
    })
}
