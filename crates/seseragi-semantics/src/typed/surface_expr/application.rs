use crate::{TypedExpr, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceExpr};

use super::{type_surface_expression, PureExpressionContext, SurfaceExpressionAnalysis};
use crate::typed::functions::application_result_type;
use crate::typed::pure_issues::PureCallIssue;
use crate::typed::type_ref::inferred_type_from_expr;

pub(super) fn type_application(
    expression: &SurfaceExpr,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    let (callee, argument_nodes) = flatten_application(expression);
    let SurfaceExpr::Name {
        name,
        span: callee_span,
    } = callee
    else {
        return type_unknown_application(callee, &argument_nodes, expression.span(), context);
    };
    let Some(signature) = context.top_level_functions.get(name) else {
        return type_unknown_application(callee, &argument_nodes, expression.span(), context);
    };

    let mut arguments = Vec::with_capacity(argument_nodes.len());
    let mut child_analyses = Vec::with_capacity(argument_nodes.len());
    for argument in &argument_nodes {
        let analysis = type_surface_expression(argument, context);
        arguments.push(analysis.value.clone());
        child_analyses.push(analysis);
    }
    let issue = call_issue(*callee_span, signature, &argument_nodes, &arguments);
    let type_ref = if issue.is_none() {
        application_result_type(signature, arguments.len())
    } else {
        TypedType::Hole
    };
    let mut result = SurfaceExpressionAnalysis::valid(TypedExpr::Call {
        callee: signature.symbol.clone(),
        arguments,
        type_ref,
        origin: expression.span(),
    });
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
    callee
}

fn call_issue(
    callee: ByteSpan,
    signature: &crate::typed::functions::TopLevelPureFunction,
    argument_nodes: &[&SurfaceExpr],
    arguments: &[TypedExpr],
) -> Option<PureCallIssue> {
    if arguments.len() > signature.parameters.len() {
        return Some(PureCallIssue::Arity {
            callee,
            expected: signature.parameters.len(),
            actual: arguments.len(),
        });
    }
    arguments
        .iter()
        .zip(&signature.parameters)
        .zip(argument_nodes)
        .enumerate()
        .find_map(|(index, ((argument, expected), source))| {
            let actual = inferred_type_from_expr(argument);
            (actual != TypedType::Hole && actual != *expected).then(|| {
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
