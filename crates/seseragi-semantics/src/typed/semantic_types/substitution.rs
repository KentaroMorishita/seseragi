use super::{SemanticTypeKey, SemanticValueType};
use crate::{SymbolId, TypedType};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum TypeParameterKey {
    Resolved(SymbolId),
    Scheme(String),
}

pub(super) fn substitute_type_parameters(
    value: &SemanticValueType,
    substitutions: &BTreeMap<SymbolId, SemanticValueType>,
) -> SemanticValueType {
    let substitutions = substitutions
        .iter()
        .map(|(parameter, value)| (TypeParameterKey::Resolved(*parameter), value.clone()))
        .collect();
    substitute_semantic_type_parameters(value, &substitutions)
}

pub(crate) fn substitute_remaining_scheme_parameters(
    value: &SemanticValueType,
    substitutions: &BTreeMap<String, TypedType>,
) -> SemanticValueType {
    let substitutions = substitutions
        .iter()
        .map(|(parameter, type_ref)| {
            (
                TypeParameterKey::Scheme(parameter.clone()),
                SemanticValueType {
                    type_ref: type_ref.clone(),
                    key: SemanticTypeKey::Other,
                },
            )
        })
        .collect();
    substitute_semantic_type_parameters(value, &substitutions)
}

fn substitute_semantic_type_parameters(
    value: &SemanticValueType,
    substitutions: &BTreeMap<TypeParameterKey, SemanticValueType>,
) -> SemanticValueType {
    if let SemanticTypeKey::Application {
        constructor,
        arguments,
    } = &value.key
    {
        let constructor = substitute_semantic_type_parameters(constructor, substitutions);
        let arguments = arguments
            .iter()
            .map(|argument| substitute_semantic_type_parameters(argument, substitutions))
            .collect::<Vec<_>>();
        let mut type_ref = constructor.type_ref.clone();
        match &mut type_ref {
            TypedType::Named {
                arguments: existing,
                ..
            }
            | TypedType::ExternalNamed {
                arguments: existing,
                ..
            } => {
                *existing = super::fill_constructor_holes(
                    existing,
                    arguments
                        .iter()
                        .map(|argument| argument.type_ref.clone())
                        .collect(),
                )
            }
            _ => return value.clone(),
        }
        let key = match constructor.key.clone() {
            SemanticTypeKey::Adt {
                owner,
                arguments: mut existing,
            } => {
                existing = fill_semantic_constructor_holes(existing, arguments);
                SemanticTypeKey::Adt {
                    owner,
                    arguments: existing,
                }
            }
            SemanticTypeKey::Struct {
                owner,
                arguments: mut existing,
            } => {
                existing = fill_semantic_constructor_holes(existing, arguments);
                SemanticTypeKey::Struct {
                    owner,
                    arguments: existing,
                }
            }
            SemanticTypeKey::ExternalNominal {
                canonical,
                arguments: mut existing,
            } => {
                existing = fill_semantic_constructor_holes(existing, arguments);
                SemanticTypeKey::ExternalNominal {
                    canonical,
                    arguments: existing,
                }
            }
            SemanticTypeKey::NamedGeneric {
                name,
                arguments: mut existing,
            } => {
                existing = fill_semantic_constructor_holes(existing, arguments);
                SemanticTypeKey::NamedGeneric {
                    name,
                    arguments: existing,
                }
            }
            SemanticTypeKey::TypeParameter(_) | SemanticTypeKey::SchemeParameter(_) => {
                SemanticTypeKey::Application {
                    constructor: Box::new(constructor),
                    arguments,
                }
            }
            _ => SemanticTypeKey::Other,
        };
        return SemanticValueType { type_ref, key };
    }
    let parameter = match &value.key {
        SemanticTypeKey::TypeParameter(parameter) => Some(TypeParameterKey::Resolved(*parameter)),
        SemanticTypeKey::SchemeParameter(parameter) => {
            Some(TypeParameterKey::Scheme(parameter.clone()))
        }
        _ => None,
    };
    if let Some(parameter) = parameter {
        return substitutions
            .get(&parameter)
            .cloned()
            .unwrap_or_else(|| value.clone());
    }

    match (&value.type_ref, &value.key) {
        (TypedType::Named { name, .. }, SemanticTypeKey::Adt { owner, arguments })
        | (TypedType::Named { name, .. }, SemanticTypeKey::Struct { owner, arguments }) => {
            let arguments = arguments
                .iter()
                .map(|argument| substitute_semantic_type_parameters(argument, substitutions))
                .collect::<Vec<_>>();
            SemanticValueType {
                type_ref: TypedType::Named {
                    name: name.clone(),
                    arguments: arguments
                        .iter()
                        .map(|argument| argument.type_ref.clone())
                        .collect(),
                },
                key: match &value.key {
                    SemanticTypeKey::Struct { .. } => SemanticTypeKey::Struct {
                        owner: *owner,
                        arguments,
                    },
                    _ => SemanticTypeKey::Adt {
                        owner: *owner,
                        arguments,
                    },
                },
            }
        }
        (
            TypedType::ExternalNamed {
                name, canonical, ..
            },
            SemanticTypeKey::Adt { owner, arguments },
        )
        | (
            TypedType::ExternalNamed {
                name, canonical, ..
            },
            SemanticTypeKey::Struct { owner, arguments },
        ) => {
            let arguments = arguments
                .iter()
                .map(|argument| substitute_semantic_type_parameters(argument, substitutions))
                .collect::<Vec<_>>();
            SemanticValueType {
                type_ref: TypedType::ExternalNamed {
                    name: name.clone(),
                    canonical: canonical.clone(),
                    arguments: arguments
                        .iter()
                        .map(|argument| argument.type_ref.clone())
                        .collect(),
                },
                key: match &value.key {
                    SemanticTypeKey::Struct { .. } => SemanticTypeKey::Struct {
                        owner: *owner,
                        arguments,
                    },
                    _ => SemanticTypeKey::Adt {
                        owner: *owner,
                        arguments,
                    },
                },
            }
        }
        (
            TypedType::Named { name, .. },
            SemanticTypeKey::ExternalNominal {
                canonical,
                arguments,
            },
        ) => {
            let arguments = arguments
                .iter()
                .map(|argument| substitute_semantic_type_parameters(argument, substitutions))
                .collect::<Vec<_>>();
            SemanticValueType {
                type_ref: TypedType::Named {
                    name: name.clone(),
                    arguments: arguments
                        .iter()
                        .map(|argument| argument.type_ref.clone())
                        .collect(),
                },
                key: SemanticTypeKey::ExternalNominal {
                    canonical: canonical.clone(),
                    arguments,
                },
            }
        }
        (
            TypedType::Named { name, .. },
            SemanticTypeKey::NamedGeneric {
                name: key_name,
                arguments,
            },
        ) if name == key_name => {
            let arguments = arguments
                .iter()
                .map(|argument| substitute_semantic_type_parameters(argument, substitutions))
                .collect::<Vec<_>>();
            SemanticValueType {
                type_ref: TypedType::Named {
                    name: name.clone(),
                    arguments: arguments
                        .iter()
                        .map(|argument| argument.type_ref.clone())
                        .collect(),
                },
                key: SemanticTypeKey::NamedGeneric {
                    name: key_name.clone(),
                    arguments,
                },
            }
        }
        (
            TypedType::ExternalNamed {
                name, canonical, ..
            },
            SemanticTypeKey::ExternalNominal {
                canonical: key_canonical,
                arguments,
            },
        ) if canonical == key_canonical => {
            let arguments = arguments
                .iter()
                .map(|argument| substitute_semantic_type_parameters(argument, substitutions))
                .collect::<Vec<_>>();
            SemanticValueType {
                type_ref: TypedType::ExternalNamed {
                    name: name.clone(),
                    canonical: canonical.clone(),
                    arguments: arguments
                        .iter()
                        .map(|argument| argument.type_ref.clone())
                        .collect(),
                },
                key: SemanticTypeKey::ExternalNominal {
                    canonical: key_canonical.clone(),
                    arguments,
                },
            }
        }
        (TypedType::Record { closed, fields }, SemanticTypeKey::Record(keys)) => {
            let values = keys
                .iter()
                .map(|(name, value)| {
                    (
                        name.clone(),
                        substitute_semantic_type_parameters(value, substitutions),
                    )
                })
                .collect::<std::collections::BTreeMap<_, _>>();
            SemanticValueType {
                type_ref: TypedType::Record {
                    closed: *closed,
                    fields: fields
                        .iter()
                        .map(|field| crate::TypedRecordField {
                            name: field.name.clone(),
                            optional: field.optional,
                            type_ref: values.get(&field.name).map_or_else(
                                || field.type_ref.clone(),
                                |value| value.type_ref.clone(),
                            ),
                        })
                        .collect(),
                },
                key: SemanticTypeKey::Record(values),
            }
        }
        (TypedType::Tuple { elements: types }, SemanticTypeKey::Tuple(keys)) => {
            let elements = types
                .iter()
                .zip(keys)
                .map(|(type_ref, key)| {
                    substitute_semantic_type_parameters(
                        &SemanticValueType {
                            type_ref: type_ref.clone(),
                            key: key.clone(),
                        },
                        substitutions,
                    )
                })
                .collect::<Vec<_>>();
            SemanticValueType {
                type_ref: TypedType::Tuple {
                    elements: elements
                        .iter()
                        .map(|element| element.type_ref.clone())
                        .collect(),
                },
                key: SemanticTypeKey::Tuple(
                    elements.into_iter().map(|element| element.key).collect(),
                ),
            }
        }
        _ => value.clone(),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InstantiatedSemanticCallable {
    pub(crate) parameters: Vec<SemanticValueType>,
    pub(crate) result: SemanticValueType,
    pub(crate) parameter_bindings: Vec<(TypedType, SemanticValueType)>,
}

pub(crate) fn instantiate_callable(
    parameter_templates: &[SemanticValueType],
    expected_result: Option<&SemanticValueType>,
    arguments: &[SemanticValueType],
    result_template: &SemanticValueType,
) -> InstantiatedSemanticCallable {
    let indexed_arguments = arguments.iter().cloned().enumerate().collect::<Vec<_>>();
    instantiate_callable_indexed(
        parameter_templates,
        expected_result,
        &indexed_arguments,
        result_template,
    )
}

pub(crate) fn instantiate_callable_indexed(
    parameter_templates: &[SemanticValueType],
    expected_result: Option<&SemanticValueType>,
    arguments: &[(usize, SemanticValueType)],
    result_template: &SemanticValueType,
) -> InstantiatedSemanticCallable {
    let mut substitutions = BTreeMap::new();
    if let Some(expected_result) = expected_result {
        collect_substitutions(&result_template.key, expected_result, &mut substitutions);
    }
    for (index, argument) in arguments {
        if let Some(template) = parameter_templates.get(*index) {
            collect_substitutions(&template.key, argument, &mut substitutions);
        }
    }
    let mut parameter_bindings = Vec::new();
    for template in parameter_templates
        .iter()
        .chain(std::iter::once(result_template))
    {
        collect_parameter_bindings(template, &substitutions, &mut parameter_bindings);
    }
    InstantiatedSemanticCallable {
        parameters: parameter_templates
            .iter()
            .map(|parameter| substitute_semantic_type_parameters(parameter, &substitutions))
            .collect(),
        result: substitute_semantic_type_parameters(result_template, &substitutions),
        parameter_bindings,
    }
}

/// Retain the source variable alongside each identity-checked substitution.
/// Reconstructing this information from an instantiated display type would
/// mistake unchanged variables for inferred ones, or confuse scoped binders.
fn collect_parameter_bindings(
    template: &SemanticValueType,
    substitutions: &BTreeMap<TypeParameterKey, SemanticValueType>,
    bindings: &mut Vec<(TypedType, SemanticValueType)>,
) {
    let parameter = match &template.key {
        SemanticTypeKey::TypeParameter(parameter) => Some(TypeParameterKey::Resolved(*parameter)),
        SemanticTypeKey::SchemeParameter(parameter) => {
            Some(TypeParameterKey::Scheme(parameter.clone()))
        }
        _ => None,
    };
    if let Some(parameter) = parameter {
        if let Some(inferred) = substitutions.get(&parameter) {
            // An expected collection can still carry an uninstantiated
            // imported scheme behind a masked display hole. It does not make
            // a call parameter resolved. Lexical TypeParameter IDs, however,
            // remain valid evidence when forwarding a generic caller's value.
            if inferred != template && !contains_scheme_parameter(&inferred.key) {
                bindings.push((template.type_ref.clone(), inferred.clone()));
            }
        }
        return;
    }
    match &template.key {
        SemanticTypeKey::Application {
            constructor,
            arguments,
        } => {
            collect_parameter_bindings(constructor, substitutions, bindings);
            for argument in arguments {
                collect_parameter_bindings(argument, substitutions, bindings);
            }
        }
        SemanticTypeKey::Adt { arguments, .. }
        | SemanticTypeKey::Struct { arguments, .. }
        | SemanticTypeKey::ExternalNominal { arguments, .. }
        | SemanticTypeKey::NamedGeneric { arguments, .. } => {
            for argument in arguments {
                collect_parameter_bindings(argument, substitutions, bindings);
            }
        }
        SemanticTypeKey::Record(fields) => {
            for field in fields.values() {
                collect_parameter_bindings(field, substitutions, bindings);
            }
        }
        SemanticTypeKey::Tuple(keys) => {
            if let TypedType::Tuple { elements } = &template.type_ref {
                for (type_ref, key) in elements.iter().zip(keys) {
                    collect_parameter_bindings(
                        &SemanticValueType {
                            type_ref: type_ref.clone(),
                            key: key.clone(),
                        },
                        substitutions,
                        bindings,
                    );
                }
            }
        }
        _ => {}
    }
}

fn contains_scheme_parameter(key: &SemanticTypeKey) -> bool {
    match key {
        SemanticTypeKey::SchemeParameter(_) => true,
        SemanticTypeKey::Application {
            constructor,
            arguments,
        } => {
            contains_scheme_parameter(&constructor.key)
                || arguments
                    .iter()
                    .any(|argument| contains_scheme_parameter(&argument.key))
        }
        SemanticTypeKey::Adt { arguments, .. }
        | SemanticTypeKey::Struct { arguments, .. }
        | SemanticTypeKey::ExternalNominal { arguments, .. }
        | SemanticTypeKey::NamedGeneric { arguments, .. } => arguments
            .iter()
            .any(|argument| contains_scheme_parameter(&argument.key)),
        SemanticTypeKey::Tuple(keys) => keys.iter().any(contains_scheme_parameter),
        SemanticTypeKey::Record(fields) => fields
            .values()
            .any(|field| contains_scheme_parameter(&field.key)),
        _ => false,
    }
}

fn collect_substitutions(
    template: &SemanticTypeKey,
    actual: &SemanticValueType,
    substitutions: &mut BTreeMap<TypeParameterKey, SemanticValueType>,
) {
    match (template, &actual.key) {
        (
            SemanticTypeKey::Application {
                constructor,
                arguments: templates,
            },
            _,
        ) => {
            let (mut head, types) = match &actual.type_ref {
                TypedType::Named { name, arguments } => (
                    TypedType::Named {
                        name: name.clone(),
                        arguments: vec![],
                    },
                    arguments,
                ),
                TypedType::ExternalNamed {
                    name,
                    canonical,
                    arguments,
                } => (
                    TypedType::ExternalNamed {
                        name: name.clone(),
                        canonical: canonical.clone(),
                        arguments: vec![],
                    },
                    arguments,
                ),
                _ => return,
            };
            let Some(prefix) = types.len().checked_sub(templates.len()) else {
                return;
            };
            match &mut head {
                TypedType::Named { arguments, .. } | TypedType::ExternalNamed { arguments, .. } => {
                    *arguments = types[..prefix].to_vec()
                }
                _ => unreachable!(),
            }
            let keys = match &actual.key {
                SemanticTypeKey::Adt { arguments, .. }
                | SemanticTypeKey::Struct { arguments, .. }
                | SemanticTypeKey::ExternalNominal { arguments, .. }
                | SemanticTypeKey::NamedGeneric { arguments, .. } => Some(arguments),
                _ => None,
            };
            let mut head_key = actual.key.clone();
            match &mut head_key {
                SemanticTypeKey::Adt { arguments, .. }
                | SemanticTypeKey::Struct { arguments, .. }
                | SemanticTypeKey::ExternalNominal { arguments, .. }
                | SemanticTypeKey::NamedGeneric { arguments, .. } => arguments.truncate(prefix),
                _ => head_key = SemanticTypeKey::Other,
            }
            collect_substitutions(
                &constructor.key,
                &SemanticValueType {
                    type_ref: head,
                    key: head_key,
                },
                substitutions,
            );
            for (offset, template) in templates.iter().enumerate() {
                let index = prefix + offset;
                let argument = keys
                    .and_then(|keys| keys.get(index))
                    .cloned()
                    .unwrap_or_else(|| SemanticValueType {
                        type_ref: types[index].clone(),
                        key: SemanticTypeKey::Other,
                    });
                collect_substitutions(&template.key, &argument, substitutions);
            }
        }
        (SemanticTypeKey::TypeParameter(parameter), _) => {
            substitutions
                .entry(TypeParameterKey::Resolved(*parameter))
                .or_insert_with(|| actual.clone());
        }
        (SemanticTypeKey::SchemeParameter(parameter), _) => {
            substitutions
                .entry(TypeParameterKey::Scheme(parameter.clone()))
                .or_insert_with(|| actual.clone());
        }
        (
            SemanticTypeKey::Adt {
                owner: template_owner,
                arguments: templates,
            },
            SemanticTypeKey::Adt {
                owner: actual_owner,
                arguments,
            },
        ) if template_owner == actual_owner && templates.len() == arguments.len() => {
            for (template, argument) in templates.iter().zip(arguments) {
                collect_substitutions(&template.key, argument, substitutions);
            }
        }
        (
            SemanticTypeKey::Struct {
                owner: template_owner,
                arguments: templates,
            },
            SemanticTypeKey::Struct {
                owner: actual_owner,
                arguments,
            },
        ) if template_owner == actual_owner && templates.len() == arguments.len() => {
            for (template, argument) in templates.iter().zip(arguments) {
                collect_substitutions(&template.key, argument, substitutions);
            }
        }
        (
            SemanticTypeKey::ExternalNominal {
                canonical: template_canonical,
                arguments: templates,
            },
            SemanticTypeKey::ExternalNominal {
                canonical: actual_canonical,
                arguments,
            },
        ) if template_canonical == actual_canonical && templates.len() == arguments.len() => {
            for (template, argument) in templates.iter().zip(arguments) {
                collect_substitutions(&template.key, argument, substitutions);
            }
        }
        (
            SemanticTypeKey::NamedGeneric {
                name: template_name,
                arguments: templates,
            },
            SemanticTypeKey::NamedGeneric {
                name: actual_name,
                arguments,
            },
        ) if template_name == actual_name && templates.len() == arguments.len() => {
            for (template, argument) in templates.iter().zip(arguments) {
                collect_substitutions(&template.key, argument, substitutions);
            }
        }
        (SemanticTypeKey::Record(templates), SemanticTypeKey::Record(actuals)) => {
            for (name, template) in templates {
                if let Some(actual) = actuals.get(name) {
                    collect_substitutions(&template.key, actual, substitutions);
                }
            }
        }
        (SemanticTypeKey::Tuple(templates), SemanticTypeKey::Tuple(actuals))
            if templates.len() == actuals.len() =>
        {
            let TypedType::Tuple { elements } = &actual.type_ref else {
                return;
            };
            for ((template, actual), type_ref) in templates.iter().zip(actuals).zip(elements) {
                collect_substitutions(
                    template,
                    &SemanticValueType {
                        type_ref: type_ref.clone(),
                        key: actual.clone(),
                    },
                    substitutions,
                );
            }
        }
        _ => {}
    }
}

fn fill_semantic_constructor_holes(
    existing: Vec<SemanticValueType>,
    arguments: Vec<SemanticValueType>,
) -> Vec<SemanticValueType> {
    let mut incoming = arguments.into_iter();
    let mut result = existing
        .into_iter()
        .map(|value| {
            if matches!(value.type_ref, TypedType::Hole) {
                incoming.next().unwrap_or(value)
            } else {
                value
            }
        })
        .collect::<Vec<_>>();
    result.extend(incoming);
    result
}
