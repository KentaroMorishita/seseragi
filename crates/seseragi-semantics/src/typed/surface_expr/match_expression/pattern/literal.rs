use crate::typed::semantic_types::SemanticValueType;
use crate::TypedPattern;
use seseragi_syntax::ByteSpan;
use std::collections::BTreeMap;

use super::{invalid, CoveragePattern, PatternAnalysis};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::typed::surface_expr::match_expression) enum LiteralPattern {
    Integer(String),
    String(String),
    Boolean(bool),
}

pub(super) fn type_integer_pattern(
    raw: &str,
    span: ByteSpan,
    expected: &SemanticValueType,
) -> PatternAnalysis {
    type_literal_pattern(
        TypedPattern::Integer {
            value: raw.to_owned(),
            type_ref: expected.type_ref.clone(),
            origin: span,
        },
        LiteralPattern::Integer(raw.to_owned()),
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
    if !super::super::super::named_type_is(&expected.type_ref, expected_name) {
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
    raw.strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(raw)
        .to_owned()
}
