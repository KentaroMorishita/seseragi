use crate::{TypedExpr, TypedType};
use seseragi_syntax::{standard_operator, ByteSpan, StandardOperatorKind, SurfaceExpr};

use super::{
    named_type, named_type_is, primitive_type_is, type_surface_expression, PureExpressionContext,
    SurfaceExpressionAnalysis,
};
use crate::typed::pure_issues::{PureCallIssue, RangeIssue};
use crate::typed::semantic_types::{semantic_values_have_same_identity, SemanticValueType};
use crate::typed::type_ref::inferred_type_from_expr;

pub(super) fn type_binary(
    operator: &str,
    operator_span: ByteSpan,
    left: &SurfaceExpr,
    right: &SurfaceExpr,
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    if operator == ":" {
        return super::cons::type_cons(left, right, span, context);
    }
    if operator == "??" {
        return super::fallback::type_fallback(left, right, span, context);
    }
    let operand_context = context.without_expected();
    let left_span = left.span();
    let right_span = right.span();
    let left = type_surface_expression(left, &operand_context);
    let right = type_surface_expression(right, &operand_context);
    let left_type = inferred_type_from_expr(&left.value);
    let right_type = inferred_type_from_expr(&right.value);
    let left_value_type = SemanticValueType {
        type_ref: left_type.clone(),
        key: left.semantic_type.clone(),
    };
    let right_value_type = SemanticValueType {
        type_ref: right_type.clone(),
        key: right.semantic_type.clone(),
    };
    let left_evidence_type = context.evidence_type(&left_value_type);
    let right_evidence_type = context.evidence_type(&right_value_type);
    let mut missing_instance = None;
    let mut arithmetic_output_identity = None;
    let standard = standard_operator(operator);
    let (result_type, evidence) = if let Some(standard) =
        standard.filter(|operator| operator.kind == StandardOperatorKind::Arithmetic)
    {
        match context.select_binary_operator_evidence(
            standard.trait_name,
            left_evidence_type.clone(),
            right_evidence_type.clone(),
        ) {
            Ok((output, evidence)) => {
                arithmetic_output_identity = Some(output.key);
                (output.type_ref, vec![evidence])
            }
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
    } else if let Some(standard) = standard.filter(|operator| {
        matches!(
            operator.kind,
            StandardOperatorKind::Equality | StandardOperatorKind::Comparison
        )
    }) {
        // An annotation and a constructor may spell the same checked owner
        // differently. Compare resolved identities and scoped arguments before
        // selecting evidence; equal display names cannot prove nominal identity.
        if left_type != TypedType::Hole
            && right_type != TypedType::Hole
            && !semantic_values_have_same_identity(&left_value_type, &right_value_type)
        {
            missing_instance = Some(PureCallIssue::ArgumentType {
                argument: right_span,
                index: 1,
                expected: left_type.clone(),
                actual: right_type.clone(),
            });
            (TypedType::Hole, Vec::new())
        } else {
            match context.select_binary_same_type_evidence(
                standard.trait_name,
                left_evidence_type.clone(),
                right_evidence_type.clone(),
            ) {
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
    } else if matches!(operator, "&&" | "||") {
        let bool_type = named_type("Bool");
        missing_instance = if left_type != TypedType::Hole
            && !primitive_type_is(&left_type, &left.semantic_type, "Bool")
        {
            Some(PureCallIssue::ArgumentType {
                argument: left_span,
                index: 0,
                expected: bool_type.clone(),
                actual: left_type.clone(),
            })
        } else if right_type != TypedType::Hole
            && !primitive_type_is(&right_type, &right.semantic_type, "Bool")
        {
            Some(PureCallIssue::ArgumentType {
                argument: right_span,
                index: 1,
                expected: bool_type.clone(),
                actual: right_type.clone(),
            })
        } else {
            None
        };
        (bool_type, Vec::new())
    } else {
        let result_type = binary_result_type(operator, &left.value, &right.value);
        (result_type, Vec::new())
    };
    let range_issue = range_issue(operator, left_span, &left_type, right_span, &right_type);
    let semantic_type = if missing_instance.is_some() || result_type == TypedType::Hole {
        crate::typed::semantic_types::SemanticTypeKey::Invalid
    } else if let Some(key) = arithmetic_output_identity {
        key
    } else if standard.is_some_and(|operator| {
        matches!(
            operator.kind,
            StandardOperatorKind::Equality | StandardOperatorKind::Comparison
        )
    }) || matches!(operator, "&&" | "||")
    {
        crate::typed::semantic_types::SemanticTypeKey::Other
    } else {
        context.semantic_value_from_typed_type(&result_type).key
    };
    let mut result = SurfaceExpressionAnalysis::valid_with_semantic_type(
        TypedExpr::Binary {
            operator: operator.to_owned(),
            left: Box::new(left.value.clone()),
            right: Box::new(right.value.clone()),
            evidence,
            type_ref: result_type,
            origin: span,
        },
        semantic_type,
    );
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
