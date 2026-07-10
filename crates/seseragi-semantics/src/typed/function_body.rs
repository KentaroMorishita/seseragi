use crate::{TypedExpr, TypedParameter, TypedType};
use seseragi_syntax::{ByteSpan, Token, TokenKind, TypeRef};
use std::collections::BTreeMap;

use super::expr::{find_value_tokens, typed_fn_body_from_tokens};
use super::functions::{typed_parameters_from_surface, TopLevelPureFunction};
use super::type_ref::{inferred_type_from_expr, typed_type_from_type_ref};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FunctionBodyIssue {
    pub(crate) body: ByteSpan,
    pub(crate) expected: TypedType,
    pub(crate) actual: TypedType,
}

pub(crate) fn function_body_issue(
    tokens: &[Token],
    span: ByteSpan,
    parameters: &[seseragi_syntax::SurfaceParameter],
    return_type: &TypeRef,
    top_level_values: &BTreeMap<String, TypedType>,
    top_level_functions: &BTreeMap<String, TopLevelPureFunction>,
) -> Option<FunctionBodyIssue> {
    let value_tokens = find_value_tokens(tokens, span);
    let typed_parameters: Vec<TypedParameter> = typed_parameters_from_surface(parameters);
    let body = typed_fn_body_from_tokens(
        &value_tokens,
        &typed_parameters,
        top_level_values,
        top_level_functions,
    )?;
    if value_tokens.first()?.kind == TokenKind::KeywordIf && !matches!(body, TypedExpr::If { .. }) {
        return None;
    }
    let expected = typed_type_from_type_ref(return_type);
    let actual = inferred_type_from_expr(&body);
    if expected == TypedType::Hole || actual == TypedType::Hole || expected == actual {
        return None;
    }

    Some(FunctionBodyIssue {
        body: ByteSpan {
            start: value_tokens.first()?.start,
            end: value_tokens.last()?.end,
        },
        expected,
        actual,
    })
}
