use crate::{TypedExpr, TypedParameter, TypedType};
use seseragi_syntax::{ByteSpan, Token, TokenKind};
use std::collections::BTreeMap;

use super::expr::typed_fn_body_from_tokens;
use super::functions::TopLevelPureFunction;
use super::type_ref::inferred_type_from_expr;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ConditionalIssue {
    ConditionNotBool {
        condition: ByteSpan,
        actual: TypedType,
    },
    BranchTypeMismatch {
        then_branch: ByteSpan,
        else_branch: ByteSpan,
        then_type: TypedType,
        else_type: TypedType,
    },
}

pub(crate) fn typed_conditional(
    tokens: &[&Token],
    parameters: &[TypedParameter],
    top_level_values: &BTreeMap<String, TypedType>,
    top_level_functions: &BTreeMap<String, TopLevelPureFunction>,
) -> Option<TypedExpr> {
    let analysis = analyze_conditional(tokens, parameters, top_level_values, top_level_functions)?;
    if issue_from_analysis(&analysis).is_some() || analysis.branch_type == TypedType::Hole {
        return None;
    }

    Some(TypedExpr::If {
        condition: Box::new(analysis.condition),
        then_branch: Box::new(analysis.then_branch),
        else_branch: Box::new(analysis.else_branch),
        type_ref: analysis.branch_type,
        origin: analysis.origin,
    })
}

pub(crate) fn conditional_issue(
    tokens: &[&Token],
    parameters: &[TypedParameter],
    top_level_values: &BTreeMap<String, TypedType>,
    top_level_functions: &BTreeMap<String, TopLevelPureFunction>,
) -> Option<ConditionalIssue> {
    let analysis = analyze_conditional(tokens, parameters, top_level_values, top_level_functions)?;
    issue_from_analysis(&analysis)
}

struct ConditionalAnalysis {
    condition: TypedExpr,
    condition_span: ByteSpan,
    then_branch: TypedExpr,
    then_span: ByteSpan,
    else_branch: TypedExpr,
    else_span: ByteSpan,
    branch_type: TypedType,
    else_type: TypedType,
    origin: ByteSpan,
}

fn analyze_conditional(
    tokens: &[&Token],
    parameters: &[TypedParameter],
    top_level_values: &BTreeMap<String, TypedType>,
    top_level_functions: &BTreeMap<String, TopLevelPureFunction>,
) -> Option<ConditionalAnalysis> {
    let (condition_tokens, then_tokens, else_tokens) = split_conditional_tokens(tokens)?;
    let condition = typed_fn_body_from_tokens(
        condition_tokens,
        parameters,
        top_level_values,
        top_level_functions,
    )?;
    let then_branch = typed_fn_body_from_tokens(
        then_tokens,
        parameters,
        top_level_values,
        top_level_functions,
    )?;
    let else_branch = typed_fn_body_from_tokens(
        else_tokens,
        parameters,
        top_level_values,
        top_level_functions,
    )?;
    let branch_type = inferred_type_from_expr(&then_branch);
    let else_type = inferred_type_from_expr(&else_branch);

    Some(ConditionalAnalysis {
        condition,
        condition_span: token_slice_span(condition_tokens)?,
        then_branch,
        then_span: token_slice_span(then_tokens)?,
        else_branch,
        else_span: token_slice_span(else_tokens)?,
        branch_type,
        else_type,
        origin: ByteSpan {
            start: tokens.first()?.start,
            end: tokens.last()?.end,
        },
    })
}

fn issue_from_analysis(analysis: &ConditionalAnalysis) -> Option<ConditionalIssue> {
    let condition_type = inferred_type_from_expr(&analysis.condition);
    if !named_type_is(&condition_type, "Bool") {
        return Some(ConditionalIssue::ConditionNotBool {
            condition: analysis.condition_span,
            actual: condition_type,
        });
    }
    if analysis.branch_type != analysis.else_type {
        return Some(ConditionalIssue::BranchTypeMismatch {
            then_branch: analysis.then_span,
            else_branch: analysis.else_span,
            then_type: analysis.branch_type.clone(),
            else_type: analysis.else_type.clone(),
        });
    }
    None
}

fn split_conditional_tokens<'a>(
    tokens: &'a [&'a Token],
) -> Option<(&'a [&'a Token], &'a [&'a Token], &'a [&'a Token])> {
    if tokens.first()?.kind != TokenKind::KeywordIf {
        return None;
    }
    let then_index = tokens
        .iter()
        .position(|token| token.kind == TokenKind::KeywordThen)?;
    let mut nested_conditionals = 0usize;
    let else_index =
        tokens
            .iter()
            .enumerate()
            .skip(then_index + 1)
            .find_map(|(index, token)| match token.kind {
                TokenKind::KeywordIf => {
                    nested_conditionals += 1;
                    None
                }
                TokenKind::KeywordElse if nested_conditionals == 0 => Some(index),
                TokenKind::KeywordElse => {
                    nested_conditionals -= 1;
                    None
                }
                _ => None,
            })?;
    let condition = &tokens[1..then_index];
    let then_branch = &tokens[then_index + 1..else_index];
    let else_branch = &tokens[else_index + 1..];
    (!condition.is_empty() && !then_branch.is_empty() && !else_branch.is_empty()).then_some((
        condition,
        then_branch,
        else_branch,
    ))
}

fn token_slice_span(tokens: &[&Token]) -> Option<ByteSpan> {
    Some(ByteSpan {
        start: tokens.first()?.start,
        end: tokens.last()?.end,
    })
}

fn named_type_is(type_ref: &TypedType, expected_name: &str) -> bool {
    matches!(type_ref, TypedType::Named { name, .. } if name == expected_name)
}
