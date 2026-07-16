use crate::{TypedConstraint, TypedRecordField, TypedType};
use seseragi_syntax::TypeParameter;
use std::collections::{BTreeMap, BTreeSet};

use super::{application_result_type_from, TopLevelPureFunction};
use crate::typed::semantic_types::{instantiate_callable, SemanticValueType};
use crate::typed::type_ref::typed_type_contains_hole;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InstantiatedApplication {
    pub(crate) parameters: Vec<SemanticValueType>,
    pub(crate) result: SemanticValueType,
    pub(crate) constraints: Vec<TypedConstraint>,
    pub(crate) constraint_identities: Vec<Option<String>>,
    pub(crate) resolved_type_parameters: BTreeSet<String>,
}

pub(crate) fn instantiated_application(
    signature: &TopLevelPureFunction,
    expected_application: Option<&SemanticValueType>,
    application_argument_count: usize,
    arguments: &[SemanticValueType],
) -> InstantiatedApplication {
    let mut substitutions = BTreeMap::new();
    if let Some(expected_application) = expected_application {
        let remaining = application_result_type_from(
            &signature.parameters,
            signature.result.clone(),
            application_argument_count.min(signature.parameters.len()),
        );
        infer_type_parameters(
            &remaining,
            &expected_application.type_ref,
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
    let expected_result = expected_application.and_then(|expected| {
        expected_final_result(
            expected,
            signature
                .parameters
                .len()
                .saturating_sub(application_argument_count),
        )
    });
    let semantic = instantiate_callable(
        &semantic_parameters,
        expected_result.as_ref(),
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
        constraint_identities: signature.constraint_identities.clone(),
        resolved_type_parameters: substitutions.keys().cloned().collect(),
    }
}

fn expected_final_result(
    expected_application: &SemanticValueType,
    remaining_parameters: usize,
) -> Option<SemanticValueType> {
    let mut type_ref = &expected_application.type_ref;
    for _ in 0..remaining_parameters {
        let TypedType::Function { result, .. } = type_ref else {
            return None;
        };
        type_ref = result;
    }
    Some(SemanticValueType {
        type_ref: type_ref.clone(),
        key: if remaining_parameters == 0 {
            expected_application.key.clone()
        } else {
            crate::typed::semantic_types::SemanticTypeKey::Other
        },
    })
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

pub(crate) fn infer_type_parameters(
    parameter: &TypedType,
    argument: &TypedType,
    type_parameters: &[TypeParameter],
    substitutions: &mut BTreeMap<String, TypedType>,
) {
    if let TypedType::Named { name, arguments } = parameter {
        if arguments.is_empty()
            && type_parameters
                .iter()
                .any(|parameter| parameter.arity == 0 && parameter.name == *name)
        {
            if !typed_type_contains_hole(argument) {
                match substitutions.entry(name.clone()) {
                    std::collections::btree_map::Entry::Vacant(entry) => {
                        entry.insert(argument.clone());
                    }
                    std::collections::btree_map::Entry::Occupied(mut entry)
                        if contains_type_parameter(entry.get(), type_parameters)
                            && !contains_type_parameter(argument, type_parameters) =>
                    {
                        entry.insert(argument.clone());
                    }
                    std::collections::btree_map::Entry::Occupied(_) => {}
                }
            }
            return;
        }

        if let Some(constructor_parameter) = type_parameters
            .iter()
            .find(|type_parameter| type_parameter.arity > 0 && type_parameter.name == *name)
        {
            if arguments.len() == constructor_parameter.arity as usize {
                if let Some((constructor, applied_arguments)) =
                    split_constructor_application(argument, arguments.len())
                {
                    substitutions.entry(name.clone()).or_insert(constructor);
                    for (parameter, argument) in arguments.iter().zip(applied_arguments) {
                        infer_type_parameters(parameter, argument, type_parameters, substitutions);
                    }
                }
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
            TypedType::ExternalNamed {
                canonical: parameter_canonical,
                arguments: parameter_arguments,
                ..
            },
            TypedType::ExternalNamed {
                canonical: argument_canonical,
                arguments: argument_arguments,
                ..
            },
        ) if parameter_canonical == argument_canonical
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

fn split_constructor_application(
    argument: &TypedType,
    application_arity: usize,
) -> Option<(TypedType, &[TypedType])> {
    let argument_count = match argument {
        TypedType::Named { arguments, .. } | TypedType::ExternalNamed { arguments, .. } => {
            arguments.len()
        }
        _ => return None,
    };
    let prefix_len = argument_count.checked_sub(application_arity)?;
    match argument {
        TypedType::Named { name, arguments } => Some((
            TypedType::Named {
                name: name.clone(),
                arguments: arguments[..prefix_len].to_vec(),
            },
            &arguments[prefix_len..],
        )),
        TypedType::ExternalNamed {
            name,
            canonical,
            arguments,
        } => Some((
            TypedType::ExternalNamed {
                name: name.clone(),
                canonical: canonical.clone(),
                arguments: arguments[..prefix_len].to_vec(),
            },
            &arguments[prefix_len..],
        )),
        _ => None,
    }
}

fn contains_type_parameter(type_ref: &TypedType, parameters: &[TypeParameter]) -> bool {
    match type_ref {
        TypedType::Named { name, arguments } => {
            parameters.iter().any(|parameter| parameter.name == *name)
                || arguments
                    .iter()
                    .any(|argument| contains_type_parameter(argument, parameters))
        }
        TypedType::ExternalNamed { arguments, .. } => arguments
            .iter()
            .any(|argument| contains_type_parameter(argument, parameters)),
        TypedType::Record { fields, .. } => fields
            .iter()
            .any(|field| contains_type_parameter(&field.type_ref, parameters)),
        TypedType::Tuple { elements } => elements
            .iter()
            .any(|element| contains_type_parameter(element, parameters)),
        TypedType::Function { parameter, result } => {
            contains_type_parameter(parameter, parameters)
                || contains_type_parameter(result, parameters)
        }
        TypedType::Hole => true,
    }
}

pub(crate) fn substitute_type_parameters(
    type_ref: &TypedType,
    substitutions: &BTreeMap<String, TypedType>,
) -> TypedType {
    match type_ref {
        TypedType::Named { name, arguments } if arguments.is_empty() => substitutions
            .get(name)
            .cloned()
            .unwrap_or_else(|| type_ref.clone()),
        TypedType::Named { name, arguments } => {
            let arguments = arguments
                .iter()
                .map(|argument| substitute_type_parameters(argument, substitutions))
                .collect::<Vec<_>>();
            match substitutions.get(name) {
                Some(TypedType::Named {
                    name: constructor,
                    arguments: existing,
                }) => {
                    let mut applied = existing.clone();
                    applied.extend(arguments);
                    TypedType::Named {
                        name: constructor.clone(),
                        arguments: applied,
                    }
                }
                Some(TypedType::ExternalNamed {
                    name: constructor,
                    canonical,
                    arguments: existing,
                }) => {
                    let mut applied = existing.clone();
                    applied.extend(arguments);
                    TypedType::ExternalNamed {
                        name: constructor.clone(),
                        canonical: canonical.clone(),
                        arguments: applied,
                    }
                }
                _ => TypedType::Named {
                    name: name.clone(),
                    arguments,
                },
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn named(name: &str, arguments: Vec<TypedType>) -> TypedType {
        TypedType::Named {
            name: name.to_owned(),
            arguments,
        }
    }

    #[test]
    fn infers_a_partially_applied_constructor_from_its_fixed_prefix() {
        let mut substitutions = BTreeMap::new();
        let parameters = vec![
            TypeParameter {
                name: "M".to_owned(),
                arity: 1,
            },
            TypeParameter {
                name: "A".to_owned(),
                arity: 0,
            },
        ];

        infer_type_parameters(
            &named("M", vec![named("A", Vec::new())]),
            &named(
                "Either",
                vec![named("String", Vec::new()), named("Int", Vec::new())],
            ),
            &parameters,
            &mut substitutions,
        );

        assert_eq!(
            substitutions.get("M"),
            Some(&named("Either", vec![named("String", Vec::new())]))
        );
        assert_eq!(substitutions.get("A"), Some(&named("Int", Vec::new())));
    }

    #[test]
    fn reapplies_an_inferred_constructor_after_its_fixed_prefix() {
        let substitutions = BTreeMap::from([(
            "M".to_owned(),
            named("Either", vec![named("String", Vec::new())]),
        )]);

        assert_eq!(
            substitute_type_parameters(&named("M", vec![named("A", Vec::new())]), &substitutions,),
            named(
                "Either",
                vec![named("String", Vec::new()), named("A", Vec::new())],
            )
        );
    }

    #[test]
    fn infers_a_constructor_from_the_expected_partial_function_type() {
        let signature = TopLevelPureFunction {
            symbol: "map".to_owned(),
            trait_identity: Some("fixture::Functor".to_owned()),
            trait_method: Some("map".to_owned()),
            type_parameters: vec![
                TypeParameter {
                    name: "F".to_owned(),
                    arity: 1,
                },
                TypeParameter {
                    name: "A".to_owned(),
                    arity: 0,
                },
                TypeParameter {
                    name: "B".to_owned(),
                    arity: 0,
                },
            ],
            constraints: vec![TypedConstraint {
                name: "Functor".to_owned(),
                arguments: vec![named("F", Vec::new())],
            }],
            constraint_identities: vec![Some("fixture::Functor".to_owned())],
            parameters: vec![
                TypedType::Function {
                    parameter: Box::new(named("A", Vec::new())),
                    result: Box::new(named("B", Vec::new())),
                },
                named("F", vec![named("A", Vec::new())]),
            ],
            semantic_parameters: vec![
                crate::typed::semantic_types::SemanticTypeKey::Other,
                crate::typed::semantic_types::SemanticTypeKey::Other,
            ],
            result: named("F", vec![named("B", Vec::new())]),
            semantic_result: crate::typed::semantic_types::SemanticTypeKey::Other,
        };
        let int = named("Int", Vec::new());
        let maybe_int = named("Maybe", vec![int.clone()]);
        let expected = SemanticValueType {
            type_ref: TypedType::Function {
                parameter: Box::new(maybe_int.clone()),
                result: Box::new(maybe_int.clone()),
            },
            key: crate::typed::semantic_types::SemanticTypeKey::Other,
        };
        let increment = SemanticValueType {
            type_ref: TypedType::Function {
                parameter: Box::new(int.clone()),
                result: Box::new(int),
            },
            key: crate::typed::semantic_types::SemanticTypeKey::Other,
        };

        let application = instantiated_application(&signature, Some(&expected), 1, &[increment]);

        assert_eq!(application.parameters[1].type_ref, maybe_int.clone());
        assert_eq!(application.result.type_ref, maybe_int);
        assert_eq!(
            application.constraints[0].arguments,
            vec![named("Maybe", Vec::new())]
        );
    }
}
