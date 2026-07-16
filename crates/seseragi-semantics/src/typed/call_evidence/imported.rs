use crate::{TypedCallEvidence, TypedConstraint, TypedInstanceEvidence, TypedType};
use seseragi_syntax::InterfaceType;
use std::collections::BTreeMap;

use super::super::TypedResolution;
use super::super::{
    functions::{infer_type_parameters, substitute_type_parameters},
    semantic_types::{semantic_values_are_compatible, SemanticTypeKey, SemanticValueType},
    type_ref::typed_type_from_interface_type,
};

pub(super) fn select_imported_instance(
    trait_identity: &str,
    constraint: &TypedConstraint,
    resolution: &TypedResolution<'_>,
    scoped: &[super::ScopedCallEvidence],
) -> Option<TypedInstanceEvidence> {
    select_imported_instance_with_stack(
        trait_identity,
        constraint,
        resolution,
        scoped,
        &mut Vec::new(),
    )
}

pub(super) fn infer_imported_functional_instance(
    trait_identity: &str,
    trait_name: &str,
    collection: &TypedType,
    resolution: &TypedResolution<'_>,
    scoped: &[super::ScopedCallEvidence],
) -> Option<(TypedType, TypedInstanceEvidence)> {
    let matches = resolution
        .resolved()
        .dependency_instances
        .iter()
        .filter_map(|instance| {
            infer_functional_instance_candidate(
                instance,
                trait_identity,
                trait_name,
                collection,
                resolution,
                scoped,
            )
        })
        .take(2)
        .collect::<Vec<_>>();
    let [selected] = matches.as_slice() else {
        return None;
    };
    Some(selected.clone())
}

fn infer_functional_instance_candidate(
    instance: &crate::ResolvedDependencyInstance,
    trait_identity: &str,
    trait_name: &str,
    collection: &TypedType,
    resolution: &TypedResolution<'_>,
    scoped: &[super::ScopedCallEvidence],
) -> Option<(TypedType, TypedInstanceEvidence)> {
    if instance.trait_identity != trait_identity {
        return None;
    }
    let InterfaceType::Apply { arguments, .. } = &instance.head else {
        return None;
    };
    let [collection_template, element_template] = arguments.as_slice() else {
        return None;
    };
    let collection_template = resolution.semantic_value_from_imported_type(
        collection_template.clone(),
        &instance.provider_module,
        &instance.type_parameters,
    )?;
    let collection_template = super::local::normalize_partial_constructor_template(
        &collection_template.type_ref,
        collection,
    );
    let mut substitutions = BTreeMap::<String, TypedType>::new();
    infer_type_parameters(
        &collection_template,
        collection,
        &instance.type_parameters,
        &mut substitutions,
    );
    let expected_collection = substitute_type_parameters(&collection_template, &substitutions);
    if !semantic_values_are_compatible(
        &resolution.semantic_value_from_typed_type(&expected_collection),
        &resolution.semantic_value_from_typed_type(collection),
    ) {
        return None;
    }
    let element_template = resolution.semantic_value_from_imported_type(
        element_template.clone(),
        &instance.provider_module,
        &instance.type_parameters,
    )?;
    let element = substitute_type_parameters(&element_template.type_ref, &substitutions);
    let constraint = TypedConstraint {
        name: trait_name.to_owned(),
        arguments: vec![collection.clone(), element.clone()],
    };
    match_imported_instance(
        instance,
        trait_identity,
        &constraint,
        resolution,
        scoped,
        &mut Vec::new(),
    )
    .map(|evidence| (element, evidence))
}

fn select_imported_instance_with_stack(
    trait_identity: &str,
    constraint: &TypedConstraint,
    resolution: &TypedResolution<'_>,
    scoped: &[super::ScopedCallEvidence],
    stack: &mut Vec<(String, Vec<TypedType>)>,
) -> Option<TypedInstanceEvidence> {
    let key = (trait_identity.to_owned(), constraint.arguments.clone());
    if stack.contains(&key) {
        return None;
    }
    stack.push(key);
    let mut matches = Vec::new();
    for instance in &resolution.resolved().dependency_instances {
        if let Some(evidence) = match_imported_instance(
            instance,
            trait_identity,
            constraint,
            resolution,
            scoped,
            stack,
        ) {
            matches.push(evidence);
            if matches.len() == 2 {
                break;
            }
        }
    }
    stack.pop();
    let [selected] = matches.as_slice() else {
        return None;
    };
    Some(selected.clone())
}

fn match_imported_instance(
    instance: &crate::ResolvedDependencyInstance,
    trait_identity: &str,
    constraint: &TypedConstraint,
    resolution: &TypedResolution<'_>,
    scoped: &[super::ScopedCallEvidence],
    stack: &mut Vec<(String, Vec<TypedType>)>,
) -> Option<TypedInstanceEvidence> {
    if instance.trait_identity != trait_identity {
        return None;
    }
    let (type_arguments, substitutions) = if instance.type_parameters.is_empty() {
        let argument_identities = constraint
            .arguments
            .iter()
            .map(|argument| canonical_typed_type(argument, resolution))
            .collect::<Option<Vec<_>>>()?;
        if instance.argument_identities != argument_identities {
            return None;
        }
        (Vec::new(), BTreeMap::new())
    } else {
        let InterfaceType::Apply { arguments, .. } = &instance.head else {
            return None;
        };
        if arguments.len() != constraint.arguments.len() {
            return None;
        }
        let templates = arguments
            .iter()
            .cloned()
            .map(typed_type_from_interface_type)
            .collect::<Option<Vec<_>>>()?;
        let matching_templates = templates
            .iter()
            .zip(&constraint.arguments)
            .map(|(template, actual)| {
                super::local::normalize_partial_constructor_template(template, actual)
            })
            .collect::<Vec<_>>();
        let mut substitutions = BTreeMap::<String, TypedType>::new();
        for (template, actual) in matching_templates.iter().zip(&constraint.arguments) {
            infer_type_parameters(
                template,
                actual,
                &instance.type_parameters,
                &mut substitutions,
            );
        }
        let type_arguments = instance
            .type_parameters
            .iter()
            .map(|parameter| substitutions.get(&parameter.name).cloned())
            .collect::<Option<Vec<_>>>()?;
        let matches = matching_templates
            .iter()
            .map(|template| substitute_type_parameters(template, &substitutions))
            .zip(&constraint.arguments)
            .all(|(expected, actual)| {
                semantic_values_are_compatible(
                    &resolution.semantic_value_from_typed_type(&expected),
                    &resolution.semantic_value_from_typed_type(actual),
                )
            });
        if !matches {
            return None;
        }
        (type_arguments, substitutions)
    };

    let evidence_arguments = instance
        .constraints
        .iter()
        .map(|required| {
            let constraint = TypedConstraint {
                name: required.name.clone(),
                arguments: required
                    .arguments
                    .iter()
                    .cloned()
                    .map(typed_type_from_interface_type)
                    .map(|argument| {
                        argument
                            .map(|argument| substitute_type_parameters(&argument, &substitutions))
                    })
                    .collect::<Option<Vec<_>>>()?,
            };
            let evidence = select_required_evidence(
                required.trait_identity.as_deref(),
                &constraint,
                resolution,
                scoped,
                stack,
            )?;
            Some(TypedCallEvidence {
                constraint,
                evidence,
            })
        })
        .collect::<Option<Vec<_>>>()?;
    Some(TypedInstanceEvidence::Imported {
        identity: instance.identity.clone(),
        provider_module: instance.provider_module.clone(),
        type_arguments,
        evidence_arguments,
    })
}

fn select_required_evidence(
    trait_identity: Option<&str>,
    constraint: &TypedConstraint,
    resolution: &TypedResolution<'_>,
    scoped: &[super::ScopedCallEvidence],
    stack: &mut Vec<(String, Vec<TypedType>)>,
) -> Option<TypedInstanceEvidence> {
    if let Some(trait_identity) = trait_identity {
        if let Some(parameter) = scoped.iter().find(|available| {
            available.trait_identity == trait_identity
                && available.constraint.arguments.len() == constraint.arguments.len()
                && available
                    .constraint
                    .arguments
                    .iter()
                    .zip(&constraint.arguments)
                    .all(|(available, required)| {
                        semantic_values_are_compatible(
                            &resolution.semantic_value_from_typed_type(available),
                            &resolution.semantic_value_from_typed_type(required),
                        )
                    })
        }) {
            return Some(TypedInstanceEvidence::Parameter {
                index: parameter.index,
            });
        }
        if let Some(local) =
            super::local::select_local_instance(trait_identity, constraint, resolution)
        {
            return Some(local);
        }
        if let Some(imported) = select_imported_instance_with_stack(
            trait_identity,
            constraint,
            resolution,
            scoped,
            stack,
        ) {
            return Some(imported);
        }
    }
    super::select_standard_instance(trait_identity, constraint)
}

fn canonical_typed_type(type_ref: &TypedType, resolution: &TypedResolution<'_>) -> Option<String> {
    let value = resolution.semantic_value_from_typed_type(type_ref);
    canonical_semantic_value(&value, resolution)
}

fn canonical_semantic_value(
    value: &SemanticValueType,
    resolution: &TypedResolution<'_>,
) -> Option<String> {
    match &value.key {
        SemanticTypeKey::Adt { owner, arguments } => {
            let symbol = resolution.symbol(*owner)?;
            let constructor = symbol
                .canonical
                .clone()
                .unwrap_or_else(|| symbol.spelling.clone());
            canonical_application(constructor, arguments, resolution)
        }
        SemanticTypeKey::ExternalNominal {
            canonical,
            arguments,
        } => canonical_application(canonical.clone(), arguments, resolution),
        SemanticTypeKey::Tuple(elements) => {
            let TypedType::Tuple { elements: types } = &value.type_ref else {
                return None;
            };
            if elements.len() != types.len() {
                return None;
            }
            Some(format!(
                "({})",
                types
                    .iter()
                    .zip(elements)
                    .map(|(type_ref, key)| {
                        canonical_semantic_value(
                            &SemanticValueType {
                                type_ref: type_ref.clone(),
                                key: key.clone(),
                            },
                            resolution,
                        )
                    })
                    .collect::<Option<Vec<_>>>()?
                    .join(",")
            ))
        }
        SemanticTypeKey::Other => canonical_other_type(&value.type_ref, resolution),
        SemanticTypeKey::Invalid
        | SemanticTypeKey::TypeParameter(_)
        | SemanticTypeKey::SchemeParameter(_) => None,
    }
}

fn canonical_application(
    constructor: String,
    arguments: &[SemanticValueType],
    resolution: &TypedResolution<'_>,
) -> Option<String> {
    if arguments.is_empty() {
        return Some(constructor);
    }
    Some(format!(
        "{constructor}<{}>",
        arguments
            .iter()
            .map(|argument| canonical_semantic_value(argument, resolution))
            .collect::<Option<Vec<_>>>()?
            .join(",")
    ))
}

fn canonical_other_type(type_ref: &TypedType, resolution: &TypedResolution<'_>) -> Option<String> {
    match type_ref {
        TypedType::Named { name, arguments } => canonical_named(name, arguments, resolution),
        TypedType::ExternalNamed {
            canonical,
            arguments,
            ..
        } => canonical_named(canonical, arguments, resolution),
        TypedType::Hole => None,
        TypedType::Tuple { elements } => Some(format!(
            "({})",
            elements
                .iter()
                .map(|element| canonical_typed_type(element, resolution))
                .collect::<Option<Vec<_>>>()?
                .join(",")
        )),
        TypedType::Function { parameter, result } => Some(format!(
            "({}->{})",
            canonical_typed_type(parameter, resolution)?,
            canonical_typed_type(result, resolution)?
        )),
        TypedType::Record { closed, fields } => {
            let mut fields = fields
                .iter()
                .map(|field| {
                    Some(format!(
                        "{}{}:{}",
                        field.name,
                        if field.optional { "?" } else { "" },
                        canonical_typed_type(&field.type_ref, resolution)?
                    ))
                })
                .collect::<Option<Vec<_>>>()?;
            fields.sort();
            Some(format!(
                "{}{{{}}}",
                if *closed { "" } else { "open" },
                fields.join(",")
            ))
        }
    }
}

fn canonical_named(
    constructor: &str,
    arguments: &[TypedType],
    resolution: &TypedResolution<'_>,
) -> Option<String> {
    let constructor =
        if crate::prelude::is_standalone_symbol(crate::SymbolNamespace::Type, constructor)
            || crate::prelude::sum_type_for_symbol(crate::SymbolNamespace::Type, constructor)
                .is_some()
        {
            format!("std/prelude::{constructor}")
        } else {
            constructor.to_owned()
        };
    if arguments.is_empty() {
        return Some(constructor);
    }
    Some(format!(
        "{constructor}<{}>",
        arguments
            .iter()
            .map(|argument| canonical_typed_type(argument, resolution))
            .collect::<Option<Vec<_>>>()?
            .join(",")
    ))
}
