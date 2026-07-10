use crate::{
    effect_ops::{known_effect_operation_by_surface, semantic_effect_operation_name},
    unit_type, TypedDoStatement, TypedExpr, TypedParameter, TypedType,
};
use seseragi_syntax::{ByteSpan, Token, TokenKind};
use std::collections::BTreeMap;

use super::call::typed_top_level_pure_call;
use super::conditional::typed_conditional;
use super::functions::TopLevelPureFunction;

pub(crate) fn typed_expr_from_value_token(token: &Token) -> TypedExpr {
    let origin = ByteSpan {
        start: token.start,
        end: token.end,
    };
    match token.kind {
        TokenKind::LiteralInteger => TypedExpr::Integer {
            value: token.raw.clone(),
            type_ref: TypedType::Named {
                name: "Int".to_owned(),
                arguments: Vec::new(),
            },
            origin,
        },
        TokenKind::LiteralString => TypedExpr::String {
            value: unquote_string(&token.raw),
            type_ref: TypedType::Named {
                name: "String".to_owned(),
                arguments: Vec::new(),
            },
            origin,
        },
        TokenKind::LiteralBoolean => TypedExpr::Boolean {
            value: token.raw == "True",
            type_ref: TypedType::Named {
                name: "Bool".to_owned(),
                arguments: Vec::new(),
            },
            origin,
        },
        _ => TypedExpr::EffectCall {
            operation: "std/prelude::unknown".to_owned(),
            arguments: Vec::new(),
            origin,
        },
    }
}

pub(crate) fn find_value_token(tokens: &[Token], span: ByteSpan) -> Option<&Token> {
    let equals_index = tokens
        .iter()
        .position(|token| token.start >= span.start && token.end <= span.end && token.raw == "=")?;
    tokens[equals_index + 1..]
        .iter()
        .find(|token| token.end <= span.end && is_significant(token))
}

pub(crate) fn find_value_tokens(tokens: &[Token], span: ByteSpan) -> Vec<&Token> {
    let Some(equals_index) = tokens
        .iter()
        .position(|token| token.start >= span.start && token.end <= span.end && token.raw == "=")
    else {
        return Vec::new();
    };
    tokens[equals_index + 1..]
        .iter()
        .take_while(|token| token.end <= span.end)
        .filter(|token| is_significant(token))
        .collect()
}

pub(crate) fn find_type_name_after(
    tokens: &[Token],
    span: ByteSpan,
    keyword: TokenKind,
) -> Option<String> {
    let keyword_index = tokens.iter().position(|token| {
        token.start >= span.start && token.end <= span.end && token.kind == keyword
    })?;
    tokens[keyword_index + 1..]
        .iter()
        .find(|token| {
            token.end <= span.end
                && matches!(
                    token.kind,
                    TokenKind::IdentifierLower | TokenKind::IdentifierUpper
                )
        })
        .map(|token| token.raw.clone())
}

pub(crate) fn find_effect_body(tokens: &[Token], span: ByteSpan) -> Option<TypedExpr> {
    let equals_index = tokens
        .iter()
        .position(|token| token.start >= span.start && token.end <= span.end && token.raw == "=")?;
    let operation = tokens[equals_index + 1..]
        .iter()
        .find(|token| token.end <= span.end && is_significant(token))?;

    if operation.kind == TokenKind::KeywordDo {
        return typed_do_block(tokens, span, operation);
    }

    let argument = tokens
        .iter()
        .skip_while(|token| token.start <= operation.start)
        .find(|token| token.end <= span.end && token.kind == TokenKind::LiteralString)
        .map(typed_expr_from_value_token);
    let origin_end = argument
        .as_ref()
        .map(expr_origin_end)
        .unwrap_or(operation.end);

    Some(TypedExpr::EffectCall {
        operation: semantic_effect_operation_name(operation.raw.as_str()),
        arguments: argument.into_iter().collect(),
        origin: ByteSpan {
            start: operation.start,
            end: origin_end,
        },
    })
}

pub(crate) fn typed_fn_body_from_tokens(
    tokens: &[&Token],
    parameters: &[TypedParameter],
    top_level_values: &BTreeMap<String, TypedType>,
    top_level_functions: &BTreeMap<String, TopLevelPureFunction>,
) -> Option<TypedExpr> {
    if let Some(conditional) =
        typed_conditional(tokens, parameters, top_level_values, top_level_functions)
    {
        return Some(conditional);
    }
    if let Some(call) =
        typed_top_level_pure_call(tokens, parameters, top_level_values, top_level_functions)
    {
        return Some(call);
    }

    match tokens {
        [left, operator, right]
            if matches!(
                operator.kind,
                TokenKind::OperatorArithmetic | TokenKind::OperatorComparison
            ) =>
        {
            let left_expr = typed_fn_body_from_token(left, parameters, top_level_values);
            let right_expr = typed_fn_body_from_token(right, parameters, top_level_values);
            let type_ref = binary_result_type(operator.raw.as_str(), &left_expr, &right_expr);
            Some(TypedExpr::Binary {
                operator: operator.raw.clone(),
                left: Box::new(left_expr),
                right: Box::new(right_expr),
                type_ref,
                origin: ByteSpan {
                    start: left.start,
                    end: right.end,
                },
            })
        }
        [token, ..] => Some(typed_fn_body_from_token(
            token,
            parameters,
            top_level_values,
        )),
        [] => None,
    }
}

pub(crate) fn lower_first(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().chain(chars).collect(),
        None => String::new(),
    }
}

pub(crate) fn typed_fn_body_from_token(
    token: &Token,
    parameters: &[TypedParameter],
    top_level_values: &BTreeMap<String, TypedType>,
) -> TypedExpr {
    if token.kind == TokenKind::IdentifierLower {
        if let Some((name, type_ref)) = find_parameter(token, parameters) {
            return TypedExpr::Variable {
                name,
                type_ref,
                origin: ByteSpan {
                    start: token.start,
                    end: token.end,
                },
            };
        }
        if let Some(type_ref) = top_level_values.get(&token.raw) {
            return TypedExpr::Variable {
                name: token.raw.clone(),
                type_ref: type_ref.clone(),
                origin: ByteSpan {
                    start: token.start,
                    end: token.end,
                },
            };
        }
        return TypedExpr::Variable {
            name: token.raw.clone(),
            type_ref: TypedType::Hole,
            origin: ByteSpan {
                start: token.start,
                end: token.end,
            },
        };
    }
    typed_expr_from_value_token(token)
}

fn binary_result_type(operator: &str, left: &TypedExpr, right: &TypedExpr) -> TypedType {
    if matches!(operator, "+" | "-" | "*" | "/" | "%" | "**")
        && expr_has_type(left, "Int")
        && expr_has_type(right, "Int")
    {
        return int_type();
    }
    if matches!(operator, "==" | "!=")
        && ["Int", "Bool", "String"]
            .iter()
            .any(|name| expr_has_type(left, name) && expr_has_type(right, name))
    {
        return bool_type();
    }
    if matches!(operator, "<" | "<=" | ">" | ">=")
        && expr_has_type(left, "Int")
        && expr_has_type(right, "Int")
    {
        return bool_type();
    }
    TypedType::Hole
}

pub(crate) fn find_parameter(
    token: &Token,
    parameters: &[TypedParameter],
) -> Option<(String, TypedType)> {
    parameters.iter().find_map(|parameter| match parameter {
        TypedParameter::Named { name, type_ref, .. } if name == &token.raw => {
            Some((name.clone(), type_ref.clone()))
        }
        _ => None,
    })
}

fn expr_has_type(expr: &TypedExpr, expected_name: &str) -> bool {
    match expr {
        TypedExpr::Integer { type_ref, .. }
        | TypedExpr::String { type_ref, .. }
        | TypedExpr::Boolean { type_ref, .. }
        | TypedExpr::Variable { type_ref, .. }
        | TypedExpr::Call { type_ref, .. }
        | TypedExpr::Binary { type_ref, .. }
        | TypedExpr::If { type_ref, .. }
        | TypedExpr::Unit { type_ref, .. } => named_type_is(type_ref, expected_name),
        TypedExpr::EffectCall { .. } | TypedExpr::DoBlock { .. } => false,
    }
}

fn named_type_is(type_ref: &TypedType, expected_name: &str) -> bool {
    matches!(type_ref, TypedType::Named { name, .. } if name == expected_name)
}

fn int_type() -> TypedType {
    TypedType::Named {
        name: "Int".to_owned(),
        arguments: Vec::new(),
    }
}

fn bool_type() -> TypedType {
    TypedType::Named {
        name: "Bool".to_owned(),
        arguments: Vec::new(),
    }
}

fn typed_do_block(tokens: &[Token], span: ByteSpan, do_token: &Token) -> Option<TypedExpr> {
    let left_brace = tokens
        .iter()
        .skip_while(|token| token.start <= do_token.start)
        .find(|token| token.end <= span.end && token.raw == "{")?;
    let right_brace = tokens
        .iter()
        .skip_while(|token| token.start <= do_token.start)
        .find(|token| token.end <= span.end && token.raw == "}")?;
    let mut statements = typed_do_statements(tokens, span, left_brace, right_brace);
    let result = match statements.pop() {
        Some(TypedDoStatement::Effect { value }) => value,
        Some(statement @ TypedDoStatement::Bind { .. }) => {
            statements.push(statement);
            TypedExpr::Unit {
                type_ref: unit_type(),
                origin: ByteSpan {
                    start: right_brace.start,
                    end: right_brace.start,
                },
            }
        }
        None => TypedExpr::Unit {
            type_ref: unit_type(),
            origin: ByteSpan {
                start: right_brace.start,
                end: right_brace.start,
            },
        },
    };
    Some(TypedExpr::DoBlock {
        statements,
        result: Box::new(result),
        origin: ByteSpan {
            start: do_token.start,
            end: right_brace.end,
        },
    })
}

fn typed_do_statements(
    tokens: &[Token],
    span: ByteSpan,
    left_brace: &Token,
    right_brace: &Token,
) -> Vec<TypedDoStatement> {
    let mut statements = Vec::new();
    let left_index = tokens
        .iter()
        .position(|token| token.start == left_brace.start && token.end == left_brace.end)
        .unwrap_or(0);
    let right_index = tokens
        .iter()
        .position(|token| token.start == right_brace.start && token.end == right_brace.end)
        .unwrap_or(tokens.len());
    let mut line_start = left_index + 1;
    for (index, token) in tokens.iter().enumerate() {
        if index <= left_index || index >= right_index {
            continue;
        }
        if token.kind != TokenKind::TriviaNewline {
            continue;
        }
        statements.extend(typed_do_line(tokens, span, line_start, index));
        line_start = index + 1;
    }
    statements.extend(typed_do_line(tokens, span, line_start, right_index));
    statements
}

fn typed_do_line(
    tokens: &[Token],
    span: ByteSpan,
    start: usize,
    end: usize,
) -> Vec<TypedDoStatement> {
    let significant = tokens[start..end]
        .iter()
        .filter(|token| is_significant(token))
        .collect::<Vec<_>>();
    let Some(first) = significant.first().copied() else {
        return Vec::new();
    };
    if let Some(bind_index) = significant
        .iter()
        .position(|token| token.kind == TokenKind::OperatorBind)
    {
        let Some(name) = significant[..bind_index]
            .iter()
            .find(|token| token.kind == TokenKind::IdentifierLower)
        else {
            return Vec::new();
        };
        let Some(operation) = significant[bind_index + 1..]
            .iter()
            .find(|token| known_effect_operation_by_surface(token.raw.as_str()).is_some())
        else {
            return Vec::new();
        };
        let known = known_effect_operation_by_surface(operation.raw.as_str()).expect("known above");
        let value = typed_effect_call(tokens, span, operation, end);
        let origin_end = expr_origin_end(&value);
        return vec![TypedDoStatement::Bind {
            name: name.raw.clone(),
            type_ref: TypedType::Named {
                name: known.success_type.to_owned(),
                arguments: known
                    .success_type_arguments
                    .iter()
                    .map(|name| TypedType::Named {
                        name: (*name).to_owned(),
                        arguments: Vec::new(),
                    })
                    .collect(),
            },
            value,
            origin: ByteSpan {
                start: first.start,
                end: origin_end,
            },
        }];
    }

    significant
        .into_iter()
        .filter(|token| known_effect_operation_by_surface(token.raw.as_str()).is_some())
        .map(|operation| TypedDoStatement::Effect {
            value: typed_effect_call(tokens, span, operation, end),
        })
        .collect()
}

fn typed_effect_call(tokens: &[Token], span: ByteSpan, operation: &Token, end: usize) -> TypedExpr {
    let effect_operation =
        known_effect_operation_by_surface(operation.raw.as_str()).expect("known effect operation");
    let argument = tokens
        .iter()
        .skip_while(|token| token.start <= operation.start)
        .take_while(|token| {
            token.end <= span.end
                && token.start
                    < tokens
                        .get(end)
                        .map(|token| token.start)
                        .unwrap_or(usize::MAX)
                && known_effect_operation_by_surface(token.raw.as_str()).is_none()
        })
        .find(|token| token.kind == TokenKind::LiteralString)
        .map(typed_expr_from_value_token);
    let origin_end = argument
        .as_ref()
        .map(expr_origin_end)
        .unwrap_or(operation.end);
    TypedExpr::EffectCall {
        operation: effect_operation.semantic_name.to_owned(),
        arguments: argument.into_iter().collect(),
        origin: ByteSpan {
            start: operation.start,
            end: origin_end,
        },
    }
}

fn expr_origin_end(expr: &TypedExpr) -> usize {
    match expr {
        TypedExpr::Unit { origin, .. }
        | TypedExpr::DoBlock { origin, .. }
        | TypedExpr::Integer { origin, .. }
        | TypedExpr::String { origin, .. }
        | TypedExpr::Boolean { origin, .. }
        | TypedExpr::Variable { origin, .. }
        | TypedExpr::Call { origin, .. }
        | TypedExpr::Binary { origin, .. }
        | TypedExpr::If { origin, .. }
        | TypedExpr::EffectCall { origin, .. } => origin.end,
    }
}

fn is_significant(token: &Token) -> bool {
    !matches!(
        token.kind,
        TokenKind::TriviaComment
            | TokenKind::TriviaNewline
            | TokenKind::TriviaSpace
            | TokenKind::Eof
    )
}

fn unquote_string(value: &str) -> String {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value)
        .to_owned()
}
