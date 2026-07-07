use crate::{unit_type, TypedExpr, TypedType};
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
        operation: match operation.raw.as_str() {
            "println" => "std/prelude::println".to_owned(),
            other => other.to_owned(),
        },
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
    let right_brace = tokens
        .iter()
        .skip_while(|token| token.start <= do_token.start)
        .find(|token| token.end <= span.end && token.raw == "}")?;
    Some(TypedExpr::DoBlock {
        statements: Vec::new(),
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

fn expr_origin_end(expr: &TypedExpr) -> usize {
    match expr {
        TypedExpr::Unit { origin, .. }
        | TypedExpr::DoBlock { origin, .. }
        | TypedExpr::Integer { origin, .. }
        | TypedExpr::String { origin, .. }
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
