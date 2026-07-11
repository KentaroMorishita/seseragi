use super::{SemanticTypeKey, SemanticValueType};
use crate::{SymbolId, TypedType};
use std::collections::BTreeMap;

pub(super) fn substitute_type_parameters(
    value: &SemanticValueType,
    substitutions: &BTreeMap<SymbolId, SemanticValueType>,
) -> SemanticValueType {
    if let SemanticTypeKey::TypeParameter(parameter) = &value.key {
        return substitutions
            .get(parameter)
            .cloned()
            .unwrap_or_else(|| value.clone());
    }

    match (&value.type_ref, &value.key) {
        (TypedType::Named { name, .. }, SemanticTypeKey::Adt { owner, arguments }) => {
            let arguments = arguments
                .iter()
                .map(|argument| substitute_type_parameters(argument, substitutions))
                .collect::<Vec<_>>();
            SemanticValueType {
                type_ref: TypedType::Named {
                    name: name.clone(),
                    arguments: arguments
                        .iter()
                        .map(|argument| argument.type_ref.clone())
                        .collect(),
                },
                key: SemanticTypeKey::Adt {
                    owner: *owner,
                    arguments,
                },
            }
        }
        (TypedType::Tuple { elements: types }, SemanticTypeKey::Tuple(keys)) => {
            let elements = types
                .iter()
                .zip(keys)
                .map(|(type_ref, key)| {
                    substitute_type_parameters(
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
}

pub(crate) fn instantiate_callable(
    parameter_templates: &[SemanticValueType],
    expected_result: Option<&SemanticValueType>,
    arguments: &[SemanticValueType],
    result_template: &SemanticValueType,
) -> InstantiatedSemanticCallable {
    let mut substitutions = BTreeMap::new();
    if let Some(expected_result) = expected_result {
        collect_substitutions(&result_template.key, expected_result, &mut substitutions);
    }
    for (template, argument) in parameter_templates.iter().zip(arguments) {
        collect_substitutions(&template.key, argument, &mut substitutions);
    }
    InstantiatedSemanticCallable {
        parameters: parameter_templates
            .iter()
            .map(|parameter| substitute_type_parameters(parameter, &substitutions))
            .collect(),
        result: substitute_type_parameters(result_template, &substitutions),
    }
}

fn collect_substitutions(
    template: &SemanticTypeKey,
    actual: &SemanticValueType,
    substitutions: &mut BTreeMap<SymbolId, SemanticValueType>,
) {
    match (template, &actual.key) {
        (SemanticTypeKey::TypeParameter(parameter), _) => {
            substitutions
                .entry(*parameter)
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
