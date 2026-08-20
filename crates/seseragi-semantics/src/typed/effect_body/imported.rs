use crate::{SymbolNamespace, TypedEffect, TypedExpr, TypedType};

use super::super::surface_expr::application;
use super::super::surface_expr::{PureExpressionContext, SurfaceExpressionAnalysis};
use super::super::type_ref::{application_argument_type_from_expr, effect_from_value_type};
use super::super::TypedResolution;
use super::{flatten_application, EffectBodyIssues};

pub(super) fn type_imported_effect_application(
    expression: &seseragi_syntax::SurfaceExpr,
    context: &PureExpressionContext<'_>,
    resolution: &TypedResolution<'_>,
    issues: &mut EffectBodyIssues<'_>,
) -> Option<SurfaceExpressionAnalysis> {
    let (callee, argument_nodes) = flatten_application(expression);
    let callee_span = callee.span();
    let target = resolution.target(callee_span, SymbolNamespace::Value)?;
    let imported = resolution.imported_effect(target)?;
    let parameter_count = imported.signature.parameters.len();
    let mut analysis = application::type_known_application_with_explicit(
        imported.signature.clone(),
        application::explicit_type_arguments(callee),
        callee_span,
        &argument_nodes,
        expression.span(),
        context,
        |argument, argument_context| {
            let value =
                super::type_effect_expression(argument, argument_context, resolution, issues);
            let semantic_type = argument_context
                .semantic_value_from_typed_type(&application_argument_type_from_expr(&value))
                .key;
            SurfaceExpressionAnalysis::valid_with_semantic_type(value, semantic_type)
        },
    );

    if argument_nodes.len() < parameter_count || analysis.pure_call_issue.is_some() {
        return Some(analysis);
    }

    let TypedExpr::Call {
        callee,
        arguments,
        evidence,
        type_ref,
        origin,
        ..
    } = analysis.value
    else {
        return Some(analysis);
    };
    let mut effect = effect_from_type(type_ref)
        .expect("imported effect signatures retain their instantiated Effect result");
    add_standard_temporal_requirement(&callee, &arguments, &mut effect);
    analysis.value = TypedExpr::EffectInvoke {
        callee,
        effect,
        arguments,
        evidence,
        origin,
    };
    Some(analysis)
}

fn add_standard_temporal_requirement(
    callee: &str,
    arguments: &[TypedExpr],
    effect: &mut TypedEffect,
) {
    if !matches!(
        callee,
        "std/effect::timeout"
            | "std/effect::timeoutFail"
            | "std/effect::retry"
            | "std/effect::repeat"
    ) {
        return;
    }
    let clock = match &effect.environment {
        TypedType::Record { fields, .. } => {
            fields.iter().find(|field| field.name == "clock").cloned()
        }
        _ => None,
    };
    if let Some(source) = arguments
        .last()
        .and_then(|argument| effect_from_value_type(&application_argument_type_from_expr(argument)))
    {
        effect.environment = source.environment;
    }
    let TypedType::Record { fields, .. } = &mut effect.environment else {
        return;
    };
    if fields.iter().any(|field| field.name == "clock") {
        return;
    }
    if let Some(clock) = clock {
        fields.push(clock);
    }
}

fn effect_from_type(type_ref: TypedType) -> Option<TypedEffect> {
    let TypedType::Named { name, arguments } = type_ref else {
        return None;
    };
    let [environment, failure, success] = arguments.as_slice() else {
        return None;
    };
    (name == "Effect").then(|| TypedEffect {
        environment: environment.clone(),
        failure: failure.clone(),
        success: success.clone(),
    })
}
