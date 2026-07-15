use crate::{SymbolNamespace, TypedConstraint, TypedInstanceEvidence, TypedType};
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
    let mut matches = resolution
        .resolved()
        .declarations
        .iter()
        .filter_map(|declaration| {
            match_instance(declaration, trait_identity, constraint, resolution)
        });
    let selected = matches.next()?;
    matches.next().is_none().then_some(selected)
}

fn match_instance(
    declaration: &SurfaceDecl,
    trait_identity: &str,
    constraint: &TypedConstraint,
    resolution: &TypedResolution<'_>,
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
    if !constraints.is_empty() || arguments.len() != constraint.arguments.len() {
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
    let mut substitutions = BTreeMap::<String, TypedType>::new();
    for (template, actual) in templates.iter().zip(&constraint.arguments) {
        infer_type_parameters(template, actual, type_parameters, &mut substitutions);
    }
    let type_arguments = type_parameters
        .iter()
        .map(|parameter| substitutions.get(parameter).cloned())
        .collect::<Option<Vec<_>>>()?;
    let matches = templates
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
        .map(|(index, parameter)| (parameter.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let canonical_arguments = arguments
        .iter()
        .map(|argument| canonical_type_ref(argument, resolution, &binders))
        .collect::<Option<Vec<_>>>()?;
    Some(TypedInstanceEvidence::Local {
        identity: canonical_instance_head_identity(trait_identity, &canonical_arguments),
        type_arguments,
    })
}
