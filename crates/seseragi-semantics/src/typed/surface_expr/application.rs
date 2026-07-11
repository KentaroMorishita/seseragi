use crate::{TypedExpr, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceExpr};

use super::{type_surface_expression, PureExpressionContext, SurfaceExpressionAnalysis};
use crate::typed::functions::{
    instantiated_application, instantiated_application_result_type, instantiated_semantic_result,
};
use crate::typed::pure_issues::PureCallIssue;
use crate::typed::semantic_types::{SemanticTypeKey, SemanticValueType};
use crate::typed::type_ref::{inferred_type_from_expr, typed_type_contains_hole};

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
    let Some(target) = context.target(*callee_span) else {
        return type_unknown_application(callee, &argument_nodes, expression.span(), context);
    };
    let Some(signature) = context.callable(target) else {
        return type_unknown_application(callee, &argument_nodes, expression.span(), context);
    };

    let mut arguments = Vec::with_capacity(argument_nodes.len());
    let mut semantic_arguments = Vec::with_capacity(argument_nodes.len());
    let mut child_analyses = Vec::with_capacity(argument_nodes.len());
    for argument in &argument_nodes {
        let analysis = type_surface_expression(argument, context);
        semantic_arguments.push(SemanticValueType {
            type_ref: inferred_type_from_expr(&analysis.value),
            key: analysis.semantic_type.clone(),
        });
        arguments.push(analysis.value.clone());
        child_analyses.push(analysis);
    }
    let argument_types = arguments
        .iter()
        .map(inferred_type_from_expr)
        .collect::<Vec<_>>();
    let application = instantiated_application(signature, &argument_types);
    let issue = call_issue(
        *callee_span,
        signature.parameters.len(),
        &application.parameters,
        &argument_nodes,
        &arguments,
    );
    let type_ref = if issue.is_none() {
        instantiated_application_result_type(&application, arguments.len())
    } else {
        TypedType::Hole
    };
    let semantic_type = if issue.is_some() {
        SemanticTypeKey::Invalid
    } else if arguments.len() >= signature.parameters.len() {
        instantiated_semantic_result(signature, &semantic_arguments)
    } else {
        SemanticTypeKey::Other
    };
    let mut result = SurfaceExpressionAnalysis::valid_with_semantic_type(
        TypedExpr::Call {
            callee: signature.symbol.clone(),
            arguments,
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
    let mut callee = type_surface_expression(callee, context);
    for argument in arguments {
        let child = type_surface_expression(argument, context);
        callee.merge_issues_from(child);
    }
    callee.value = TypedExpr::Variable {
        name: String::new(),
        type_ref: TypedType::Hole,
        origin: span,
    };
    callee.semantic_type = SemanticTypeKey::Invalid;
    callee
}

fn call_issue(
    callee: ByteSpan,
    parameter_count: usize,
    parameters: &[TypedType],
    argument_nodes: &[&SurfaceExpr],
    arguments: &[TypedExpr],
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
        .zip(parameters)
        .zip(argument_nodes)
        .enumerate()
        .find_map(|(index, ((argument, expected), source))| {
            let actual = inferred_type_from_expr(argument);
            (!typed_type_contains_hole(&actual) && actual != *expected).then(|| {
                PureCallIssue::ArgumentType {
                    argument: source.span(),
                    index,
                    expected: expected.clone(),
                    actual,
                }
            })
        })
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
