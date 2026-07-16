use crate::{TypedConstraint, TypedInstanceEvidence, TypedType};
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
) -> Option<TypedInstanceEvidence> {
    let matches = resolution
        .resolved()
        .dependency_instances
        .iter()
        .filter_map(|instance| {
            match_imported_instance(instance, trait_identity, constraint, resolution)
        })
        .take(2)
        .collect::<Vec<_>>();
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
) -> Option<TypedInstanceEvidence> {
    if instance.trait_identity != trait_identity || !instance.constraints.is_empty() {
        return None;
    }
    if instance.type_parameters.is_empty() {
        let argument_identities = constraint
            .arguments
            .iter()
            .map(|argument| canonical_typed_type(argument, resolution))
            .collect::<Option<Vec<_>>>()?;
        return (instance.argument_identities == argument_identities).then(|| {
            TypedInstanceEvidence::Imported {
                identity: instance.identity.clone(),
                provider_module: instance.provider_module.clone(),
                type_arguments: Vec::new(),
                evidence_arguments: Vec::new(),
            }
        });
    }
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
    matches.then(|| TypedInstanceEvidence::Imported {
        identity: instance.identity.clone(),
        provider_module: instance.provider_module.clone(),
        type_arguments,
        evidence_arguments: Vec::new(),
    })
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
    if arguments.is_empty() {
        return Some(constructor.to_owned());
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
