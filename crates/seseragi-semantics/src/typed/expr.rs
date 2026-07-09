use crate::{
    effect_ops::{known_effect_operation_by_surface, semantic_effect_operation_name},
    unit_type, TypedExpr, TypedParameter, TypedType,
};
use seseragi_syntax::{ByteSpan, Token, TokenKind};

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
) -> Option<TypedExpr> {
    match tokens {
        [left, operator, right, ..] if operator.kind == TokenKind::OperatorArithmetic => {
            let left_expr = typed_fn_body_from_token(left, parameters);
            let right_expr = typed_fn_body_from_token(right, parameters);
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
        [token, ..] => Some(typed_fn_body_from_token(token, parameters)),
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

fn typed_fn_body_from_token(token: &Token, parameters: &[TypedParameter]) -> TypedExpr {
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
    TypedType::Hole
}

fn find_parameter(token: &Token, parameters: &[TypedParameter]) -> Option<(String, TypedType)> {
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
        | TypedExpr::Binary { type_ref, .. }
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

fn typed_do_block(tokens: &[Token], span: ByteSpan, do_token: &Token) -> Option<TypedExpr> {
    let left_brace = tokens
        .iter()
        .skip_while(|token| token.start <= do_token.start)
        .find(|token| token.end <= span.end && token.raw == "{")?;
    let right_brace = tokens
        .iter()
        .skip_while(|token| token.start <= do_token.start)
        .find(|token| token.end <= span.end && token.raw == "}")?;
    let statements = typed_do_statements(tokens, span, left_brace, right_brace);
    Some(TypedExpr::DoBlock {
        statements,
        result: Box::new(TypedExpr::Unit {
            type_ref: unit_type(),
            origin: ByteSpan {
                start: right_brace.start,
                end: right_brace.start,
            },
        }),
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
) -> Vec<TypedExpr> {
    let Some(operation) = tokens
        .iter()
        .skip_while(|token| token.start <= left_brace.start)
        .find(|token| token.end <= right_brace.start && is_significant(token))
    else {
        return Vec::new();
    };

    let Some(effect_operation) = known_effect_operation_by_surface(operation.raw.as_str()) else {
        return Vec::new();
    };

    let argument = tokens
        .iter()
        .skip_while(|token| token.start <= operation.start)
        .find(|token| {
            token.end <= span.end
                && token.start < right_brace.start
                && token.kind == TokenKind::LiteralString
        })
        .map(typed_expr_from_value_token);
    let origin_end = argument
        .as_ref()
        .map(expr_origin_end)
        .unwrap_or(operation.end);

    vec![TypedExpr::EffectCall {
        operation: effect_operation.semantic_name.to_owned(),
        arguments: argument.into_iter().collect(),
        origin: ByteSpan {
            start: operation.start,
            end: origin_end,
        },
    }]
}

fn expr_origin_end(expr: &TypedExpr) -> usize {
    match expr {
        TypedExpr::Unit { origin, .. }
        | TypedExpr::DoBlock { origin, .. }
        | TypedExpr::Integer { origin, .. }
        | TypedExpr::String { origin, .. }
        | TypedExpr::Boolean { origin, .. }
        | TypedExpr::Variable { origin, .. }
        | TypedExpr::Binary { origin, .. }
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
