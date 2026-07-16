use crate::{
    SymbolNamespace, TypedCallEvidence, TypedConstraint, TypedInstanceEvidence, TypedType,
};
use seseragi_syntax::SurfaceDecl;
use std::collections::BTreeMap;

use super::super::functions::{infer_type_parameters, substitute_type_parameters};
use super::super::instances::{canonical_instance_head_identity, canonical_type_ref};
use super::super::semantic_types::semantic_values_are_compatible;
use super::super::type_ref::typed_type_from_type_ref;
use super::super::TypedResolution;

pub(super) fn select_local_instance(
    trait_identity: &str,
    constraint: &TypedConstraint,
    resolution: &TypedResolution<'_>,
) -> Option<TypedInstanceEvidence> {
    select_local_instance_with_stack(trait_identity, constraint, resolution, &mut Vec::new())
}

pub(super) fn infer_local_functional_instance(
    trait_identity: &str,
    trait_name: &str,
    collection: &TypedType,
    resolution: &TypedResolution<'_>,
) -> Option<(TypedType, TypedInstanceEvidence)> {
    let matches = resolution
        .resolved()
        .declarations
        .iter()
        .filter_map(|declaration| {
            infer_functional_instance_candidate(
                declaration,
                trait_identity,
                trait_name,
                collection,
                resolution,
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
    declaration: &SurfaceDecl,
    trait_identity: &str,
    trait_name: &str,
    collection: &TypedType,
    resolution: &TypedResolution<'_>,
) -> Option<(TypedType, TypedInstanceEvidence)> {
    let SurfaceDecl::Instance {
        type_parameters,
        trait_name_span,
        arguments,
        ..
    } = declaration
    else {
        return None;
    };
    let [collection_template, element_template] = arguments.as_slice() else {
        return None;
    };
    let target = resolution.target(*trait_name_span, SymbolNamespace::Trait)?;
    if resolution.symbol(target)?.canonical.as_deref() != Some(trait_identity) {
        return None;
    }
    let collection_template = normalize_partial_constructor_template(
        &typed_type_from_type_ref(collection_template),
        collection,
    );
    let mut substitutions = BTreeMap::<String, TypedType>::new();
    infer_type_parameters(
        &collection_template,
        collection,
        type_parameters,
        &mut substitutions,
    );
    let expected_collection = substitute_type_parameters(&collection_template, &substitutions);
    if !semantic_values_are_compatible(
        &resolution.semantic_value_from_typed_type(&expected_collection),
        &resolution.semantic_value_from_typed_type(collection),
    ) {
        return None;
    }
    let element =
        substitute_type_parameters(&typed_type_from_type_ref(element_template), &substitutions);
    let constraint = TypedConstraint {
        name: trait_name.to_owned(),
        arguments: vec![collection.clone(), element.clone()],
    };
    match_instance(
        declaration,
        trait_identity,
        &constraint,
        resolution,
        &mut Vec::new(),
    )
    .map(|evidence| (element, evidence))
}

fn select_local_instance_with_stack(
    trait_identity: &str,
    constraint: &TypedConstraint,
    resolution: &TypedResolution<'_>,
    stack: &mut Vec<(String, Vec<TypedType>)>,
) -> Option<TypedInstanceEvidence> {
    let key = (trait_identity.to_owned(), constraint.arguments.clone());
    if stack.contains(&key) {
        return None;
    }
    stack.push(key);
    let matches = resolution
        .resolved()
        .declarations
        .iter()
        .filter_map(|declaration| {
            match_instance(declaration, trait_identity, constraint, resolution, stack)
        })
        .take(2)
        .collect::<Vec<_>>();
    stack.pop();
    match matches.as_slice() {
        [selected] => Some(selected.clone()),
        _ => None,
    }
}

fn match_instance(
    declaration: &SurfaceDecl,
    trait_identity: &str,
    constraint: &TypedConstraint,
    resolution: &TypedResolution<'_>,
    stack: &mut Vec<(String, Vec<TypedType>)>,
) -> Option<TypedInstanceEvidence> {
    let SurfaceDecl::Instance {
        type_parameters,
        trait_name_span,
        arguments,
        constraints,
        ..
    } = declaration
    else {
        return None;
    };
    if arguments.len() != constraint.arguments.len() {
        return None;
    }
    let target = resolution.target(*trait_name_span, SymbolNamespace::Trait)?;
    let symbol = resolution.symbol(target)?;
    if symbol.canonical.as_deref() != Some(trait_identity) {
        return None;
    }

    let templates = arguments
        .iter()
        .map(typed_type_from_type_ref)
        .collect::<Vec<_>>();
    let matching_templates = templates
        .iter()
        .zip(&constraint.arguments)
        .map(|(template, actual)| normalize_partial_constructor_template(template, actual))
        .collect::<Vec<_>>();
    let mut substitutions = BTreeMap::<String, TypedType>::new();
    for (template, actual) in matching_templates.iter().zip(&constraint.arguments) {
        infer_type_parameters(template, actual, type_parameters, &mut substitutions);
    }
    let type_arguments = type_parameters
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

    let binders = type_parameters
        .iter()
        .enumerate()
        .map(|(index, parameter)| (parameter.name.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let canonical_arguments = arguments
        .iter()
        .map(|argument| canonical_type_ref(argument, resolution, &binders))
        .collect::<Option<Vec<_>>>()?;
    let supertraits =
        super::direct_supertrait_constraints(*trait_name_span, &constraint.arguments, resolution);
    let mut evidence_arguments = Vec::new();
    for required in supertraits {
        let evidence = select_local_instance_with_stack(
            &required.trait_identity,
            &required.constraint,
            resolution,
            stack,
        )
        .or_else(|| {
            super::select_standard_instance(Some(&required.trait_identity), &required.constraint)
        })?;
        evidence_arguments.push(TypedCallEvidence {
            constraint: required.constraint,
            evidence,
        });
    }
    for required in constraints {
        let target = resolution.target(required.name_span, SymbolNamespace::Trait)?;
        let required_trait = resolution.symbol(target)?.canonical.clone()?;
        let constraint = TypedConstraint {
            name: required.name.clone(),
            arguments: required
                .arguments
                .iter()
                .map(typed_type_from_type_ref)
                .map(|argument| substitute_type_parameters(&argument, &substitutions))
                .collect(),
        };
        let evidence =
            select_local_instance_with_stack(&required_trait, &constraint, resolution, stack)
                .or_else(|| super::select_standard_instance(Some(&required_trait), &constraint))?;
        evidence_arguments.push(TypedCallEvidence {
            constraint,
            evidence,
        });
    }
    Some(TypedInstanceEvidence::Local {
        identity: canonical_instance_head_identity(trait_identity, &canonical_arguments),
        type_arguments,
        evidence_arguments,
    })
}

pub(super) fn normalize_partial_constructor_template(
    template: &TypedType,
    actual: &TypedType,
) -> TypedType {
    let (template_name, template_arguments, actual_name, actual_arguments) =
        match (template, actual) {
            (
                TypedType::Named {
                    name: template_name,
                    arguments: template_arguments,
                },
                TypedType::Named {
                    name: actual_name,
                    arguments: actual_arguments,
                },
            ) => (
                template_name,
                template_arguments,
                actual_name,
                actual_arguments,
            ),
            _ => return template.clone(),
        };
    if template_name != actual_name || template_arguments.len() <= actual_arguments.len() {
        return template.clone();
    }
    let fixed = &template_arguments[..actual_arguments.len()];
    let unfilled = &template_arguments[actual_arguments.len()..];
    if unfilled.iter().all(|argument| *argument == TypedType::Hole) {
        TypedType::Named {
            name: template_name.clone(),
            arguments: fixed.to_vec(),
        }
    } else {
        template.clone()
    }
}
