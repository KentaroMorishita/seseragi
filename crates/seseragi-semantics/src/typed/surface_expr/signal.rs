use crate::typed::pure_issues::PureCallIssue;
use crate::typed::semantic_types::semantic_values_are_compatible;
use crate::typed::type_ref::inferred_type_from_expr;
use crate::{TypedExpr, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceExpr};

use super::{type_surface_expression, PureExpressionContext, SurfaceExpressionAnalysis};

pub(super) fn type_read(
    operand: &SurfaceExpr,
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    let operand_span = operand.span();
    let operand = type_surface_expression(operand, &context.without_expected());
    let actual = inferred_type_from_expr(&operand.value);
    let call = crate::standard::standard_signal_read_call(&actual);
    let mut result = SurfaceExpressionAnalysis::valid(TypedExpr::Call {
        callee: call
            .as_ref()
            .map(|call| call.canonical)
            .unwrap_or_else(|| crate::standard::standard_signal_read_recovery_call().canonical)
            .to_owned(),
        arguments: vec![operand.value.clone()],
        evidence: Vec::new(),
        deferred_evidence_parameters: Vec::new(),
        deferred_evidence_type_constructor_parameters: Vec::new(),
        trait_dispatch: None,
        type_ref: call
            .map(|call| call.result)
            .unwrap_or_else(|| crate::standard::standard_signal_read_recovery_call().result),
        origin: span,
    });
    result.merge_issues_from(operand);
    if actual != TypedType::Hole && crate::standard::standard_signal_read_call(&actual).is_none() {
        result.pure_call_issue = result.pure_call_issue.or(Some(PureCallIssue::ArgumentType {
            argument: operand_span,
            index: 0,
            expected: crate::standard::standard_signal_expected(false),
            actual,
        }));
    }
    result
}

pub(super) fn type_assignment(
    target: &SurfaceExpr,
    value: &SurfaceExpr,
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    let target_span = target.span();
    let value_span = value.span();
    let target = type_surface_expression(target, &context.without_expected());
    let target_type = inferred_type_from_expr(&target.value);
    let contract = crate::standard::standard_signal_set_call(&target_type);
    let value_context = contract
        .as_ref()
        .map(|(expected, _)| {
            context.with_expected(Some(context.semantic_value_from_typed_type(expected)))
        })
        .unwrap_or_else(|| context.without_expected());
    let value = type_surface_expression(value, &value_context);
    let value_type = inferred_type_from_expr(&value.value);
    let call = contract
        .as_ref()
        .map(|(_, call)| call)
        .map(|call| (call.canonical, call.result.clone()))
        .unwrap_or_else(|| {
            let call = crate::standard::standard_signal_set_recovery_call();
            (call.canonical, call.result)
        });
    let mut result = SurfaceExpressionAnalysis::valid(TypedExpr::Call {
        callee: call.0.to_owned(),
        arguments: vec![value.value.clone(), target.value.clone()],
        evidence: Vec::new(),
        deferred_evidence_parameters: Vec::new(),
        deferred_evidence_type_constructor_parameters: Vec::new(),
        trait_dispatch: None,
        type_ref: call.1,
        origin: span,
    });
    result.merge_issues_from(target);
    result.merge_issues_from(value);

    let issue = if target_type != TypedType::Hole && contract.is_none() {
        Some(PureCallIssue::ArgumentType {
            argument: target_span,
            index: 1,
            expected: crate::standard::standard_signal_expected(true),
            actual: target_type,
        })
    } else if let Some((expected, _)) = contract {
        let expected_semantic = context.semantic_value_from_typed_type(&expected);
        let actual_semantic = context.semantic_value_from_typed_type(&value_type);
        (!semantic_values_are_compatible(&expected_semantic, &actual_semantic)).then_some(
            PureCallIssue::ArgumentType {
                argument: value_span,
                index: 0,
                expected,
                actual: value_type,
            },
        )
    } else {
        None
    };
    result.pure_call_issue = result.pure_call_issue.or(issue);
    result
}
