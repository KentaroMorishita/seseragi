use crate::typed::semantic_types::SemanticValueType;
use crate::TypedPattern;
use seseragi_syntax::{decode_string_literal, ByteSpan};
use std::collections::BTreeMap;

use super::{invalid, CoveragePattern, PatternAnalysis};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::typed::surface_expr) enum LiteralPattern {
    Integer(String),
    String(String),
    Char(String),
    Boolean(bool),
}

pub(super) fn type_integer_pattern(
    raw: &str,
    span: ByteSpan,
    expected: &SemanticValueType,
) -> PatternAnalysis {
    let unsigned = raw.strip_prefix('-').unwrap_or(raw);
    let Ok(value) = seseragi_syntax::decode_integer_literal(unsigned) else {
        return invalid(span, "Integer pattern requires a valid Int literal");
    };
    let value = if raw.starts_with('-') { -value } else { value }.to_string();
    type_literal_pattern(
        TypedPattern::Integer {
            value: value.clone(),
            type_ref: expected.type_ref.clone(),
            origin: span,
        },
        LiteralPattern::Integer(value),
        "Int",
        span,
        expected,
    )
}

pub(super) fn type_string_pattern(
    raw: &str,
    span: ByteSpan,
    expected: &SemanticValueType,
) -> PatternAnalysis {
    let value = unquote_string(raw);
    type_literal_pattern(
        TypedPattern::String {
            value: value.clone(),
            type_ref: expected.type_ref.clone(),
            origin: span,
        },
        LiteralPattern::String(value),
        "String",
        span,
        expected,
    )
}

pub(super) fn type_char_pattern(
    raw: &str,
    span: ByteSpan,
    expected: &SemanticValueType,
) -> PatternAnalysis {
    if !matches!(
        expected.key,
        crate::typed::semantic_types::SemanticTypeKey::Other
    ) {
        return invalid(span, "Char literal pattern requires the builtin Char type");
    }
    let value = seseragi_syntax::decode_char_literal(raw).unwrap_or_default();
    type_literal_pattern(
        TypedPattern::Char {
            value: value.clone(),
            type_ref: expected.type_ref.clone(),
            origin: span,
        },
        LiteralPattern::Char(value),
        "Char",
        span,
        expected,
    )
}

pub(super) fn type_boolean_pattern(
    value: bool,
    span: ByteSpan,
    expected: &SemanticValueType,
) -> PatternAnalysis {
    type_literal_pattern(
        TypedPattern::Boolean {
            value,
            type_ref: expected.type_ref.clone(),
            origin: span,
        },
        LiteralPattern::Boolean(value),
        "Bool",
        span,
        expected,
    )
}

fn type_literal_pattern(
    typed: TypedPattern,
    literal: LiteralPattern,
    expected_name: &str,
    span: ByteSpan,
    expected: &SemanticValueType,
) -> PatternAnalysis {
    if expected.key != crate::typed::semantic_types::SemanticTypeKey::Other
        || !super::super::named_type_is(&expected.type_ref, expected_name)
    {
        return invalid(
            span,
            format!("{expected_name} literal pattern does not match the scrutinee type"),
        );
    }
    PatternAnalysis {
        typed,
        coverage: CoveragePattern::Literal(literal),
        locals: BTreeMap::new(),
        issues: Vec::new(),
        invalid: false,
    }
}

fn unquote_string(raw: &str) -> String {
    decode_string_literal(raw).unwrap_or_else(|_| raw.to_owned())
}
