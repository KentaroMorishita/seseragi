use crate::{SymbolNamespace, TypedEffect, TypedExpr, TypedType};

use super::super::functions::application_result_type_from;
use super::super::pure_issues::PureCallIssue;
use super::super::semantic_types::{semantic_values_are_compatible, SemanticValueType};
use super::super::surface_expr::{analyze_resolved_expression, PureExpressionContext};
use super::super::type_ref::{inferred_type_from_expr, typed_type_contains_hole};
use super::super::TypedResolution;
use super::flatten_application;

pub(super) fn type_imported_effect_application(
    expression: &seseragi_syntax::SurfaceExpr,
    context: &PureExpressionContext<'_>,
    resolution: &TypedResolution<'_>,
    issues: &mut Vec<PureCallIssue>,
) -> Option<TypedExpr> {
    let (callee, argument_nodes) = flatten_application(expression);
    let seseragi_syntax::SurfaceExpr::Name { span: callee, .. } = callee else {
        return None;
    };
    let target = resolution.target(*callee, SymbolNamespace::Value)?;
    let signature = resolution.imported_effect(target)?;
    let analyses = argument_nodes
        .iter()
        .enumerate()
        .map(|(index, argument)| {
            let expected = signature.parameters.get(index).cloned();
            analyze_resolved_expression(argument, &context.with_expected(expected))
        })
        .collect::<Vec<_>>();

    if argument_nodes.len() > signature.parameters.len() {
        issues.push(PureCallIssue::Arity {
            callee: *callee,
            expected: signature.parameters.len(),
            actual: argument_nodes.len(),
        });
    } else if let Some(issue) =
        argument_type_issue(&argument_nodes, &analyses, &signature.parameters)
    {
        issues.push(issue);
    }

    let arguments = analyses
        .into_iter()
        .map(|analysis| analysis.value)
        .collect::<Vec<_>>();
    if arguments.len() < signature.parameters.len() {
        return Some(TypedExpr::Call {
            callee: signature.symbol.clone(),
            arguments,
            type_ref: application_result_type_from(
                &signature
                    .parameters
                    .iter()
                    .map(|parameter| parameter.type_ref.clone())
                    .collect::<Vec<_>>(),
                effect_value_type(&signature.effect),
                argument_nodes.len(),
            ),
            origin: expression.span(),
        });
    }

    Some(TypedExpr::EffectInvoke {
        callee: signature.symbol.clone(),
        effect: signature.effect.clone(),
        arguments,
        origin: expression.span(),
    })
}

fn argument_type_issue(
    argument_nodes: &[&seseragi_syntax::SurfaceExpr],
    analyses: &[super::super::surface_expr::SurfaceExpressionAnalysis],
    expected: &[SemanticValueType],
) -> Option<PureCallIssue> {
    argument_nodes
        .iter()
        .zip(analyses)
        .zip(expected)
        .enumerate()
        .find_map(|(index, ((argument, analysis), expected))| {
            let actual = SemanticValueType {
                type_ref: inferred_type_from_expr(&analysis.value),
                key: analysis.semantic_type.clone(),
            };
            (!typed_type_contains_hole(&actual.type_ref)
                && !semantic_values_are_compatible(expected, &actual))
            .then(|| PureCallIssue::ArgumentType {
                argument: argument.span(),
                index,
                expected: expected.type_ref.clone(),
                actual: actual.type_ref,
            })
        })
}

fn effect_value_type(effect: &TypedEffect) -> TypedType {
    TypedType::Named {
        name: "Effect".to_owned(),
        arguments: vec![
            effect.environment.clone(),
            effect.failure.clone(),
            effect.success.clone(),
        ],
    }
}
