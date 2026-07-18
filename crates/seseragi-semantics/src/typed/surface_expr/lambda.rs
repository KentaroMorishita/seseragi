use std::collections::BTreeMap;

use crate::{TypedExpr, TypedParameter, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceExpr, SurfaceLambdaParameter};

use super::{type_surface_expression, PureExpressionContext, SurfaceExpressionAnalysis};
use crate::typed::pure_issues::PureCallIssue;
use crate::typed::semantic_types::{
    semantic_values_are_compatible, SemanticTypeKey, SemanticValueType,
};
use crate::typed::type_ref::{inferred_type_from_expr, typed_type_contains_hole};

pub(super) fn type_lambda(
    parameter: &SurfaceLambdaParameter,
    body: &SurfaceExpr,
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    let expected_parts = context.expected().and_then(|expected| {
        let TypedType::Function { parameter, result } = &expected.type_ref else {
            return None;
        };
        Some((
            context.semantic_value_from_typed_type(parameter),
            context.semantic_value_from_typed_type(result),
        ))
    });
    let annotated = parameter
        .type_ref
        .as_ref()
        .map(|type_ref| context.semantic_value_from_type_ref(type_ref));
    let mut issue = match (&expected_parts, &annotated) {
        (Some((expected, _)), Some(actual))
            if !semantic_values_are_compatible(expected, actual) =>
        {
            Some(PureCallIssue::LambdaParameterTypeMismatch {
                parameter: parameter.name_span,
                expected: expected.type_ref.clone(),
                actual: actual.type_ref.clone(),
            })
        }
        _ => None,
    };
    let parameter_type = annotated
        .or_else(|| {
            expected_parts
                .as_ref()
                .map(|(parameter, _)| parameter.clone())
        })
        .unwrap_or_else(|| {
            issue.get_or_insert(PureCallIssue::LambdaParameterTypeUnresolved {
                parameter: parameter.name_span,
            });
            SemanticValueType {
                type_ref: TypedType::Hole,
                key: SemanticTypeKey::Invalid,
            }
        });
    if typed_type_contains_hole(&parameter_type.type_ref) {
        issue.get_or_insert(PureCallIssue::LambdaParameterTypeUnresolved {
            parameter: parameter.name_span,
        });
    }

    let mut locals = BTreeMap::new();
    if let Some(symbol) = context.lambda_parameter_symbol(parameter.name_span) {
        locals.insert(symbol, parameter_type.clone());
    }
    let expected_result = expected_parts.as_ref().map(|(_, result)| result.clone());
    let body_analysis = type_surface_expression(
        body,
        &context
            .with_locals(locals)
            .with_expected(expected_result.clone()),
    );
    let actual_result = SemanticValueType {
        type_ref: inferred_type_from_expr(&body_analysis.value),
        key: body_analysis.semantic_type.clone(),
    };
    if let Some(expected) = expected_result {
        if !typed_type_contains_hole(&actual_result.type_ref)
            && !semantic_values_are_compatible(&expected, &actual_result)
        {
            issue.get_or_insert(PureCallIssue::LambdaBodyTypeMismatch {
                body: body.span(),
                expected: expected.type_ref,
                actual: actual_result.type_ref.clone(),
            });
        }
    }

    let type_ref = TypedType::Function {
        parameter: Box::new(parameter_type.type_ref.clone()),
        result: Box::new(actual_result.type_ref),
    };
    let mut result = SurfaceExpressionAnalysis::valid_with_semantic_type(
        TypedExpr::Lambda {
            parameter: TypedParameter::Named {
                name: parameter.name.clone(),
                type_ref: parameter_type.type_ref,
                origin: parameter.name_span,
            },
            body: Box::new(body_analysis.value.clone()),
            type_ref,
            origin: span,
        },
        if issue.is_some() {
            SemanticTypeKey::Invalid
        } else {
            SemanticTypeKey::Other
        },
    );
    result.pure_call_issue = issue;
    result.merge_issues_from(body_analysis);
    result
}
