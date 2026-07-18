use crate::{TypedExpr, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceExpr};

use super::{type_surface_expression, PureExpressionContext, SurfaceExpressionAnalysis};
use crate::typed::functions::{
    instantiated_application_indexed, instantiated_application_result_type,
};
use crate::typed::pure_issues::PureCallIssue;
use crate::typed::semantic_types::{
    semantic_values_are_compatible, SemanticTypeKey, SemanticValueType,
};
use crate::typed::type_ref::{application_argument_type_from_expr, typed_type_contains_hole};

mod candidates;

use candidates::{select_trait_method_candidate, type_trait_method_selection_error};

pub(super) fn type_application(
    expression: &SurfaceExpr,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    type_application_with(expression, context, type_surface_expression)
}

pub(crate) fn type_application_with(
    expression: &SurfaceExpr,
    context: &PureExpressionContext<'_>,
    mut type_argument: impl FnMut(&SurfaceExpr, &PureExpressionContext<'_>) -> SurfaceExpressionAnalysis,
) -> SurfaceExpressionAnalysis {
    let (callee, argument_nodes) = flatten_application(expression);
    if let SurfaceExpr::Member {
        receiver,
        field,
        field_span,
        ..
    } = callee
    {
        let receiver_analysis = type_argument(receiver, &context.without_expected());
        if let Some(signature) = context.inherent_method(&receiver_analysis.semantic_type, field) {
            let mut method_arguments = Vec::with_capacity(argument_nodes.len() + 1);
            method_arguments.push(receiver.as_ref());
            method_arguments.extend(argument_nodes);
            return type_known_application(
                signature,
                *field_span,
                &method_arguments,
                expression.span(),
                context,
                &mut type_argument,
            );
        }
    }
    let callee_span = callee.span();
    let signature = if let Some(target) = context.target(callee_span) {
        let Some(signature) = context.callable_value(target) else {
            return type_unknown_application(callee, &argument_nodes, expression.span(), context);
        };
        signature
    } else if matches!(callee, SurfaceExpr::Name { .. }) {
        match select_trait_method_candidate(callee_span, &argument_nodes, context) {
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
    } else {
        return type_unknown_application(callee, &argument_nodes, expression.span(), context);
    };

    type_known_application(
        signature,
        callee_span,
        &argument_nodes,
        expression.span(),
        context,
        type_argument,
    )
}

pub(super) fn type_inherent_method_member(
    receiver: &SurfaceExpr,
    field: &str,
    field_span: ByteSpan,
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> Option<SurfaceExpressionAnalysis> {
    let receiver_analysis = type_surface_expression(receiver, &context.without_expected());
    let signature = context.inherent_method(&receiver_analysis.semantic_type, field)?;
    Some(type_known_application(
        signature,
        field_span,
        &[receiver],
        span,
        context,
        type_surface_expression,
    ))
}

fn type_known_application(
    signature: crate::typed::functions::TopLevelPureFunction,
    callee_span: ByteSpan,
    argument_nodes: &[&SurfaceExpr],
    expression_span: ByteSpan,
    context: &PureExpressionContext<'_>,
    mut type_argument: impl FnMut(&SurfaceExpr, &PureExpressionContext<'_>) -> SurfaceExpressionAnalysis,
) -> SurfaceExpressionAnalysis {
    let expected_application = context.expected();
    let mut analyses = (0..argument_nodes.len())
        .map(|_| None)
        .collect::<Vec<Option<SurfaceExpressionAnalysis>>>();
    let mut semantic_arguments = (0..argument_nodes.len())
        .map(|_| None)
        .collect::<Vec<Option<SemanticValueType>>>();
    let argument_order = argument_nodes
        .iter()
        .enumerate()
        .filter(|(_, argument)| !is_lambda_expression(argument))
        .chain(
            argument_nodes
                .iter()
                .enumerate()
                .filter(|(_, argument)| is_lambda_expression(argument)),
        )
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    for index in argument_order {
        let argument = argument_nodes[index];
        let indexed_arguments = semantic_arguments
            .iter()
            .enumerate()
            .filter_map(|(index, argument)| argument.clone().map(|argument| (index, argument)))
            .collect::<Vec<_>>();
        let mut partial_application = instantiated_application_indexed(
            &signature,
            expected_application,
            argument_nodes.len(),
            &indexed_arguments,
        );
        for parameter in &mut partial_application.parameters {
            *parameter = context.hydrate_semantic_value(parameter.clone());
        }
        let expected = partial_application
            .parameters
            .get(index)
            .cloned()
            .filter(|expected| {
                !is_unresolved_parameter_expectation(
                    &expected.type_ref,
                    &signature.type_parameters,
                    &partial_application.resolved_type_parameters,
                )
            })
            .map(|mut expected| {
                expected.type_ref = mask_unresolved_type_parameters(
                    &expected.type_ref,
                    &signature.type_parameters,
                    &partial_application.resolved_type_parameters,
                );
                expected
            });
        let argument_context = context.with_expected(expected);
        let analysis = type_argument(argument, &argument_context);
        semantic_arguments[index] = Some(SemanticValueType {
            type_ref: application_argument_type_from_expr(&analysis.value),
            key: analysis.semantic_type.clone(),
        });
        analyses[index] = Some(analysis);
    }
    let indexed_arguments = semantic_arguments
        .iter()
        .enumerate()
        .filter_map(|(index, argument)| argument.clone().map(|argument| (index, argument)))
        .collect::<Vec<_>>();
    let mut application = instantiated_application_indexed(
        &signature,
        expected_application,
        argument_nodes.len(),
        &indexed_arguments,
    );
    for parameter in &mut application.parameters {
        *parameter = context.hydrate_semantic_value(parameter.clone());
    }
    application.result = context.hydrate_semantic_value(application.result);
    let child_analyses = analyses
        .into_iter()
        .map(|analysis| analysis.expect("every application argument is typed"))
        .collect::<Vec<_>>();
    let arguments = child_analyses
        .iter()
        .map(|analysis| analysis.value.clone())
        .collect::<Vec<_>>();
    let semantic_arguments = semantic_arguments
        .into_iter()
        .map(|argument| argument.expect("every application argument has a semantic type"))
        .collect::<Vec<_>>();
    let mut issue = call_issue(
        callee_span,
        signature.parameters.len(),
        &application.parameters,
        argument_nodes,
        &arguments,
        &semantic_arguments,
    );
    let saturated = arguments.len() >= signature.parameters.len();
    let concrete_partial_constraints = !saturated
        && signature.trait_identity.is_none()
        && !application.constraints.is_empty()
        && application.constraints.iter().all(|constraint| {
            constraint.arguments.iter().all(|argument| {
                !contains_unresolved_type_parameter(
                    argument,
                    &signature.type_parameters,
                    &application.resolved_type_parameters,
                )
            })
        });
    let scoped_partial_evidence = (!saturated
        && signature.trait_identity.is_none()
        && !application.constraints.is_empty()
        && !concrete_partial_constraints)
        .then(|| {
            context
                .select_call_evidence(&application.constraints, &application.constraint_identities)
                .ok()
                .filter(|evidence| {
                    evidence.iter().all(|item| {
                        matches!(
                            item.evidence,
                            crate::TypedInstanceEvidence::Parameter { .. }
                        )
                    })
                })
        })
        .flatten();
    let requires_evidence = saturated
        || signature.trait_identity.is_some()
        || concrete_partial_constraints
        || scoped_partial_evidence.is_some();
    let evidence = if issue.is_none() && requires_evidence {
        match scoped_partial_evidence.map(Ok).unwrap_or_else(|| {
            context
                .select_call_evidence(&application.constraints, &application.constraint_identities)
        }) {
            Ok(evidence) => evidence,
            Err(constraint) => {
                issue = Some(PureCallIssue::MissingInstance {
                    callee: callee_span,
                    constraint,
                });
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
    let deferred_evidence_parameters =
        if !evidence.is_empty() && !saturated && signature.trait_identity.is_none() {
            application
                .parameters
                .iter()
                .skip(arguments.len())
                .map(|parameter| parameter.type_ref.clone())
                .collect()
        } else {
            Vec::new()
        };
    let deferred_evidence_type_constructor_parameters = if deferred_evidence_parameters.is_empty() {
        Vec::new()
    } else {
        signature
            .type_parameters
            .iter()
            .filter(|parameter| parameter.is_constructor())
            .map(|parameter| parameter.name.clone())
            .collect()
    };
    let mut result = SurfaceExpressionAnalysis::valid_with_semantic_type(
        TypedExpr::Call {
            callee: signature.symbol.clone(),
            arguments,
            evidence,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            trait_dispatch: signature
                .trait_identity
                .clone()
                .zip(signature.trait_method.clone())
                .map(|(trait_identity, method)| crate::TypedTraitDispatch {
                    trait_identity,
                    method,
                }),
            type_ref,
            origin: expression_span,
        },
        semantic_type,
    );
    result.pure_call_issue = issue;
    for child in child_analyses {
        result.merge_issues_from(child);
    }
    result
}

fn is_lambda_expression(expression: &SurfaceExpr) -> bool {
    match expression {
        SurfaceExpr::Lambda { .. } => true,
        SurfaceExpr::Grouped { value, .. } => is_lambda_expression(value),
        _ => false,
    }
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
                let actual = application_argument_type_from_expr(argument);
                (!typed_type_contains_hole(&actual)
                    && expected.type_ref != actual
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

fn is_unresolved_parameter_expectation(
    type_ref: &TypedType,
    parameters: &[seseragi_syntax::TypeParameter],
    resolved: &std::collections::BTreeSet<String>,
) -> bool {
    match type_ref {
        TypedType::Named { name, .. } => {
            parameters.iter().any(|parameter| parameter.name == *name) && !resolved.contains(name)
        }
        TypedType::Hole => true,
        TypedType::ExternalNamed { .. }
        | TypedType::Record { .. }
        | TypedType::Tuple { .. }
        | TypedType::Function { .. } => false,
    }
}

fn mask_unresolved_type_parameters(
    type_ref: &TypedType,
    parameters: &[seseragi_syntax::TypeParameter],
    resolved: &std::collections::BTreeSet<String>,
) -> TypedType {
    match type_ref {
        TypedType::Named { name, .. }
            if parameters.iter().any(|parameter| parameter.name == *name)
                && !resolved.contains(name) =>
        {
            TypedType::Hole
        }
        TypedType::Named { name, arguments } => TypedType::Named {
            name: name.clone(),
            arguments: arguments
                .iter()
                .map(|argument| mask_unresolved_type_parameters(argument, parameters, resolved))
                .collect(),
        },
        TypedType::ExternalNamed {
            canonical,
            name,
            arguments,
        } => TypedType::ExternalNamed {
            canonical: canonical.clone(),
            name: name.clone(),
            arguments: arguments
                .iter()
                .map(|argument| mask_unresolved_type_parameters(argument, parameters, resolved))
                .collect(),
        },
        TypedType::Record { closed, fields } => TypedType::Record {
            closed: *closed,
            fields: fields
                .iter()
                .map(|field| crate::TypedRecordField {
                    name: field.name.clone(),
                    optional: field.optional,
                    type_ref: mask_unresolved_type_parameters(
                        &field.type_ref,
                        parameters,
                        resolved,
                    ),
                })
                .collect(),
        },
        TypedType::Tuple { elements } => TypedType::Tuple {
            elements: elements
                .iter()
                .map(|element| mask_unresolved_type_parameters(element, parameters, resolved))
                .collect(),
        },
        TypedType::Function { parameter, result } => TypedType::Function {
            parameter: Box::new(mask_unresolved_type_parameters(
                parameter, parameters, resolved,
            )),
            result: Box::new(mask_unresolved_type_parameters(
                result, parameters, resolved,
            )),
        },
        TypedType::Hole => TypedType::Hole,
    }
}

fn contains_unresolved_type_parameter(
    type_ref: &TypedType,
    parameters: &[seseragi_syntax::TypeParameter],
    resolved: &std::collections::BTreeSet<String>,
) -> bool {
    match type_ref {
        TypedType::Named { name, arguments } => {
            (parameters.iter().any(|parameter| parameter.name == *name) && !resolved.contains(name))
                || arguments.iter().any(|argument| {
                    contains_unresolved_type_parameter(argument, parameters, resolved)
                })
        }
        TypedType::ExternalNamed { arguments, .. } => arguments
            .iter()
            .any(|argument| contains_unresolved_type_parameter(argument, parameters, resolved)),
        TypedType::Record { fields, .. } => fields
            .iter()
            .any(|field| contains_unresolved_type_parameter(&field.type_ref, parameters, resolved)),
        TypedType::Tuple { elements } => elements
            .iter()
            .any(|element| contains_unresolved_type_parameter(element, parameters, resolved)),
        TypedType::Function { parameter, result } => {
            contains_unresolved_type_parameter(parameter, parameters, resolved)
                || contains_unresolved_type_parameter(result, parameters, resolved)
        }
        TypedType::Hole => true,
    }
}
