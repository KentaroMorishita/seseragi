use crate::{TypedExpr, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceExpr};

use super::{type_surface_expression, PureExpressionContext, SurfaceExpressionAnalysis};
use crate::typed::functions::{instantiated_application, instantiated_application_result_type};
use crate::typed::pure_issues::PureCallIssue;
use crate::typed::semantic_types::{
    semantic_values_are_compatible, SemanticTypeKey, SemanticValueType,
};
use crate::typed::type_ref::{inferred_type_from_expr, typed_type_contains_hole};

mod candidates;

use candidates::{select_trait_method_candidate, type_trait_method_selection_error};

pub(super) fn type_application(
    expression: &SurfaceExpr,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    let (callee, argument_nodes) = flatten_application(expression);
    let SurfaceExpr::Name {
        span: callee_span, ..
    } = callee
    else {
        return type_unknown_application(callee, &argument_nodes, expression.span(), context);
    };
    let signature = if let Some(target) = context.target(*callee_span) {
        let Some(signature) = context.callable_value(target) else {
            return type_unknown_application(callee, &argument_nodes, expression.span(), context);
        };
        signature
    } else {
        match select_trait_method_candidate(*callee_span, &argument_nodes, context) {
            Some(Ok(signature)) => signature,
            Some(Err(issue)) => {
                return type_trait_method_selection_error(
                    &argument_nodes,
                    expression.span(),
                    issue,
                    context,
                );
            }
            None => {
                return type_unknown_application(
                    callee,
                    &argument_nodes,
                    expression.span(),
                    context,
                );
            }
        }
    };

    let expected_result = if argument_nodes.len() >= signature.parameters.len() {
        context.expected()
    } else {
        None
    };
    let seeded_application = instantiated_application(&signature, expected_result, &[]);
    let mut arguments = Vec::with_capacity(argument_nodes.len());
    let mut semantic_arguments = Vec::with_capacity(argument_nodes.len());
    let mut child_analyses = Vec::with_capacity(argument_nodes.len());
    for (index, argument) in argument_nodes.iter().enumerate() {
        let expected = seeded_application.parameters.get(index).cloned();
        let argument_context = context.with_expected(expected);
        let analysis = type_surface_expression(argument, &argument_context);
        semantic_arguments.push(SemanticValueType {
            type_ref: inferred_type_from_expr(&analysis.value),
            key: analysis.semantic_type.clone(),
        });
        arguments.push(analysis.value.clone());
        child_analyses.push(analysis);
    }
    let application = instantiated_application(&signature, expected_result, &semantic_arguments);
    let mut issue = call_issue(
        *callee_span,
        signature.parameters.len(),
        &application.parameters,
        &argument_nodes,
        &arguments,
        &semantic_arguments,
    );
    let saturated = arguments.len() >= signature.parameters.len();
    let evidence = if issue.is_none() && (saturated || signature.trait_identity.is_some()) {
        match context
            .select_call_evidence(&application.constraints, &application.constraint_identities)
        {
            Ok(evidence) => evidence,
            Err(constraint) => {
                if saturated || signature.trait_identity.is_some() {
                    issue = Some(PureCallIssue::MissingInstance {
                        callee: *callee_span,
                        constraint,
                    });
                }
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };
    let type_ref = if issue.is_none() {
        instantiated_application_result_type(&application, arguments.len())
    } else {
        TypedType::Hole
    };
    let semantic_type = if issue.is_some() {
        SemanticTypeKey::Invalid
    } else if arguments.len() >= signature.parameters.len() {
        application.result.key.clone()
    } else {
        SemanticTypeKey::Other
    };
    let mut result = SurfaceExpressionAnalysis::valid_with_semantic_type(
        TypedExpr::Call {
            callee: signature.symbol.clone(),
            arguments,
            evidence,
            trait_dispatch: signature
                .trait_identity
                .clone()
                .zip(signature.trait_method.clone())
                .map(|(trait_identity, method)| crate::TypedTraitDispatch {
                    trait_identity,
                    method,
                }),
            type_ref,
            origin: expression.span(),
        },
        semantic_type,
    );
    result.pure_call_issue = issue;
    for child in child_analyses {
        result.merge_issues_from(child);
    }
    result
}

fn type_unknown_application(
    callee: &SurfaceExpr,
    arguments: &[&SurfaceExpr],
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    let child_context = context.without_expected();
    let mut callee = type_surface_expression(callee, &child_context);
    for argument in arguments {
        let child = type_surface_expression(argument, &child_context);
        callee.merge_issues_from(child);
    }
    callee.value = TypedExpr::Variable {
        name: String::new(),
        evidence: Vec::new(),
        type_ref: TypedType::Hole,
        origin: span,
    };
    callee.semantic_type = SemanticTypeKey::Invalid;
    callee
}

fn call_issue(
    callee: ByteSpan,
    parameter_count: usize,
    parameters: &[SemanticValueType],
    argument_nodes: &[&SurfaceExpr],
    arguments: &[TypedExpr],
    semantic_arguments: &[SemanticValueType],
) -> Option<PureCallIssue> {
    if arguments.len() > parameter_count {
        return Some(PureCallIssue::Arity {
            callee,
            expected: parameter_count,
            actual: arguments.len(),
        });
    }
    arguments
        .iter()
        .zip(semantic_arguments)
        .zip(parameters)
        .zip(argument_nodes)
        .enumerate()
        .find_map(
            |(index, (((argument, actual_semantic), expected), source))| {
                let actual = inferred_type_from_expr(argument);
                (!typed_type_contains_hole(&actual)
                    && !semantic_values_are_compatible(expected, actual_semantic))
                .then(|| PureCallIssue::ArgumentType {
                    argument: source.span(),
                    index,
                    expected: expected.type_ref.clone(),
                    actual,
                })
            },
        )
}

fn flatten_application(expression: &SurfaceExpr) -> (&SurfaceExpr, Vec<&SurfaceExpr>) {
    let mut callee = expression;
    let mut arguments = Vec::new();
    while let SurfaceExpr::Application {
        function, argument, ..
    } = callee
    {
        arguments.push(argument.as_ref());
        callee = function.as_ref();
    }
    arguments.reverse();
    (callee, arguments)
}
