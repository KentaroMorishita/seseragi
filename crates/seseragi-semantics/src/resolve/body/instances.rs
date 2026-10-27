use crate::{ResolveIssue, ResolvedDependencyInstance};
use seseragi_project::LinkedDependency;
use seseragi_syntax::{InterfaceInstance, InterfaceType};
use std::collections::BTreeMap;

pub(super) fn resolve_dependency_instances(
    dependencies: &[LinkedDependency],
) -> (Vec<ResolvedDependencyInstance>, Vec<ResolveIssue>) {
    let mut resolved = BTreeMap::<
        crate::instance_identity::CanonicalInstanceHeadKey,
        Vec<ResolvedDependencyInstance>,
    >::new();
    let mut heads_by_trait = BTreeMap::<
        crate::instance_identity::CanonicalInstanceHeadKey,
        Vec<Option<crate::typed::instances::CoherenceHead>>,
    >::new();
    let mut conflict_origins = BTreeMap::new();
    for dependency in dependencies {
        for instance in &dependency.interface.instances {
            let Some(instance) = resolved_dependency_instance(dependency, instance) else {
                continue;
            };
            let key = crate::instance_identity::canonical_instance_head_key(
                &instance.trait_identity,
                &instance.argument_identities,
            );
            if resolved.get(&key).is_some_and(|candidates| {
                candidates.iter().any(|existing| {
                    existing.identity == instance.identity
                        && existing.provider_module == instance.provider_module
                        && existing.trait_identity == instance.trait_identity
                        && existing.type_parameters == instance.type_parameters
                        && existing.head == instance.head
                        && existing.constraints == instance.constraints
                })
            }) {
                continue;
            }
            let head = crate::typed::instances::dependency_head(&instance);
            let trait_key = crate::instance_identity::canonical_instance_head_key(
                &instance.trait_identity,
                &[],
            );
            let canonical_trait =
                crate::typed::instances::canonical_trait(&instance.trait_identity);
            let heads = heads_by_trait.entry(trait_key).or_default();
            use crate::typed::instances::HeadRelation;
            let relation = if resolved.contains_key(&key) {
                HeadRelation::Overlap
            } else {
                heads
                    .iter()
                    .map(|left| match (left, &head) {
                        (Some(left), Some(right)) => left.relation(right, &canonical_trait),
                        _ => HeadRelation::Unproved,
                    })
                    .fold(HeadRelation::Disjoint, |result, next| {
                        match (result, next) {
                            (HeadRelation::Overlap, _) | (_, HeadRelation::Overlap) => {
                                HeadRelation::Overlap
                            }
                            (HeadRelation::Unproved, _) | (_, HeadRelation::Unproved) => {
                                HeadRelation::Unproved
                            }
                            _ => HeadRelation::Disjoint,
                        }
                    })
            };
            if relation != HeadRelation::Disjoint {
                let origin = (dependency.origin.start, dependency.origin.end);
                conflict_origins
                    .entry(origin)
                    .and_modify(|previous| {
                        if relation == HeadRelation::Overlap {
                            *previous = relation;
                        }
                    })
                    .or_insert(relation);
            }
            heads.push(head);
            let candidates = resolved.entry(key).or_default();
            candidates.push(instance);
        }
    }
    let issues = conflict_origins
        .into_iter()
        .map(|((start, end), relation)| ResolveIssue {
            code: if relation == crate::typed::instances::HeadRelation::Overlap {
                "SES-T0202"
            } else {
                "SES-T0101"
            }
            .to_owned(),
            message_key: if relation == crate::typed::instances::HeadRelation::Overlap {
                "trait.instance-ambiguous"
            } else {
                "trait.instance-coherence-unproved"
            }
            .to_owned(),
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
    let canonical_identity = crate::instance_identity::canonical_instance_head_identity(
        &trait_identity,
        &argument_identities,
    );
    let source_identity = crate::instance_identity::canonical_instance_head_identity(
        &instance.trait_name,
        &argument_identities,
    );
    if identity != canonical_identity && identity != source_identity {
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
                namespace_deprecations: Vec::new(),
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

#[cfg(test)]
mod overlap_tests {
    use super::resolve_dependency_instances;
    use seseragi_project::LinkedDependency;
    use seseragi_syntax::{
        ByteSpan, InterfaceInstance, InterfaceType, ModuleInterface, TypeParameter,
    };

    fn dependency(provider: &str, generic: bool, origin: usize) -> LinkedDependency {
        let argument = if generic { "$0" } else { "std/prelude::Int" };
        let canonical = format!("fixture/types::Box<{argument}>");
        LinkedDependency {
            specifier: format!("./{provider}"),
            origin: ByteSpan {
                start: origin,
                end: origin + 1,
            },
            header: None,
            imports: Vec::new(),
            interface: ModuleInterface {
                namespace_deprecations: Vec::new(),
                schema: 1,
                module: format!("fixture/{provider}"),
                source: "lib.ssrg".into(),
                dependencies: Vec::new(),
                exports: Vec::new(),
                operators: Vec::new(),
                instances: vec![InterfaceInstance {
                    identity: Some(format!("std/prelude::Show<{canonical}>")),
                    provider_module: Some(format!("fixture/{provider}")),
                    trait_identity: Some("std/prelude::Show".into()),
                    trait_name: "Show".into(),
                    argument_identities: vec![canonical.clone()],
                    type_identity: Some(canonical),
                    type_parameters: if generic {
                        vec![TypeParameter::value("A")]
                    } else {
                        vec![]
                    },
                    head: InterfaceType::Apply {
                        constructor: "Show".into(),
                        arguments: vec![InterfaceType::ExternalNamed {
                            name: "Box".into(),
                            canonical: "fixture/types::Box".into(),
                            provider_module: "fixture/types".into(),
                            provider_export: "Box".into(),
                            arguments: vec![InterfaceType::Named {
                                name: if generic { "A" } else { "Int" }.into(),
                                arguments: vec![],
                            }],
                        }],
                    },
                    constraints: vec![],
                    origin: ByteSpan { start: 0, end: 1 },
                }],
            },
        }
    }

    #[test]
    fn overlap_checks_imported_generic_heads_without_selecting_a_dictionary() {
        for generic_first in [true, false] {
            let first = dependency("a", generic_first, 1);
            let second = dependency("b", !generic_first, 3);
            let (instances, issues) = resolve_dependency_instances(&[first, second]);
            assert_eq!(instances.len(), 2);
            assert_eq!(issues.len(), 1);
            assert_eq!(issues[0].code, "SES-T0202");
            assert_eq!(issues[0].primary, ByteSpan { start: 3, end: 4 });
        }
    }

    #[test]
    fn overlap_keeps_reexports_unique_and_unrelated_canonical_owners_distinct() {
        let first = dependency("a", true, 1);
        let mut facade = first.clone();
        facade.interface.module = "fixture/facade".into();
        facade.origin = ByteSpan { start: 3, end: 4 };
        let (instances, issues) = resolve_dependency_instances(&[first.clone(), facade]);
        assert_eq!(instances.len(), 1);
        assert!(issues.is_empty());
        let mut second = dependency("b", false, 3);
        second.interface.instances[0].trait_identity = Some("fixture/custom::Show".into());
        second.interface.instances[0].identity =
            Some("fixture/custom::Show<fixture/types::Box<std/prelude::Int>>".into());
        let (_, issues) = resolve_dependency_instances(&[first.clone(), second]);
        assert!(issues.is_empty());
        let mut second = dependency("b", false, 3);
        let instance = &mut second.interface.instances[0];
        instance.identity = instance
            .identity
            .as_ref()
            .map(|s| s.replace("fixture/types::Box", "fixture/other::Box"));
        instance.argument_identities[0] =
            instance.argument_identities[0].replace("fixture/types::Box", "fixture/other::Box");
        if let InterfaceType::Apply { arguments, .. } = &mut instance.head {
            if let InterfaceType::ExternalNamed {
                canonical,
                provider_module,
                ..
            } = &mut arguments[0]
            {
                *canonical = "fixture/other::Box".into();
                *provider_module = "fixture/other".into();
            }
        }
        let (_, issues) = resolve_dependency_instances(&[first, second]);
        assert!(issues.is_empty());
    }
    #[test]
    fn imported_applied_constructors_require_disjointness_and_keep_facades_unique() {
        let mut first = dependency("a", true, 11);
        let instance = &mut first.interface.instances[0];
        instance.identity = Some("std/prelude::Show<$0<std/prelude::Bool>>".into());
        instance.argument_identities = vec!["$0<std/prelude::Bool>".into()];
        instance.type_identity = Some(instance.argument_identities[0].clone());
        instance.type_parameters = vec![TypeParameter::constructor("F", 1)];
        instance.head = InterfaceType::Apply {
            constructor: "Show".into(),
            arguments: vec![InterfaceType::Named {
                name: "F".into(),
                arguments: vec![InterfaceType::Named {
                    name: "Bool".into(),
                    arguments: vec![],
                }],
            }],
        };
        let mut facade = first.clone();
        facade.interface.module = "fixture/facade".into();
        facade.origin = ByteSpan { start: 21, end: 22 };
        let second = dependency("b", false, 31);
        let (instances, issues) = resolve_dependency_instances(&[first, facade, second]);
        assert_eq!(instances.len(), 2);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, "SES-T0101");
        assert_eq!(issues[0].message_key, "trait.instance-coherence-unproved");
        assert_eq!(issues[0].primary, ByteSpan { start: 31, end: 32 });
    }
}
