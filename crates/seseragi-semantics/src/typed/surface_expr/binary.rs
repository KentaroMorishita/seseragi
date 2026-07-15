use crate::{TypedExpr, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceExpr};

use super::{
    named_type, named_type_is, type_surface_expression, PureExpressionContext,
    SurfaceExpressionAnalysis,
};
use crate::typed::call_evidence::{select_arithmetic_evidence, select_equality_evidence};
use crate::typed::pure_issues::RangeIssue;
use crate::typed::type_ref::inferred_type_from_expr;

pub(super) fn type_binary(
    operator: &str,
    left: &SurfaceExpr,
    right: &SurfaceExpr,
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    let operand_context = context.without_expected();
    let left_span = left.span();
    let right_span = right.span();
    let left = type_surface_expression(left, &operand_context);
    let right = type_surface_expression(right, &operand_context);
    let result_type = binary_result_type(operator, &left.value, &right.value);
    let left_type = inferred_type_from_expr(&left.value);
    let right_type = inferred_type_from_expr(&right.value);
    let range_issue = range_issue(operator, left_span, &left_type, right_span, &right_type);
    let evidence = if matches!(operator, "==" | "!=") {
        select_equality_evidence(operator, left_type, right_type)
    } else {
        select_arithmetic_evidence(operator, left_type, right_type, result_type.clone())
    };
    let mut result = SurfaceExpressionAnalysis::valid(TypedExpr::Binary {
        operator: operator.to_owned(),
        left: Box::new(left.value.clone()),
        right: Box::new(right.value.clone()),
        evidence,
        type_ref: result_type,
        origin: span,
    });
    result.merge_issues_from(left);
    result.merge_issues_from(right);
    result.range_issue = result.range_issue.or(range_issue);
    result
}

fn binary_result_type(operator: &str, left: &TypedExpr, right: &TypedExpr) -> TypedType {
    let left_type = inferred_type_from_expr(left);
    let right_type = inferred_type_from_expr(right);
    if matches!(operator, ".." | "..=")
        && named_type_is(&left_type, "Int")
        && named_type_is(&right_type, "Int")
    {
        return TypedType::Named {
            name: "Range".to_owned(),
            arguments: vec![named_type("Int")],
        };
    }
    if matches!(operator, "+" | "-" | "*" | "/" | "%" | "**")
        && named_type_is(&left_type, "Int")
        && named_type_is(&right_type, "Int")
    {
        return named_type("Int");
    }
    if operator == "+"
        && named_type_is(&left_type, "String")
        && named_type_is(&right_type, "String")
    {
        return named_type("String");
    }
    if matches!(operator, "==" | "!=")
        && ["Int", "Bool", "String"]
            .iter()
            .any(|name| named_type_is(&left_type, name) && named_type_is(&right_type, name))
    {
        return named_type("Bool");
    }
    if matches!(operator, "<" | "<=" | ">" | ">=")
        && named_type_is(&left_type, "Int")
        && named_type_is(&right_type, "Int")
    {
        return named_type("Bool");
    }
    TypedType::Hole
}

fn range_issue(
    operator: &str,
    left: ByteSpan,
    left_type: &TypedType,
    right: ByteSpan,
    right_type: &TypedType,
) -> Option<RangeIssue> {
    if !matches!(operator, ".." | "..=") {
        return None;
    }
    if !named_type_is(left_type, "Int") {
        return Some(RangeIssue {
            endpoint: left,
            position: "start",
            actual: left_type.clone(),
        });
    }
    (!named_type_is(right_type, "Int")).then(|| RangeIssue {
        endpoint: right,
        position: "end",
        actual: right_type.clone(),
    })
}
