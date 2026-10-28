use crate::{ResolvedModule, TypedInstanceImplementation, TypedModule};
use seseragi_syntax::{
    declarable_standard_operator, ByteSpan, Deprecation, InterfaceOperatorDeprecation, SurfaceDecl,
    SurfaceImplMember,
};
use std::collections::BTreeMap;

pub(crate) struct OperatorSource {
    declaration: ByteSpan,
    name_span: ByteSpan,
    spelling: String,
    trait_identity: String,
    method: String,
    deprecation: Deprecation,
}

pub(crate) fn sources(resolved: &ResolvedModule) -> Vec<OperatorSource> {
    let mut sources = Vec::new();
    for declaration in &resolved.declarations {
        let SurfaceDecl::Impl { members, .. } = declaration else {
            continue;
        };
        for member in members {
            let SurfaceImplMember::Operator {
                spelling,
                spelling_span,
                span,
                ..
            } = member
            else {
                continue;
            };
            let Some(deprecation) = resolved
                .deprecations
                .iter()
                .find(|entry| entry.declaration == *span)
            else {
                continue;
            };
            let Some(operator) = declarable_standard_operator(spelling) else {
                continue;
            };
            let Some(contract) = crate::prelude::trait_by_name(operator.trait_name) else {
                continue;
            };
            sources.push(OperatorSource {
                declaration: *span,
                name_span: *spelling_span,
                spelling: spelling.clone(),
                trait_identity: contract.canonical.to_owned(),
                method: operator.method_name.to_owned(),
                deprecation: deprecation.deprecation.clone(),
            });
        }
    }
    sources
}

/// A source span alone is not enough: the final typed instance must retain
/// the canonical trait and its actual method implementation at that origin.
pub(crate) fn local(
    sources: Vec<OperatorSource>,
    typed: &TypedModule,
) -> Vec<InterfaceOperatorDeprecation> {
    sources.into_iter().filter_map(|source| {
        let instance = typed.instances.iter().find(|instance| {
            instance.origin == source.declaration && instance.trait_identity == source.trait_identity
                && matches!(&instance.implementation, TypedInstanceImplementation::UserDefined { methods }
                    if methods.iter().any(|method| method.name == source.method && method.origin == source.declaration))
        })?;
        Some(InterfaceOperatorDeprecation {
            instance: instance.identity.clone(), provider_module: typed.module.clone(),
            trait_identity: source.trait_identity, method: source.method, spelling: source.spelling,
            declaration: source.declaration, name_span: source.name_span, deprecation: source.deprecation,
        })
    }).collect()
}

pub(crate) fn merge(
    entries: impl IntoIterator<Item = InterfaceOperatorDeprecation>,
) -> Vec<InterfaceOperatorDeprecation> {
    let mut unique = BTreeMap::new();
    for entry in entries {
        let key = (
            entry.provider_module.clone(),
            entry.instance.clone(),
            entry.trait_identity.clone(),
            entry.method.clone(),
        );
        unique
            .entry(key)
            .and_modify(|previous: &mut Option<InterfaceOperatorDeprecation>| {
                if previous.as_ref() != Some(&entry) {
                    *previous = None;
                }
            })
            .or_insert(Some(entry));
    }
    unique.into_values().flatten().collect()
}
