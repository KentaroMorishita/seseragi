use super::{SemanticTypeKey, SemanticValueType};
use crate::{SymbolId, TypedType};
use std::collections::{BTreeMap, BTreeSet};

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
        // A display-only self binding supplies no new identity. Keeping its
        // checked slot is essential when a let scheme is saved as a value;
        // replacing it by Other would prevent later semantic instantiation.
        .filter(|(parameter, type_ref)| {
            !matches!(type_ref, TypedType::Named { name, arguments }
                if name == *parameter && arguments.is_empty())
        })
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
        if matches!(constructor.type_ref, TypedType::TypeFunction { .. }) {
            return seseragi_syntax::type_functions::apply_type_function(&constructor, &arguments)
                .unwrap_or_else(|| SemanticValueType {
                    type_ref: TypedType::Hole,
                    key: SemanticTypeKey::Invalid,
                });
        }
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
        (
            TypedType::TypeFunction {
                parameter_kinds, ..
            },
            SemanticTypeKey::TypeFunction { body, .. },
        ) => {
            use seseragi_syntax::type_functions::TypeFunctionTerm;
            SemanticValueType::from_type_function(
                parameter_kinds.clone(),
                substitute_semantic_type_parameters(body, substitutions),
            )
        }
        (TypedType::Function { .. }, SemanticTypeKey::Function { parameter, result }) => {
            SemanticValueType::function(
                substitute_semantic_type_parameters(parameter, substitutions),
                substitute_semantic_type_parameters(result, substitutions),
            )
        }
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
    instantiate_callable_indexed_with_display_types(
        parameter_templates,
        expected_result,
        arguments,
        result_template,
        None,
        arguments.len(),
    )
}

pub(crate) fn instantiate_callable_indexed_with_display_types(
    parameter_templates: &[SemanticValueType],
    expected_result: Option<&SemanticValueType>,
    arguments: &[(usize, SemanticValueType)],
    result_template: &SemanticValueType,
    display_types: Option<&BTreeMap<String, TypedType>>,
    source_argument_count: usize,
) -> InstantiatedSemanticCallable {
    assert!(source_argument_count <= arguments.len());
    let mut collected = CallableSubstitutions {
        values: BTreeMap::new(),
        source_bindings: BTreeSet::new(),
        source_argument: false,
        display_types,
    };
    if let Some(expected_result) = expected_result {
        collect_substitutions(result_template, expected_result, &mut collected);
    }
    for (position, (index, argument)) in arguments.iter().enumerate() {
        collected.source_argument = position < source_argument_count;
        if let Some(template) = parameter_templates.get(*index) {
            collect_substitutions(template, argument, &mut collected);
        }
    }
    let substitutions = collected.values;
    let mut parameter_bindings = Vec::new();
    for template in parameter_templates
        .iter()
        .chain(std::iter::once(result_template))
    {
        collect_parameter_bindings(
            template,
            &substitutions,
            &collected.source_bindings,
            &mut parameter_bindings,
        );
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

struct CallableSubstitutions<'a> {
    values: BTreeMap<TypeParameterKey, SemanticValueType>,
    source_bindings: BTreeSet<TypeParameterKey>,
    source_argument: bool,
    display_types: Option<&'a BTreeMap<String, TypedType>>,
}

impl CallableSubstitutions<'_> {
    fn bind(
        &mut self,
        parameter: TypeParameterKey,
        template: &SemanticValueType,
        actual: &SemanticValueType,
    ) {
        let selected = match &template.type_ref {
            TypedType::Named { name, arguments } if arguments.is_empty() => {
                self.display_types.and_then(|types| types.get(name))
            }
            _ => None,
        };
        match self.values.entry(parameter.clone()) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                if self.source_argument {
                    self.source_bindings.insert(parameter);
                }
                entry.insert(actual.clone());
            }
            std::collections::btree_map::Entry::Occupied(mut entry)
                if selected.is_some_and(|selected| {
                    actual.type_ref == *selected && entry.get().type_ref != *selected
                }) =>
            {
                // Keep the checked candidate that display inference selected.
                // The display type is only a selection hint, never owner or
                // binder evidence. When two candidates share a display type,
                // preserve the contextual identity and check the argument.
                if self.source_argument {
                    self.source_bindings.insert(parameter);
                } else {
                    self.source_bindings.remove(&parameter);
                }
                entry.insert(actual.clone());
            }
            _ => {}
        }
    }
}

/// Retain the source variable alongside each identity-checked substitution.
/// Reconstructing this information from an instantiated display type would
/// mistake unchanged variables for inferred ones, or confuse scoped binders.
fn collect_parameter_bindings(
    template: &SemanticValueType,
    substitutions: &BTreeMap<TypeParameterKey, SemanticValueType>,
    source_bindings: &BTreeSet<TypeParameterKey>,
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
            // Uninstantiated contextual schemes cannot resolve a call. A
            // distinct caller let scheme selected from a checked source
            // argument can: retain its slot instead of recognizing its name.
            if inferred != template
                && (!contains_scheme_parameter(&inferred.key)
                    || source_bindings.contains(&parameter))
            {
                bindings.push((template.type_ref.clone(), inferred.clone()));
            }
        }
        return;
    }
    match &template.key {
        SemanticTypeKey::Function { parameter, result } => {
            collect_parameter_bindings(parameter, substitutions, source_bindings, bindings);
            collect_parameter_bindings(result, substitutions, source_bindings, bindings);
        }
        SemanticTypeKey::Application {
            constructor,
            arguments,
        } => {
            collect_parameter_bindings(constructor, substitutions, source_bindings, bindings);
            for argument in arguments {
                collect_parameter_bindings(argument, substitutions, source_bindings, bindings);
            }
        }
        SemanticTypeKey::Adt { arguments, .. }
        | SemanticTypeKey::Struct { arguments, .. }
        | SemanticTypeKey::ExternalNominal { arguments, .. }
        | SemanticTypeKey::NamedGeneric { arguments, .. } => {
            for argument in arguments {
                collect_parameter_bindings(argument, substitutions, source_bindings, bindings);
            }
        }
        SemanticTypeKey::Record(fields) => {
            for field in fields.values() {
                collect_parameter_bindings(field, substitutions, source_bindings, bindings);
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
                        source_bindings,
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
        SemanticTypeKey::Function { parameter, result } => {
            contains_scheme_parameter(&parameter.key) || contains_scheme_parameter(&result.key)
        }
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
    template: &SemanticValueType,
    actual: &SemanticValueType,
    substitutions: &mut CallableSubstitutions<'_>,
) {
    match (&template.key, &actual.key) {
        (SemanticTypeKey::Function { parameter, result }, _) => {
            if let Some((actual_parameter, actual_result)) = actual.function_parts() {
                collect_substitutions(parameter, &actual_parameter, substitutions);
                collect_substitutions(result, &actual_result, substitutions);
            }
        }
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
                | SemanticTypeKey::NamedGeneric { arguments, .. } => Some(arguments.clone()),
                SemanticTypeKey::Application { .. } => applied_argument_values(actual),
                _ => None,
            };
            let mut head_key = actual.key.clone();
            match &mut head_key {
                SemanticTypeKey::Adt { arguments, .. }
                | SemanticTypeKey::Struct { arguments, .. }
                | SemanticTypeKey::ExternalNominal { arguments, .. }
                | SemanticTypeKey::NamedGeneric { arguments, .. } => arguments.truncate(prefix),
                SemanticTypeKey::Application {
                    constructor,
                    arguments,
                } => {
                    let fixed = match &constructor.type_ref {
                        TypedType::Named { arguments, .. }
                        | TypedType::ExternalNamed { arguments, .. } => arguments.len(),
                        _ => return,
                    };
                    let Some(applied_prefix) = prefix.checked_sub(fixed) else {
                        return;
                    };
                    if applied_prefix > arguments.len() {
                        return;
                    }
                    head_key = if applied_prefix == 0 {
                        constructor.key.clone()
                    } else {
                        SemanticTypeKey::Application {
                            constructor: constructor.clone(),
                            arguments: arguments[..applied_prefix].to_vec(),
                        }
                    };
                }
                _ => head_key = SemanticTypeKey::Other,
            }
            collect_substitutions(
                constructor,
                &SemanticValueType {
                    type_ref: head,
                    key: head_key,
                },
                substitutions,
            );
            for (offset, template) in templates.iter().enumerate() {
                let index = prefix + offset;
                let argument = keys
                    .as_ref()
                    .and_then(|keys| keys.get(index))
                    .cloned()
                    .unwrap_or_else(|| SemanticValueType {
                        type_ref: types[index].clone(),
                        key: SemanticTypeKey::Other,
                    });
                collect_substitutions(template, &argument, substitutions);
            }
        }
        (SemanticTypeKey::TypeParameter(parameter), _) => {
            substitutions.bind(TypeParameterKey::Resolved(*parameter), template, actual);
        }
        (SemanticTypeKey::SchemeParameter(parameter), _) => {
            substitutions.bind(
                TypeParameterKey::Scheme(parameter.clone()),
                template,
                actual,
            );
        }
        (
            SemanticTypeKey::Adt {
                arguments: templates,
                ..
            }
            | SemanticTypeKey::Struct {
                arguments: templates,
                ..
            },
            SemanticTypeKey::ExternalNominal {
                canonical,
                arguments,
            },
        )
        | (
            SemanticTypeKey::ExternalNominal {
                canonical,
                arguments: templates,
            },
            SemanticTypeKey::Adt { arguments, .. } | SemanticTypeKey::Struct { arguments, .. },
        ) if templates.len() == arguments.len()
            && matches!((&template.type_ref, &actual.type_ref),
                (TypedType::ExternalNamed { canonical: left, .. },
                 TypedType::ExternalNamed { canonical: right, .. })
                    if left == canonical && right == canonical) =>
        {
            // An imported nominal can have its checked local owner in the
            // signature and its canonical external owner in a stored value.
            // Match those explicit owners before retaining its argument keys;
            // falling back to display-type substitution loses function slots.
            let TypedType::ExternalNamed {
                arguments: types, ..
            } = &actual.type_ref
            else {
                unreachable!("canonical external type was checked above");
            };
            if types.len() != arguments.len() {
                return;
            }
            for ((template, argument), type_ref) in templates.iter().zip(arguments).zip(types) {
                // Contextual inference masks unresolved slots in the display
                // type while retaining their old scheme keys. Such a slot
                // cannot bind the result before an actual source argument.
                if matches!(type_ref, TypedType::Hole) {
                    continue;
                }
                collect_substitutions(
                    template,
                    &SemanticValueType {
                        type_ref: type_ref.clone(),
                        key: argument.key.clone(),
                    },
                    substitutions,
                );
            }
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
                collect_substitutions(template, argument, substitutions);
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
                collect_substitutions(template, argument, substitutions);
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
                collect_substitutions(template, argument, substitutions);
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
                collect_substitutions(template, argument, substitutions);
            }
        }
        (SemanticTypeKey::Record(templates), SemanticTypeKey::Record(actuals)) => {
            for (name, template) in templates {
                if let Some(actual) = actuals.get(name) {
                    collect_substitutions(template, actual, substitutions);
                }
            }
        }
        (SemanticTypeKey::Tuple(templates), SemanticTypeKey::Tuple(actuals))
            if templates.len() == actuals.len() =>
        {
            let (
                TypedType::Tuple {
                    elements: template_types,
                },
                TypedType::Tuple { elements },
            ) = (&template.type_ref, &actual.type_ref)
            else {
                return;
            };
            for (((template, template_type), actual), type_ref) in templates
                .iter()
                .zip(template_types)
                .zip(actuals)
                .zip(elements)
            {
                collect_substitutions(
                    &SemanticValueType {
                        type_ref: template_type.clone(),
                        key: template.clone(),
                    },
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

/// Preserve the checked identities of constructor applications, including an
/// already fixed prefix. A constructor binder is not a same-spelled primitive.
fn applied_argument_values(value: &SemanticValueType) -> Option<Vec<SemanticValueType>> {
    let types = match &value.type_ref {
        TypedType::Named { arguments, .. } | TypedType::ExternalNamed { arguments, .. } => {
            arguments
        }
        _ => return None,
    };
    let values = match &value.key {
        SemanticTypeKey::Adt { arguments, .. }
        | SemanticTypeKey::Struct { arguments, .. }
        | SemanticTypeKey::ExternalNominal { arguments, .. }
        | SemanticTypeKey::NamedGeneric { arguments, .. } => arguments.clone(),
        SemanticTypeKey::Application {
            constructor,
            arguments,
        } => {
            let mut values = applied_argument_values(constructor)?;
            values.extend(arguments.iter().cloned());
            values
        }
        _ if types.is_empty() => Vec::new(),
        _ => return None,
    };
    (values.len() == types.len()).then_some(values)
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

#[cfg(test)]
mod tests {
    use super::{
        instantiate_callable, instantiate_callable_indexed_with_display_types,
        substitute_remaining_scheme_parameters,
    };
    use crate::typed::semantic_types::{SemanticTypeKey, SemanticValueType};
    use crate::{SymbolId, TypedType};
    use std::collections::BTreeMap;

    #[test]
    fn unchanged_display_substitution_preserves_a_checked_scheme_slot() {
        let slot = SemanticValueType {
            type_ref: TypedType::Named {
                name: "A".to_owned(),
                arguments: vec![],
            },
            key: SemanticTypeKey::SchemeParameter("A".to_owned()),
        };
        let nominal = SemanticValueType {
            type_ref: TypedType::ExternalNamed {
                name: "Box".to_owned(),
                canonical: "fixture/model::Box".to_owned(),
                arguments: vec![slot.type_ref.clone()],
            },
            key: SemanticTypeKey::ExternalNominal {
                canonical: "fixture/model::Box".to_owned(),
                arguments: vec![slot.clone()],
            },
        };
        let value = SemanticValueType::function(slot.clone(), nominal);
        let display = BTreeMap::from([("A".to_owned(), slot.type_ref)]);
        assert_eq!(
            substitute_remaining_scheme_parameters(&value, &display),
            value
        );

        let resolved = SemanticValueType {
            type_ref: TypedType::Named {
                name: "A".to_owned(),
                arguments: vec![],
            },
            key: SemanticTypeKey::TypeParameter(SymbolId(42)),
        };
        assert_eq!(
            substitute_remaining_scheme_parameters(&resolved, &display),
            resolved
        );
    }

    #[test]
    fn projects_checked_caller_schemes_without_resolving_contextual_schemes() {
        let slot = |name: &str| SemanticValueType {
            type_ref: TypedType::Named {
                name: name.to_owned(),
                arguments: vec![],
            },
            key: SemanticTypeKey::SchemeParameter(name.to_owned()),
        };
        let template = slot("A");
        let caller = slot("Caller");
        let source = instantiate_callable(&[template.clone()], None, &[caller.clone()], &template);
        assert_eq!(source.result, caller);
        assert!(!source.parameter_bindings.is_empty());

        let contextual = instantiate_callable(&[template.clone()], Some(&caller), &[], &template);
        assert!(contextual.parameter_bindings.is_empty());
        let remaining = instantiate_callable_indexed_with_display_types(
            &[template.clone()],
            None,
            &[(0, caller)],
            &template,
            None,
            0,
        );
        assert!(remaining.parameter_bindings.is_empty());
    }

    #[test]
    fn imported_nominal_substitution_preserves_function_slots_only_for_checked_owners() {
        let slot = SemanticValueType {
            type_ref: TypedType::Named {
                name: "A".to_owned(),
                arguments: vec![],
            },
            key: SemanticTypeKey::SchemeParameter("A".to_owned()),
        };
        let binder = SemanticValueType {
            type_ref: TypedType::Named {
                name: "Unit".to_owned(),
                arguments: vec![],
            },
            key: SemanticTypeKey::TypeParameter(SymbolId(42)),
        };
        let callback = SemanticValueType::function(binder.clone(), binder);
        let nominal = |argument: SemanticValueType, local: bool| SemanticValueType {
            type_ref: TypedType::ExternalNamed {
                canonical: "fixture/owner::Box".to_owned(),
                name: "Box".to_owned(),
                arguments: vec![argument.type_ref.clone()],
            },
            key: if local {
                SemanticTypeKey::Adt {
                    owner: SymbolId(7),
                    arguments: vec![argument],
                }
            } else {
                SemanticTypeKey::ExternalNominal {
                    canonical: "fixture/owner::Box".to_owned(),
                    arguments: vec![argument],
                }
            },
        };
        for local_template in [false, true] {
            let template = nominal(slot.clone(), local_template);
            let argument = nominal(callback.clone(), !local_template);
            let inferred =
                instantiate_callable(&[template.clone()], None, &[argument.clone()], &slot);
            assert_eq!(inferred.result, callback);

            let mut masked = nominal(slot.clone(), !local_template);
            if let TypedType::ExternalNamed { arguments, .. } = &mut masked.type_ref {
                arguments[0] = TypedType::Hole;
            }
            let contextual = instantiate_callable(
                &[template.clone()],
                Some(&masked),
                &[argument.clone()],
                &template,
            );
            assert_eq!(contextual.result, nominal(callback.clone(), local_template));

            let incompatible_context = nominal(
                SemanticValueType {
                    type_ref: TypedType::Named {
                        name: "Bool".to_owned(),
                        arguments: vec![],
                    },
                    key: SemanticTypeKey::Other,
                },
                !local_template,
            );
            let contextual_without_selection = instantiate_callable(
                &[template.clone()],
                Some(&incompatible_context),
                &[argument.clone()],
                &template,
            );
            assert_eq!(
                contextual_without_selection.result,
                nominal(
                    SemanticValueType {
                        type_ref: TypedType::Named {
                            name: "Bool".to_owned(),
                            arguments: vec![]
                        },
                        key: SemanticTypeKey::Other
                    },
                    local_template,
                )
            );
            let selected = BTreeMap::from([("A".to_owned(), callback.type_ref.clone())]);
            let source_selected = instantiate_callable_indexed_with_display_types(
                &[template.clone()],
                Some(&incompatible_context),
                &[(0, argument.clone())],
                &template,
                Some(&selected),
                1,
            );
            assert_eq!(
                source_selected.result,
                nominal(callback.clone(), local_template)
            );

            let mut other_binder = callback.clone();
            if let SemanticTypeKey::Function { parameter, result } = &mut other_binder.key {
                parameter.key = SemanticTypeKey::TypeParameter(SymbolId(43));
                result.key = SemanticTypeKey::TypeParameter(SymbolId(43));
            }
            let distinct_context = nominal(other_binder.clone(), !local_template);
            let ambiguous_display = instantiate_callable_indexed_with_display_types(
                &[template.clone()],
                Some(&distinct_context),
                &[(0, argument.clone())],
                &template,
                Some(&selected),
                1,
            );
            assert_eq!(
                ambiguous_display.result,
                nominal(other_binder, local_template)
            );

            let mut wrong_owner = argument.clone();
            if let TypedType::ExternalNamed { canonical, .. } = &mut wrong_owner.type_ref {
                *canonical = "fixture/other::Box".to_owned();
            }
            let rejected = instantiate_callable(&[template.clone()], None, &[wrong_owner], &slot);
            assert_eq!(rejected.result, slot);

            let mut display_only = argument;
            if let TypedType::ExternalNamed {
                name, arguments, ..
            } = display_only.type_ref
            {
                display_only.type_ref = TypedType::Named { name, arguments };
            }
            let unproved = instantiate_callable(&[template], None, &[display_only], &slot);
            assert_eq!(unproved.result, slot);
        }
    }
}
