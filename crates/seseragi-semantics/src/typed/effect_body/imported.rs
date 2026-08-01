use crate::{SymbolNamespace, TypedEffect, TypedExpr, TypedType};

use super::super::surface_expr::{analyze_resolved_expression, application};
use super::super::surface_expr::{PureExpressionContext, SurfaceExpressionAnalysis};
use super::super::TypedResolution;
use super::{flatten_application, EffectBodyIssues};

pub(super) fn type_imported_effect_application(
    expression: &seseragi_syntax::SurfaceExpr,
    context: &PureExpressionContext<'_>,
    resolution: &TypedResolution<'_>,
    _issues: &mut EffectBodyIssues<'_>,
) -> Option<SurfaceExpressionAnalysis> {
    let (callee, argument_nodes) = flatten_application(expression);
    let callee_span = callee.span();
    let target = resolution.target(callee_span, SymbolNamespace::Value)?;
    let imported = resolution.imported_effect(target)?;
    let parameter_count = imported.signature.parameters.len();
    let mut analysis = application::type_known_application(
        imported.signature.clone(),
        callee_span,
        &argument_nodes,
        expression.span(),
        context,
        analyze_resolved_expression,
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
    let effect = effect_from_type(type_ref)
        .expect("imported effect signatures retain their instantiated Effect result");
    analysis.value = TypedExpr::EffectInvoke {
        callee,
        effect,
        arguments,
        evidence,
        origin,
    };
    Some(analysis)
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
