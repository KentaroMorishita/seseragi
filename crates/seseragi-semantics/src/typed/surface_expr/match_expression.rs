use crate::typed::pure_issues::MatchIssue;
use crate::typed::semantic_types::{SemanticTypeKey, SemanticValueType};
use crate::typed::type_ref::{inferred_type_from_expr, typed_type_contains_hole};
use crate::{TypedExpr, TypedMatchArm, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceExpr, SurfaceMatchArm};

use super::{
    named_type_is, type_surface_expression, PureExpressionContext, SurfaceExpressionAnalysis,
};

mod coverage;
mod pattern;

use coverage::analyze_coverage;
use pattern::type_pattern;

pub(super) fn type_match(
    scrutinee: &SurfaceExpr,
    arms: &[SurfaceMatchArm],
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    let scrutinee = type_surface_expression(scrutinee, context);
    let expected = SemanticValueType {
        type_ref: inferred_type_from_expr(&scrutinee.value),
        key: scrutinee.semantic_type.clone(),
    };
    let mut result_type: Option<(TypedType, ByteSpan)> = None;
    let mut result_key: Option<SemanticTypeKey> = None;
    let mut typed_arms = Vec::with_capacity(arms.len());
    let mut coverage_arms = Vec::with_capacity(arms.len());
    let mut issues = Vec::new();
    let mut invalid_pattern = false;
    let mut coverage_proves_total = false;
    let mut child_analyses = Vec::new();

    for arm in arms {
        let pattern = type_pattern(&arm.pattern, &expected, context);
        invalid_pattern |= pattern.invalid;
        issues.extend(pattern.issues);
        coverage_arms.push((pattern.coverage, arm.guard.is_some()));
        let arm_context = context.with_locals(pattern.locals);

        let guard = arm
            .guard
            .as_ref()
            .map(|guard| type_surface_expression(guard, &arm_context));
        if let Some(guard) = &guard {
            let guard_type = inferred_type_from_expr(&guard.value);
            if !typed_type_contains_hole(&guard_type) && !named_type_is(&guard_type, "Bool") {
                issues.push(MatchIssue::GuardNotBool {
                    guard: arm.guard.as_ref().expect("guard analysis exists").span(),
                    actual: guard_type,
                });
            }
        }

        let body = type_surface_expression(&arm.body, &arm_context);
        let body_type = inferred_type_from_expr(&body.value);
        if !typed_type_contains_hole(&body_type) {
            if let Some((expected_type, expected_span)) = &result_type {
                if body_type != *expected_type {
                    issues.push(MatchIssue::BranchTypeMismatch {
                        expected_branch: *expected_span,
                        actual_branch: arm.body.span(),
                        expected: expected_type.clone(),
                        actual: body_type.clone(),
                    });
                }
            } else {
                result_type = Some((body_type.clone(), arm.body.span()));
            }
        }
        if body.semantic_type != SemanticTypeKey::Invalid {
            match &result_key {
                Some(expected_key) if *expected_key != body.semantic_type => {
                    result_key = Some(SemanticTypeKey::Invalid);
                }
                None => result_key = Some(body.semantic_type.clone()),
                _ => {}
            }
        }

        let typed_guard = guard.as_ref().map(|guard| guard.value.clone());
        typed_arms.push(TypedMatchArm {
            pattern: pattern.typed,
            guard: typed_guard,
            body: body.value.clone(),
            origin: arm.span,
        });
        if let Some(guard) = guard {
            child_analyses.push(guard);
        }
        child_analyses.push(body);
    }

    let suppress_coverage = invalid_pattern
        || typed_type_contains_hole(&expected.type_ref)
        || expected.key == SemanticTypeKey::Invalid;
    if !suppress_coverage {
        if let Some(coverage) =
            analyze_coverage(&expected.key, context.semantic_types(), &coverage_arms)
        {
            coverage_proves_total = coverage.missing.is_empty() && coverage.unreachable.is_empty();
            for index in coverage.unreachable {
                issues.push(MatchIssue::Unreachable {
                    arm: arms[index].span,
                });
            }
            if !coverage.missing.is_empty() {
                issues.push(MatchIssue::NonExhaustive {
                    expression: span,
                    missing: coverage.missing,
                });
            }
        }
    }

    let has_branch_mismatch = issues
        .iter()
        .any(|issue| matches!(issue, MatchIssue::BranchTypeMismatch { .. }));
    let type_ref = if has_branch_mismatch {
        TypedType::Hole
    } else {
        result_type
            .map(|(type_ref, _)| type_ref)
            .unwrap_or(TypedType::Hole)
    };
    let semantic_type = if has_branch_mismatch {
        SemanticTypeKey::Invalid
    } else {
        result_key.unwrap_or(SemanticTypeKey::Invalid)
    };
    let exhaustive = coverage_proves_total && issues.is_empty();
    let mut result = SurfaceExpressionAnalysis::valid_with_semantic_type(
        TypedExpr::Match {
            scrutinee: Box::new(scrutinee.value.clone()),
            arms: typed_arms,
            exhaustive,
            type_ref,
            origin: span,
        },
        semantic_type,
    );
    result.match_issues = issues;
    result.merge_issues_from(scrutinee);
    for child in child_analyses {
        result.merge_issues_from(child);
    }
    result
}
