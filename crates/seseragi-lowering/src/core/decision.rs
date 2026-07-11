use serde::{Deserialize, Serialize};
use seseragi_semantics::{TypedExpr, TypedMatchArm, TypedPattern, TypedType};
use seseragi_syntax::ByteSpan;

use crate::{source_span, SourceSpan};

use super::expr::lower_expr;
use super::types::lower_typed_type;
use super::{CoreExpr, CoreType};

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
    Constructor {
        path: Vec<CoreDecisionProjection>,
        constructor: String,
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
        TypedPattern::Invalid { origin } => tests.push(CoreDecisionTest::Invalid {
            origin: source_span(source, origin),
        }),
    }
}

fn typed_expr_type(expression: &TypedExpr) -> TypedType {
    match expression {
        TypedExpr::Unit { type_ref, .. }
        | TypedExpr::Integer { type_ref, .. }
        | TypedExpr::String { type_ref, .. }
        | TypedExpr::Boolean { type_ref, .. }
        | TypedExpr::Variable { type_ref, .. }
        | TypedExpr::Call { type_ref, .. }
        | TypedExpr::Tuple { type_ref, .. }
        | TypedExpr::Binary { type_ref, .. }
        | TypedExpr::If { type_ref, .. }
        | TypedExpr::Match { type_ref, .. } => type_ref.clone(),
        TypedExpr::DoBlock { result, .. } => typed_expr_type(result),
        TypedExpr::EffectCall { effect, .. } => effect.success.clone(),
    }
}

#[cfg(test)]
mod tests;
