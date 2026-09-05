use crate::{TypedExpr, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceExpr};

use super::{
    named_type_is, type_surface_expression, PureExpressionContext, SurfaceExpressionAnalysis,
};
use crate::typed::pure_issues::ConditionalIssue;
use crate::typed::semantic_types::SemanticTypeKey;
use crate::typed::type_ref::{
    application_argument_type_from_expr, inferred_type_from_expr, typed_type_contains_hole,
};

pub(super) fn type_if(
    condition: &SurfaceExpr,
    then_branch: &SurfaceExpr,
    else_branch: &SurfaceExpr,
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    type_if_with(
        condition,
        then_branch,
        else_branch,
        span,
        context,
        type_surface_expression,
    )
}

pub(crate) fn type_if_with(
    condition: &SurfaceExpr,
    then_branch: &SurfaceExpr,
    else_branch: &SurfaceExpr,
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
    mut type_body: impl FnMut(&SurfaceExpr, &PureExpressionContext<'_>) -> SurfaceExpressionAnalysis,
) -> SurfaceExpressionAnalysis {
    let condition_span = condition.span();
    let then_span = then_branch.span();
    let else_span = else_branch.span();
    let condition_context = context.without_expected();
    let condition = type_surface_expression(condition, &condition_context);
    let then_branch = type_body(then_branch, context);
    let else_branch = type_body(else_branch, context);
    let condition_type = inferred_type_from_expr(&condition.value);
    let then_type = application_argument_type_from_expr(&then_branch.value);
    let else_type = application_argument_type_from_expr(&else_branch.value);
    let joined_type = crate::typed::effect::join_branch_types(&then_type, &else_type);
    let has_unresolved_type = typed_type_contains_hole(&condition_type)
        || typed_type_contains_hole(&then_type)
        || typed_type_contains_hole(&else_type);
    let conditional_issue = if typed_type_contains_hole(&condition_type) {
        None
    } else if !named_type_is(&condition_type, "Bool") {
        Some(ConditionalIssue::ConditionNotBool {
            condition: condition_span,
            actual: condition_type,
        })
    } else if !typed_type_contains_hole(&then_type)
        && !typed_type_contains_hole(&else_type)
        && joined_type.is_none()
    {
        Some(ConditionalIssue::BranchTypeMismatch {
            then_branch: then_span,
            else_branch: else_span,
            then_type: then_type.clone(),
            else_type: else_type.clone(),
        })
    } else {
        None
    };
    let type_ref = if conditional_issue.is_none() && !has_unresolved_type {
        joined_type.unwrap_or(TypedType::Hole)
    } else {
        TypedType::Hole
    };
    let semantic_type =
        if conditional_issue.is_none() && then_branch.semantic_type == else_branch.semantic_type {
            then_branch.semantic_type.clone()
        } else {
            SemanticTypeKey::Invalid
        };
    let semantic_type = if crate::typed::type_ref::effect_from_value_type(&type_ref).is_some() {
        context.semantic_value_from_typed_type(&type_ref).key
    } else {
        semantic_type
    };
    let mut result = SurfaceExpressionAnalysis::valid_with_semantic_type(
        TypedExpr::If {
            condition: Box::new(condition.value.clone()),
            then_branch: Box::new(then_branch.value.clone()),
            else_branch: Box::new(else_branch.value.clone()),
            type_ref,
            origin: span,
        },
        semantic_type,
    );
    result.conditional_issue = conditional_issue;
    result.merge_issues_from(condition);
    result.merge_issues_from(then_branch);
    result.merge_issues_from(else_branch);
    result
}
