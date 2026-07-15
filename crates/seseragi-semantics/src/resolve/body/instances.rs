use crate::{ResolveIssue, ResolvedDependencyInstance};
use seseragi_project::LinkedDependency;
use seseragi_syntax::{InterfaceInstance, InterfaceType};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn resolve_dependency_instances(
    dependencies: &[LinkedDependency],
) -> (Vec<ResolvedDependencyInstance>, Vec<ResolveIssue>) {
    let mut resolved = BTreeMap::<(String, Vec<String>), Vec<ResolvedDependencyInstance>>::new();
    let mut conflict_origins = BTreeSet::new();
    for dependency in dependencies {
        for instance in &dependency.interface.instances {
            let Some(instance) = resolved_dependency_instance(dependency, instance) else {
                continue;
            };
            let key = (
                instance.trait_identity.clone(),
                instance.argument_identities.clone(),
            );
            let candidates = resolved.entry(key).or_default();
            if candidates.iter().any(|existing| {
                existing.identity == instance.identity
                    && existing.provider_module == instance.provider_module
                    && existing.trait_identity == instance.trait_identity
                    && existing.type_parameters == instance.type_parameters
                    && existing.head == instance.head
                    && existing.constraints == instance.constraints
            }) {
                continue;
            }
            if !candidates.is_empty() {
                conflict_origins.insert((dependency.origin.start, dependency.origin.end));
            }
            candidates.push(instance);
        }
    }
    let issues = conflict_origins
        .into_iter()
        .map(|(start, end)| ResolveIssue {
            code: "SES-T0202".to_owned(),
            message_key: "trait.instance-ambiguous".to_owned(),
            primary: seseragi_syntax::ByteSpan { start, end },
        })
        .collect();
    (resolved.into_values().flatten().collect(), issues)
}

fn resolved_dependency_instance(
    dependency: &LinkedDependency,
    instance: &InterfaceInstance,
) -> Option<ResolvedDependencyInstance> {
    let identity = instance.identity.clone()?;
    let provider_module = instance
        .provider_module
        .clone()
        .unwrap_or_else(|| dependency.interface.module.clone());
    let argument_identities = if instance.argument_identities.is_empty() {
        vec![canonical_head_type(dependency, instance)?]
    } else {
        instance.argument_identities.clone()
    };
    let trait_identity = instance
        .trait_identity
        .clone()
        .unwrap_or_else(|| instance.trait_name.clone());
    let type_identity = instance.type_identity.clone();
    if identity
        != crate::instance_identity::canonical_instance_head_identity(
            &trait_identity,
            &argument_identities,
        )
    {
        return None;
    }
    Some(ResolvedDependencyInstance {
        identity,
        provider_module,
        trait_identity,
        trait_name: instance.trait_name.clone(),
        argument_identities,
        type_identity,
        type_parameters: instance.type_parameters.clone(),
        head: instance.head.clone(),
        constraints: instance.constraints.clone(),
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

#[cfg(test)]
mod tests {
    use super::resolve_dependency_instances;
    use seseragi_project::LinkedDependency;
    use seseragi_syntax::{ByteSpan, InterfaceInstance, InterfaceType, ModuleInterface};

    #[test]
    fn preserves_conflicting_providers_and_reports_ambiguity() {
        let dependencies = [
            dependency_with_instance("fixture/facade-a", "fixture/provider-a"),
            dependency_with_instance("fixture/facade-b", "fixture/provider-b"),
        ];

        let (instances, issues) = resolve_dependency_instances(&dependencies);

        assert_eq!(instances.len(), 2);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, "SES-T0202");
        assert_eq!(issues[0].message_key, "trait.instance-ambiguous");
        assert_eq!(issues[0].primary, dependencies[1].origin);
    }

    #[test]
    fn reports_a_same_provider_contract_mismatch_as_ambiguity() {
        let first = dependency_with_instance("fixture/facade-a", "fixture/provider");
        let mut second = dependency_with_instance("fixture/facade-b", "fixture/provider");
        second.interface.instances[0].head = InterfaceType::Named {
            name: "DifferentHead".to_owned(),
            arguments: Vec::new(),
        };

        let (instances, issues) = resolve_dependency_instances(&[first, second]);

        assert_eq!(instances.len(), 2);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, "SES-T0202");
    }

    #[test]
    fn keeps_same_spelling_traits_separate_by_canonical_identity() {
        let mut first = dependency_with_instance("fixture/left", "fixture/left");
        first.interface.instances[0].trait_name = "Ready".to_owned();
        first.interface.instances[0].trait_identity = Some("fixture/left::trait(Ready)".to_owned());
        first.interface.instances[0].identity =
            Some("fixture/left::trait(Ready)<fixture/types::InputError>".to_owned());
        let mut second = dependency_with_instance("fixture/right", "fixture/right");
        second.interface.instances[0].trait_name = "Ready".to_owned();
        second.interface.instances[0].trait_identity =
            Some("fixture/right::trait(Ready)".to_owned());
        second.interface.instances[0].identity =
            Some("fixture/right::trait(Ready)<fixture/types::InputError>".to_owned());

        let (instances, issues) = resolve_dependency_instances(&[first, second]);

        assert_eq!(instances.len(), 2);
        assert!(issues.is_empty());
    }

    #[test]
    fn aggregates_multiple_conflicting_instance_keys_for_one_import_edge() {
        let first = dependency_with_two_instances("fixture/facade-a", "fixture/provider-a");
        let second = dependency_with_two_instances("fixture/facade-b", "fixture/provider-b");

        let (instances, issues) = resolve_dependency_instances(&[first, second]);

        assert_eq!(instances.len(), 4);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, "SES-T0202");
        assert_eq!(issues[0].primary, ByteSpan { start: 0, end: 1 });
    }

    fn dependency_with_two_instances(module: &str, provider_module: &str) -> LinkedDependency {
        let mut dependency = dependency_with_instance(module, provider_module);
        let mut second = dependency.interface.instances[0].clone();
        second.identity = Some("Show<fixture/types::OtherError>".to_owned());
        second.argument_identities = vec!["fixture/types::OtherError".to_owned()];
        second.type_identity = Some("fixture/types::OtherError".to_owned());
        second.head = InterfaceType::Named {
            name: "OtherError".to_owned(),
            arguments: Vec::new(),
        };
        dependency.interface.instances.push(second);
        dependency
    }

    fn dependency_with_instance(module: &str, provider_module: &str) -> LinkedDependency {
        LinkedDependency {
            specifier: format!("./{module}"),
            origin: ByteSpan { start: 0, end: 1 },
            interface: ModuleInterface {
                schema: 1,
                module: module.to_owned(),
                source: "facade.ssrg".to_owned(),
                dependencies: Vec::new(),
                exports: Vec::new(),
                operators: Vec::new(),
                instances: vec![InterfaceInstance {
                    identity: Some("Show<fixture/types::InputError>".to_owned()),
                    provider_module: Some(provider_module.to_owned()),
                    trait_identity: Some("Show".to_owned()),
                    argument_identities: vec!["fixture/types::InputError".to_owned()],
                    type_identity: Some("fixture/types::InputError".to_owned()),
                    trait_name: "Show".to_owned(),
                    type_parameters: Vec::new(),
                    head: InterfaceType::Named {
                        name: "InputError".to_owned(),
                        arguments: Vec::new(),
                    },
                    constraints: Vec::new(),
                    origin: ByteSpan { start: 0, end: 1 },
                }],
            },
            header: None,
            imports: Vec::new(),
        }
    }
}
