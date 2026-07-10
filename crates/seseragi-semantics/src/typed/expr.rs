use crate::{
    effect_ops::{known_effect_operation_by_surface, semantic_effect_operation_name},
    unit_type, TypedDoStatement, TypedExpr, TypedType,
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

pub(crate) fn lower_first(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().chain(chars).collect(),
        None => String::new(),
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
        | TypedExpr::Tuple { origin, .. }
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
