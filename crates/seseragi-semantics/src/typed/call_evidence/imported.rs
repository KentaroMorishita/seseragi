use crate::{TypedConstraint, TypedInstanceEvidence, TypedType};

use super::super::semantic_types::{SemanticTypeKey, SemanticValueType};
use super::super::TypedResolution;

pub(super) fn select_imported_instance(
    trait_identity: &str,
    constraint: &TypedConstraint,
    resolution: &TypedResolution<'_>,
) -> Option<TypedInstanceEvidence> {
    let argument_identities = constraint
        .arguments
        .iter()
        .map(|argument| canonical_typed_type(argument, resolution))
        .collect::<Option<Vec<_>>>()?;
    let matches = resolution
        .resolved()
        .dependency_instances
        .iter()
        .filter(|instance| {
            instance.trait_identity == trait_identity
                && instance.argument_identities == argument_identities
                // Generic imported factories need substitution and recursive
                // evidence materialization. Do not treat their template head
                // as a concrete dictionary in this exact-match slice.
                && instance.type_parameters.is_empty()
                && instance.constraints.is_empty()
        })
        .take(2)
        .collect::<Vec<_>>();
    let [selected] = matches.as_slice() else {
        return None;
    };
    Some(TypedInstanceEvidence::Imported {
        identity: selected.identity.clone(),
        provider_module: selected.provider_module.clone(),
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
