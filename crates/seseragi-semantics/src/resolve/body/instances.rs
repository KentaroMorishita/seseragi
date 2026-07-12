use crate::ResolvedDependencyInstance;
use seseragi_project::LinkedDependency;
use seseragi_syntax::{InterfaceInstance, InterfaceType};
use std::collections::BTreeMap;

pub(super) fn resolve_dependency_instances(
    dependencies: &[LinkedDependency],
) -> Vec<ResolvedDependencyInstance> {
    let mut resolved = BTreeMap::new();
    for dependency in dependencies {
        for instance in &dependency.interface.instances {
            let Some(instance) = resolved_dependency_instance(dependency, instance) else {
                continue;
            };
            resolved
                .entry((instance.provider_module.clone(), instance.identity.clone()))
                .or_insert(instance);
        }
    }
    resolved.into_values().collect()
}

fn resolved_dependency_instance(
    dependency: &LinkedDependency,
    instance: &InterfaceInstance,
) -> Option<ResolvedDependencyInstance> {
    let type_identity = canonical_head_type(dependency, instance)?;
    let identity = instance.identity.clone()?;
    if identity
        != crate::instance_identity::canonical_instance_identity(
            &instance.trait_name,
            &type_identity,
        )
    {
        return None;
    }
    Some(ResolvedDependencyInstance {
        identity,
        provider_module: dependency.interface.module.clone(),
        trait_name: instance.trait_name.clone(),
        type_identity,
        origin: instance.origin,
    })
}

fn canonical_head_type(
    dependency: &LinkedDependency,
    instance: &InterfaceInstance,
) -> Option<String> {
    let InterfaceType::Apply {
        constructor,
        arguments,
    } = &instance.head
    else {
        return None;
    };
    let [InterfaceType::Named { name, arguments }] = arguments.as_slice() else {
        return None;
    };
    if constructor != &instance.trait_name || !arguments.is_empty() {
        return None;
    }
    dependency
        .interface
        .exports
        .iter()
        .find(|export| {
            export.namespace == "type"
                && export.declaration_kind.as_deref() == Some("type")
                && export.name == *name
        })
        .map(|export| export.symbol.clone())
}
