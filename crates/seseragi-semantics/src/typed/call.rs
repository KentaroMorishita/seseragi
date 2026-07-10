use crate::{TypedExpr, TypedParameter, TypedType};
use seseragi_syntax::{ByteSpan, Token, TokenKind};
use std::collections::BTreeMap;

use super::expr::{find_parameter, typed_fn_body_from_token};
use super::functions::{accepts_saturated_arguments, TopLevelPureFunction};
use super::type_ref::inferred_type_from_expr;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PureCallIssue {
    Arity {
        callee: ByteSpan,
        expected: usize,
        actual: usize,
    },
    ArgumentType {
        argument: ByteSpan,
        index: usize,
        expected: TypedType,
        actual: TypedType,
    },
}

pub(crate) fn typed_top_level_pure_call(
    tokens: &[&Token],
    parameters: &[TypedParameter],
    top_level_values: &BTreeMap<String, TypedType>,
    top_level_functions: &BTreeMap<String, TopLevelPureFunction>,
) -> Option<TypedExpr> {
    let (callee_token, signature) = direct_top_level_pure_call_signature(
        tokens,
        parameters,
        top_level_values,
        top_level_functions,
    )?;
    let argument_tokens = &tokens[1..];
    let arguments = argument_tokens
        .iter()
        .map(|token| typed_fn_body_from_token(token, parameters, top_level_values))
        .collect::<Vec<_>>();
    let argument_types = arguments
        .iter()
        .map(inferred_type_from_expr)
        .collect::<Vec<_>>();
    if !accepts_saturated_arguments(signature, &argument_types) {
        return None;
    }
    let last_argument = argument_tokens.last()?;
    Some(TypedExpr::Call {
        callee: signature.symbol.clone(),
        arguments,
        type_ref: signature.result.clone(),
        origin: ByteSpan {
            start: callee_token.start,
            end: last_argument.end,
        },
    })
}

pub(crate) fn is_supported_top_level_pure_call(
    tokens: &[&Token],
    parameters: &[TypedParameter],
    top_level_values: &BTreeMap<String, TypedType>,
    top_level_functions: &BTreeMap<String, TopLevelPureFunction>,
) -> bool {
    typed_top_level_pure_call(tokens, parameters, top_level_values, top_level_functions).is_some()
}

pub(crate) fn is_known_top_level_pure_call(
    tokens: &[&Token],
    parameters: &[TypedParameter],
    top_level_values: &BTreeMap<String, TypedType>,
    top_level_functions: &BTreeMap<String, TopLevelPureFunction>,
) -> bool {
    direct_top_level_pure_call_signature(tokens, parameters, top_level_values, top_level_functions)
        .is_some()
}

pub(crate) fn top_level_pure_call_issue(
    tokens: &[&Token],
    parameters: &[TypedParameter],
    top_level_values: &BTreeMap<String, TypedType>,
    top_level_functions: &BTreeMap<String, TopLevelPureFunction>,
) -> Option<PureCallIssue> {
    let (callee_token, signature) = direct_top_level_pure_call_signature(
        tokens,
        parameters,
        top_level_values,
        top_level_functions,
    )?;
    let argument_tokens = &tokens[1..];
    if argument_tokens.len() != signature.parameters.len() {
        return Some(PureCallIssue::Arity {
            callee: token_span(callee_token),
            expected: signature.parameters.len(),
            actual: argument_tokens.len(),
        });
    }

    argument_tokens
        .iter()
        .zip(&signature.parameters)
        .enumerate()
        .find_map(|(index, (token, expected))| {
            let actual = inferred_type_from_expr(&typed_fn_body_from_token(
                token,
                parameters,
                top_level_values,
            ));
            (actual != *expected).then(|| PureCallIssue::ArgumentType {
                argument: token_span(token),
                index,
                expected: expected.clone(),
                actual,
            })
        })
}

fn direct_top_level_pure_call_signature<'a>(
    tokens: &[&'a Token],
    parameters: &[TypedParameter],
    top_level_values: &BTreeMap<String, TypedType>,
    top_level_functions: &'a BTreeMap<String, TopLevelPureFunction>,
) -> Option<(&'a Token, &'a TopLevelPureFunction)> {
    let callee_token = *tokens.first()?;
    if callee_token.kind != TokenKind::IdentifierLower
        || find_parameter(callee_token, parameters).is_some()
        || top_level_values.contains_key(&callee_token.raw)
    {
        return None;
    }
    let signature = top_level_functions.get(&callee_token.raw)?;
    Some((callee_token, signature))
}

fn token_span(token: &Token) -> ByteSpan {
    ByteSpan {
        start: token.start,
        end: token.end,
    }
}
