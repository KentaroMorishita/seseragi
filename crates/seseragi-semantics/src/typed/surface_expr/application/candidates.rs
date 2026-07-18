use crate::{TypedExpr, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceExpr};

use super::super::{type_surface_expression, PureExpressionContext, SurfaceExpressionAnalysis};
use super::call_issue;
use crate::typed::functions::{instantiated_application, TopLevelPureFunction};
use crate::typed::pure_issues::PureCallIssue;
use crate::typed::semantic_types::{
    semantic_values_are_compatible, SemanticTypeKey, SemanticValueType,
};
use crate::typed::type_ref::inferred_type_from_expr;

pub(super) fn select_trait_method_candidate(
    callee: ByteSpan,
    argument_nodes: &[&SurfaceExpr],
    context: &PureExpressionContext<'_>,
) -> Option<Result<TopLevelPureFunction, PureCallIssue>> {
    let candidates = context.callable_candidates(callee);
    if candidates.is_empty() {
        return None;
    }
    let child_context = context.without_expected();
    let analyses = argument_nodes
        .iter()
        .map(|argument| type_surface_expression(argument, &child_context))
        .collect::<Vec<_>>();
    let arguments = analyses
        .iter()
        .map(|analysis| analysis.value.clone())
        .collect::<Vec<_>>();
    let semantic_arguments = analyses
        .iter()
        .map(|analysis| SemanticValueType {
            type_ref: inferred_type_from_expr(&analysis.value),
            key: analysis.semantic_type.clone(),
        })
        .collect::<Vec<_>>();
    let type_matches = candidates
        .into_iter()
        .filter_map(|signature| {
            let mut application = instantiated_application(
                &signature,
                context.expected(),
                argument_nodes.len(),
                &semantic_arguments,
            );
            for parameter in &mut application.parameters {
                *parameter = context.hydrate_semantic_value(parameter.clone());
            }
            application.result = context.hydrate_semantic_value(application.result);
            let issue = call_issue(
                callee,
                signature.parameters.len(),
                &application.parameters,
                argument_nodes,
                &arguments,
                &semantic_arguments,
            );
            let result_matches = argument_nodes.len() < signature.parameters.len()
                || context.expected().is_none_or(|expected| {
                    semantic_values_are_compatible(expected, &application.result)
                });
            (issue.is_none() && result_matches).then_some((signature, application))
        })
        .collect::<Vec<_>>();
    match type_matches.as_slice() {
        [] => Some(Err(PureCallIssue::TraitMethodNoMatch { callee })),
        [(signature, _)] => Some(Ok(signature.clone())),
        _ => {
            let evidence_matches = type_matches
                .iter()
                .filter(|(_, application)| {
                    context
                        .select_call_evidence(
                            &application.constraints,
                            &application.constraint_identities,
                        )
                        .is_ok()
                })
                .map(|(signature, _)| signature.clone())
                .collect::<Vec<_>>();
            match evidence_matches.as_slice() {
                [signature] => Some(Ok(signature.clone())),
                _ => Some(Err(PureCallIssue::TraitMethodAmbiguous { callee })),
            }
        }
    }
}

pub(super) fn type_trait_method_selection_error(
    argument_nodes: &[&SurfaceExpr],
    span: ByteSpan,
    issue: PureCallIssue,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    let child_context = context.without_expected();
    let mut result = SurfaceExpressionAnalysis::valid_with_semantic_type(
        TypedExpr::Variable {
            name: String::new(),
            evidence: Vec::new(),
            type_ref: TypedType::Hole,
            origin: span,
        },
        SemanticTypeKey::Invalid,
    );
    result.pure_call_issue = Some(issue);
    for argument in argument_nodes {
        result.merge_issues_from(type_surface_expression(argument, &child_context));
    }
    result
}
