use crate::{TypedExpr, TypedType};
use seseragi_syntax::{standard_operator, ByteSpan, StandardOperatorKind, SurfaceExpr};

use super::{
    named_type, named_type_is, type_surface_expression, PureExpressionContext,
    SurfaceExpressionAnalysis,
};
use crate::typed::pure_issues::{PureCallIssue, RangeIssue};
use crate::typed::type_ref::inferred_type_from_expr;

pub(super) fn type_binary(
    operator: &str,
    operator_span: ByteSpan,
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
    let left_type = inferred_type_from_expr(&left.value);
    let right_type = inferred_type_from_expr(&right.value);
    let mut missing_instance = None;
    let standard = standard_operator(operator);
    let (result_type, evidence) = if let Some(standard) =
        standard.filter(|operator| operator.kind == StandardOperatorKind::Arithmetic)
    {
        match context.select_binary_operator_evidence(
            standard.trait_name,
            left_type.clone(),
            right_type.clone(),
        ) {
            Ok((output, evidence)) => (output, vec![evidence]),
            Err(constraint) => {
                if left_type != TypedType::Hole && right_type != TypedType::Hole {
                    missing_instance = Some(PureCallIssue::MissingInstance {
                        callee: operator_span,
                        constraint,
                    });
                }
                (TypedType::Hole, Vec::new())
            }
        }
    } else if standard.is_some_and(|operator| operator.kind == StandardOperatorKind::Equality) {
        if left_type != TypedType::Hole && right_type != TypedType::Hole && left_type != right_type
        {
            missing_instance = Some(PureCallIssue::ArgumentType {
                argument: right_span,
                index: 1,
                expected: left_type.clone(),
                actual: right_type.clone(),
            });
            (TypedType::Hole, Vec::new())
        } else {
            match context.select_binary_equality_evidence(left_type.clone(), right_type.clone()) {
                Ok(evidence) => (named_type("Bool"), vec![evidence]),
                Err(constraint) => {
                    if left_type != TypedType::Hole && right_type != TypedType::Hole {
                        missing_instance = Some(PureCallIssue::MissingInstance {
                            callee: operator_span,
                            constraint,
                        });
                    }
                    (TypedType::Hole, Vec::new())
                }
            }
        }
    } else {
        let result_type = binary_result_type(operator, &left.value, &right.value);
        (result_type, Vec::new())
    };
    let range_issue = range_issue(operator, left_span, &left_type, right_span, &right_type);
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
    result.pure_call_issue = result.pure_call_issue.or(missing_instance);
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
