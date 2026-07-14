use crate::{TypedConstraint, TypedRecordField, TypedType};
use std::collections::BTreeMap;

use super::{application_result_type_from, TopLevelPureFunction};
use crate::typed::semantic_types::{instantiate_callable, SemanticValueType};
use crate::typed::type_ref::typed_type_contains_hole;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InstantiatedApplication {
    pub(crate) parameters: Vec<SemanticValueType>,
    pub(crate) result: SemanticValueType,
    pub(crate) constraints: Vec<TypedConstraint>,
}

pub(crate) fn instantiated_application(
    signature: &TopLevelPureFunction,
    expected_result: Option<&SemanticValueType>,
    arguments: &[SemanticValueType],
) -> InstantiatedApplication {
    let mut substitutions = BTreeMap::new();
    if let Some(expected_result) = expected_result {
        infer_type_parameters(
            &signature.result,
            &expected_result.type_ref,
            &signature.type_parameters,
            &mut substitutions,
        );
    }
    for (parameter, argument) in signature.parameters.iter().zip(arguments) {
        infer_type_parameters(
            parameter,
            &argument.type_ref,
            &signature.type_parameters,
            &mut substitutions,
        );
    }
    let semantic_parameters = signature
        .parameters
        .iter()
        .cloned()
        .zip(signature.semantic_parameters.iter().cloned())
        .map(|(type_ref, key)| SemanticValueType { type_ref, key })
        .collect::<Vec<_>>();
    let semantic = instantiate_callable(
        &semantic_parameters,
        expected_result,
        arguments,
        &SemanticValueType {
            type_ref: signature.result.clone(),
            key: signature.semantic_result.clone(),
        },
    );
    let parameter_types = signature
        .parameters
        .iter()
        .map(|parameter| substitute_type_parameters(parameter, &substitutions))
        .collect::<Vec<_>>();
    let result_type = substitute_type_parameters(&signature.result, &substitutions);
    let constraints = signature
        .constraints
        .iter()
        .map(|constraint| TypedConstraint {
            name: constraint.name.clone(),
            arguments: constraint
                .arguments
                .iter()
                .map(|argument| substitute_type_parameters(argument, &substitutions))
                .collect(),
        })
        .collect();
    InstantiatedApplication {
        parameters: parameter_types
            .into_iter()
            .zip(semantic.parameters)
            .map(|(type_ref, semantic)| SemanticValueType {
                type_ref,
                key: semantic.key,
            })
            .collect(),
        result: SemanticValueType {
            type_ref: result_type,
            key: semantic.result.key,
        },
        constraints,
    }
}

pub(crate) fn instantiated_application_result_type(
    application: &InstantiatedApplication,
    argument_count: usize,
) -> TypedType {
    application_result_type_from(
        &application
            .parameters
            .iter()
            .map(|parameter| parameter.type_ref.clone())
            .collect::<Vec<_>>(),
        application.result.type_ref.clone(),
        argument_count,
    )
}

fn infer_type_parameters(
    parameter: &TypedType,
    argument: &TypedType,
    type_parameters: &[String],
    substitutions: &mut BTreeMap<String, TypedType>,
) {
    if let TypedType::Named { name, arguments } = parameter {
        if arguments.is_empty() && type_parameters.iter().any(|parameter| parameter == name) {
            if !typed_type_contains_hole(argument) {
                substitutions
                    .entry(name.clone())
                    .or_insert_with(|| argument.clone());
            }
            return;
        }
    }

    match (parameter, argument) {
        (
            TypedType::Named {
                name: parameter_name,
                arguments: parameter_arguments,
            },
            TypedType::Named {
                name: argument_name,
                arguments: argument_arguments,
            },
        ) if parameter_name == argument_name
            && parameter_arguments.len() == argument_arguments.len() =>
        {
            for (parameter, argument) in parameter_arguments.iter().zip(argument_arguments) {
                infer_type_parameters(parameter, argument, type_parameters, substitutions);
            }
        }
        (
            TypedType::Record {
                closed: parameter_closed,
                fields: parameter_fields,
            },
            TypedType::Record {
                closed: argument_closed,
                fields: argument_fields,
            },
        ) if parameter_closed == argument_closed
            && parameter_fields.len() == argument_fields.len() =>
        {
            for (parameter, argument) in parameter_fields.iter().zip(argument_fields) {
                if parameter.name == argument.name && parameter.optional == argument.optional {
                    infer_type_parameters(
                        &parameter.type_ref,
                        &argument.type_ref,
                        type_parameters,
                        substitutions,
                    );
                }
            }
        }
        (
            TypedType::Tuple {
                elements: parameter_elements,
            },
            TypedType::Tuple {
                elements: argument_elements,
            },
        ) if parameter_elements.len() == argument_elements.len() => {
            for (parameter, argument) in parameter_elements.iter().zip(argument_elements) {
                infer_type_parameters(parameter, argument, type_parameters, substitutions);
            }
        }
        (
            TypedType::Function {
                parameter: parameter_parameter,
                result: parameter_result,
            },
            TypedType::Function {
                parameter: argument_parameter,
                result: argument_result,
            },
        ) => {
            infer_type_parameters(
                parameter_parameter,
                argument_parameter,
                type_parameters,
                substitutions,
            );
            infer_type_parameters(
                parameter_result,
                argument_result,
                type_parameters,
                substitutions,
            );
        }
        _ => {}
    }
}

fn substitute_type_parameters(
    type_ref: &TypedType,
    substitutions: &BTreeMap<String, TypedType>,
) -> TypedType {
    match type_ref {
        TypedType::Named { name, arguments } if arguments.is_empty() => substitutions
            .get(name)
            .cloned()
            .unwrap_or_else(|| type_ref.clone()),
        TypedType::Named { name, arguments } => TypedType::Named {
            name: name.clone(),
            arguments: arguments
                .iter()
                .map(|argument| substitute_type_parameters(argument, substitutions))
                .collect(),
        },
        TypedType::ExternalNamed {
            name,
            canonical,
            arguments,
        } => TypedType::ExternalNamed {
            name: name.clone(),
            canonical: canonical.clone(),
            arguments: arguments
                .iter()
                .map(|argument| substitute_type_parameters(argument, substitutions))
                .collect(),
        },
        TypedType::Hole => TypedType::Hole,
        TypedType::Record { closed, fields } => TypedType::Record {
            closed: *closed,
            fields: fields
                .iter()
                .map(|field| TypedRecordField {
                    name: field.name.clone(),
                    optional: field.optional,
                    type_ref: substitute_type_parameters(&field.type_ref, substitutions),
                })
                .collect(),
        },
        TypedType::Tuple { elements } => TypedType::Tuple {
            elements: elements
                .iter()
                .map(|element| substitute_type_parameters(element, substitutions))
                .collect(),
        },
        TypedType::Function { parameter, result } => TypedType::Function {
            parameter: Box::new(substitute_type_parameters(parameter, substitutions)),
            result: Box::new(substitute_type_parameters(result, substitutions)),
        },
    }
}
