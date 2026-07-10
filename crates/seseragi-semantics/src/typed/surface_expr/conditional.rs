use crate::{TypedExpr, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceExpr};

use super::{
    named_type_is, type_surface_expression, PureExpressionContext, SurfaceExpressionAnalysis,
};
use crate::typed::pure_issues::ConditionalIssue;
use crate::typed::type_ref::inferred_type_from_expr;

pub(super) fn type_if(
    condition: &SurfaceExpr,
    then_branch: &SurfaceExpr,
    else_branch: &SurfaceExpr,
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    let condition_span = condition.span();
    let then_span = then_branch.span();
    let else_span = else_branch.span();
    let condition = type_surface_expression(condition, context);
    let then_branch = type_surface_expression(then_branch, context);
    let else_branch = type_surface_expression(else_branch, context);
    let condition_type = inferred_type_from_expr(&condition.value);
    let then_type = inferred_type_from_expr(&then_branch.value);
    let else_type = inferred_type_from_expr(&else_branch.value);
    let conditional_issue = if !named_type_is(&condition_type, "Bool") {
        Some(ConditionalIssue::ConditionNotBool {
            condition: condition_span,
            actual: condition_type,
        })
    } else if then_type != else_type {
        Some(ConditionalIssue::BranchTypeMismatch {
            then_branch: then_span,
            else_branch: else_span,
            then_type: then_type.clone(),
            else_type: else_type.clone(),
        })
    } else {
        None
    };
    let type_ref = if conditional_issue.is_none() {
        then_type
    } else {
        TypedType::Hole
    };
    let mut result = SurfaceExpressionAnalysis::valid(TypedExpr::If {
        condition: Box::new(condition.value.clone()),
        then_branch: Box::new(then_branch.value.clone()),
        else_branch: Box::new(else_branch.value.clone()),
        type_ref,
        origin: span,
    });
    result.conditional_issue = conditional_issue;
    result.merge_issues_from(condition);
    result.merge_issues_from(then_branch);
    result.merge_issues_from(else_branch);
    result
}
