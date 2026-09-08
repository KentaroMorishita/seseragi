use serde::{Deserialize, Serialize};
use seseragi_semantics::{TypedExpr, TypedMatchArm, TypedPattern, TypedType};
use seseragi_syntax::ByteSpan;

use crate::{source_span, SourceSpan};

use super::expr::lower_expr;
use super::types::lower_typed_type;
use super::{CoreExpr, CoreType};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CorePatternBindingPlan {
    pub(super) input_type: CoreType,
    pub(super) tests: Vec<CoreDecisionTest>,
    pub(super) bindings: Vec<CoreDecisionBinding>,
    pub(super) origin: SourceSpan,
}

pub(super) fn lower_pattern_binding_plan(
    source: &str,
    pattern: TypedPattern,
) -> CorePatternBindingPlan {
    let input_type = lower_typed_type(typed_pattern_type(&pattern));
    let origin = source_span(source, typed_pattern_origin(&pattern));
    let mut tests = Vec::new();
    let mut bindings = Vec::new();
    lower_pattern(source, pattern, &mut Vec::new(), &mut tests, &mut bindings);
    CorePatternBindingPlan {
        input_type,
        tests,
        bindings,
        origin,
    }
}

pub(super) fn projection_expression(
    temporary: &str,
    plan: &CorePatternBindingPlan,
    binding: &CoreDecisionBinding,
) -> CoreExpr {
    CoreExpr::Decision {
        scrutinee: Box::new(CoreExpr::Variable {
            name: temporary.to_owned(),
            evidence: Vec::new(),
            type_ref: plan.input_type.clone(),
            origin: plan.origin.clone(),
        }),
        scrutinee_type: plan.input_type.clone(),
        branches: vec![CoreDecisionBranch {
            tests: plan.tests.clone(),
            bindings: plan.bindings.clone(),
            guard: None,
            value: CoreExpr::Variable {
                name: binding.name.clone(),
                evidence: Vec::new(),
                type_ref: binding.type_ref.clone(),
                origin: binding.origin.clone(),
            },
            origin: plan.origin.clone(),
        }],
        exhaustive: true,
        type_ref: binding.type_ref.clone(),
        origin: plan.origin.clone(),
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreDecisionBranch {
    pub tests: Vec<CoreDecisionTest>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bindings: Vec<CoreDecisionBinding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guard: Option<CoreExpr>,
    pub value: CoreExpr,
    pub origin: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreDecisionBinding {
    pub name: String,
    #[serde(rename = "type")]
    pub type_ref: CoreType,
    pub path: Vec<CoreDecisionProjection>,
    pub origin: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum CoreDecisionTest {
    Integer {
        path: Vec<CoreDecisionProjection>,
        value: String,
        origin: SourceSpan,
    },
    String {
        path: Vec<CoreDecisionProjection>,
        value: String,
        origin: SourceSpan,
    },
    Boolean {
        path: Vec<CoreDecisionProjection>,
        value: bool,
        origin: SourceSpan,
    },
    Constructor {
        path: Vec<CoreDecisionProjection>,
        constructor: String,
        origin: SourceSpan,
    },
    ArrayLength {
        path: Vec<CoreDecisionProjection>,
        length: usize,
        minimum: bool,
        origin: SourceSpan,
    },
    ListLength {
        path: Vec<CoreDecisionProjection>,
        length: usize,
        minimum: bool,
        origin: SourceSpan,
    },
    Invalid {
        origin: SourceSpan,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum CoreDecisionProjection {
    TupleElement { index: usize },
    ArrayElement { index: usize },
    ArrayRest { start: usize },
    ListElement { index: usize },
    ListRest { start: usize },
    RecordField { name: String },
    AdtPayload,
}

pub(super) fn lower_match(
    source: &str,
    scrutinee: TypedExpr,
    arms: Vec<TypedMatchArm>,
    exhaustive: bool,
    type_ref: TypedType,
    origin: ByteSpan,
) -> CoreExpr {
    let scrutinee_type = lower_typed_type(typed_expr_type(&scrutinee));
    let mut branches = arms
        .into_iter()
        .map(|arm| lower_branch(source, arm))
        .collect::<Vec<_>>();
    normalize_residual_branch(exhaustive, &mut branches);
    CoreExpr::Decision {
        scrutinee: Box::new(lower_expr(source, scrutinee)),
        scrutinee_type,
        branches,
        exhaustive,
        type_ref: lower_typed_type(type_ref),
        origin: source_span(source, origin),
    }
}

fn normalize_residual_branch(exhaustive: bool, branches: &mut [CoreDecisionBranch]) {
    if !exhaustive {
        return;
    }
    let Some(last) = branches.last_mut() else {
        return;
    };
    let binds_adt_payload = last.bindings.iter().any(|binding| {
        binding
            .path
            .iter()
            .any(|projection| projection == &CoreDecisionProjection::AdtPayload)
    });
    if last.guard.is_none()
        && !binds_adt_payload
        && !last
            .tests
            .iter()
            .any(|test| matches!(test, CoreDecisionTest::Invalid { .. }))
    {
        last.tests.clear();
    }
}

fn lower_branch(source: &str, arm: TypedMatchArm) -> CoreDecisionBranch {
    let mut tests = Vec::new();
    let mut bindings = Vec::new();
    lower_pattern(
        source,
        arm.pattern,
        &mut Vec::new(),
        &mut tests,
        &mut bindings,
    );
    CoreDecisionBranch {
        tests,
        bindings,
        guard: arm.guard.map(|guard| lower_expr(source, guard)),
        value: lower_expr(source, arm.body),
        origin: source_span(source, arm.origin),
    }
}

fn lower_pattern(
    source: &str,
    pattern: TypedPattern,
    path: &mut Vec<CoreDecisionProjection>,
    tests: &mut Vec<CoreDecisionTest>,
    bindings: &mut Vec<CoreDecisionBinding>,
) {
    match pattern {
        TypedPattern::Integer { value, origin, .. } => {
            tests.push(CoreDecisionTest::Integer {
                path: path.clone(),
                value,
                origin: source_span(source, origin),
            });
        }
        TypedPattern::String { value, origin, .. } | TypedPattern::Char { value, origin, .. } => {
            tests.push(CoreDecisionTest::String {
                path: path.clone(),
                value,
                origin: source_span(source, origin),
            });
        }
        TypedPattern::Boolean { value, origin, .. } => {
            tests.push(CoreDecisionTest::Boolean {
                path: path.clone(),
                value,
                origin: source_span(source, origin),
            });
        }
        TypedPattern::Wildcard { .. } => {}
        TypedPattern::Binding {
            name,
            type_ref,
            origin,
            ..
        } => bindings.push(CoreDecisionBinding {
            name,
            type_ref: lower_typed_type(type_ref),
            path: path.clone(),
            origin: source_span(source, origin),
        }),
        TypedPattern::Constructor {
            symbol,
            argument,
            origin,
            ..
        } => {
            tests.push(CoreDecisionTest::Constructor {
                path: path.clone(),
                constructor: symbol,
                origin: source_span(source, origin),
            });
            if let Some(argument) = argument {
                path.push(CoreDecisionProjection::AdtPayload);
                lower_pattern(source, *argument, path, tests, bindings);
                path.pop();
            }
        }
        TypedPattern::Tuple { elements, .. } => {
            for (index, element) in elements.into_iter().enumerate() {
                path.push(CoreDecisionProjection::TupleElement { index });
                lower_pattern(source, element, path, tests, bindings);
                path.pop();
            }
        }
        TypedPattern::Array {
            elements,
            rest,
            origin,
            ..
        } => {
            let length = elements.len();
            tests.push(CoreDecisionTest::ArrayLength {
                path: path.clone(),
                length,
                minimum: rest.is_some(),
                origin: source_span(source, origin),
            });
            for (index, element) in elements.into_iter().enumerate() {
                path.push(CoreDecisionProjection::ArrayElement { index });
                lower_pattern(source, element, path, tests, bindings);
                path.pop();
            }
            if let Some(rest) = rest {
                path.push(CoreDecisionProjection::ArrayRest { start: length });
                lower_pattern(source, *rest, path, tests, bindings);
                path.pop();
            }
        }
        TypedPattern::List {
            elements,
            rest,
            origin,
            ..
        } => {
            let length = elements.len();
            tests.push(CoreDecisionTest::ListLength {
                path: path.clone(),
                length,
                minimum: rest.is_some(),
                origin: source_span(source, origin),
            });
            for (index, element) in elements.into_iter().enumerate() {
                path.push(CoreDecisionProjection::ListElement { index });
                lower_pattern(source, element, path, tests, bindings);
                path.pop();
            }
            if let Some(rest) = rest {
                path.push(CoreDecisionProjection::ListRest { start: length });
                lower_pattern(source, *rest, path, tests, bindings);
                path.pop();
            }
        }
        TypedPattern::Record { fields, .. } => {
            for field in fields {
                path.push(CoreDecisionProjection::RecordField { name: field.name });
                lower_pattern(source, field.pattern, path, tests, bindings);
                path.pop();
            }
        }
        TypedPattern::Invalid { origin } => tests.push(CoreDecisionTest::Invalid {
            origin: source_span(source, origin),
        }),
    }
}

fn typed_pattern_type(pattern: &TypedPattern) -> TypedType {
    match pattern {
        TypedPattern::Integer { type_ref, .. }
        | TypedPattern::String { type_ref, .. }
        | TypedPattern::Char { type_ref, .. }
        | TypedPattern::Boolean { type_ref, .. }
        | TypedPattern::Wildcard { type_ref, .. }
        | TypedPattern::Binding { type_ref, .. }
        | TypedPattern::Constructor { type_ref, .. }
        | TypedPattern::Tuple { type_ref, .. }
        | TypedPattern::Array { type_ref, .. }
        | TypedPattern::List { type_ref, .. }
        | TypedPattern::Record { type_ref, .. } => type_ref.clone(),
        TypedPattern::Invalid { .. } => TypedType::Hole,
    }
}

fn typed_pattern_origin(pattern: &TypedPattern) -> ByteSpan {
    match pattern {
        TypedPattern::Integer { origin, .. }
        | TypedPattern::String { origin, .. }
        | TypedPattern::Char { origin, .. }
        | TypedPattern::Boolean { origin, .. }
        | TypedPattern::Wildcard { origin, .. }
        | TypedPattern::Binding { origin, .. }
        | TypedPattern::Constructor { origin, .. }
        | TypedPattern::Tuple { origin, .. }
        | TypedPattern::Array { origin, .. }
        | TypedPattern::List { origin, .. }
        | TypedPattern::Record { origin, .. }
        | TypedPattern::Invalid { origin } => *origin,
    }
}

fn typed_expr_type(expression: &TypedExpr) -> TypedType {
    match expression {
        TypedExpr::Unit { type_ref, .. }
        | TypedExpr::Integer { type_ref, .. }
        | TypedExpr::Float { type_ref, .. }
        | TypedExpr::String { type_ref, .. }
        | TypedExpr::Char { type_ref, .. }
        | TypedExpr::Template { type_ref, .. }
        | TypedExpr::Boolean { type_ref, .. }
        | TypedExpr::Variable { type_ref, .. }
        | TypedExpr::Call { type_ref, .. }
        | TypedExpr::Lambda { type_ref, .. }
        | TypedExpr::Tuple { type_ref, .. }
        | TypedExpr::FieldAccess { type_ref, .. }
        | TypedExpr::OptionalFieldAccess { type_ref, .. }
        | TypedExpr::Record { type_ref, .. }
        | TypedExpr::Array { type_ref, .. }
        | TypedExpr::List { type_ref, .. }
        | TypedExpr::ArrayComprehension { type_ref, .. }
        | TypedExpr::ListComprehension { type_ref, .. }
        | TypedExpr::Binary { type_ref, .. }
        | TypedExpr::Unary { type_ref, .. }
        | TypedExpr::If { type_ref, .. }
        | TypedExpr::Match { type_ref, .. }
        | TypedExpr::Block { type_ref, .. }
        | TypedExpr::MonadDo { type_ref, .. } => type_ref.clone(),
        TypedExpr::DoBlock { result, .. } => typed_expr_type(result),
        TypedExpr::EffectCall { effect, .. } | TypedExpr::EffectInvoke { effect, .. } => {
            effect.success.clone()
        }
    }
}

#[cfg(test)]
mod tests;

/// Reuse the decision backend so the scrutinee is evaluated once and fallback stays lazy.
pub(super) fn lower_fallback(
    source: &str,
    left: TypedExpr,
    right: TypedExpr,
    result_type: TypedType,
    span: ByteSpan,
) -> CoreExpr {
    let scrutinee_type = lower_typed_type(typed_expr_type(&left));
    let type_ref = lower_typed_type(result_type);
    let origin = source_span(source, span);
    let name = "$ssrg$fallbackValue".to_owned();
    CoreExpr::Decision {
        scrutinee: Box::new(lower_expr(source, left)),
        scrutinee_type,
        exhaustive: true,
        branches: vec![
            CoreDecisionBranch {
                tests: vec![CoreDecisionTest::Constructor {
                    path: vec![],
                    constructor: "std/prelude::Just".to_owned(),
                    origin: origin.clone(),
                }],
                bindings: vec![CoreDecisionBinding {
                    name: name.clone(),
                    type_ref: type_ref.clone(),
                    path: vec![CoreDecisionProjection::AdtPayload],
                    origin: origin.clone(),
                }],
                guard: None,
                value: CoreExpr::Variable {
                    name,
                    evidence: vec![],
                    type_ref: type_ref.clone(),
                    origin: origin.clone(),
                },
                origin: origin.clone(),
            },
            CoreDecisionBranch {
                tests: vec![],
                bindings: vec![],
                guard: None,
                value: lower_expr(source, right),
                origin: origin.clone(),
            },
        ],
        type_ref,
        origin,
    }
}
