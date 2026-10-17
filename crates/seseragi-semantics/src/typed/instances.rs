use crate::{ResolvedModule, TypedInstance, TypedType};
use seseragi_syntax::ByteSpan;

use super::TypedResolution;

mod contracts;
mod show;
#[cfg(test)]
mod tests;
mod traits;
mod user;

pub(crate) use crate::instance_identity::{
    canonical_instance_head_identity, canonical_instance_head_key, canonical_instance_identity,
};
pub(crate) use contracts::{
    analyze_instance_contracts, dependency_head, CoherenceHead, HeadRelation, InstanceContractIssue,
};
pub(crate) use show::derived_display_requirements;
pub(crate) use user::canonical_type_ref;

pub(crate) struct InstanceAnalysis {
    pub(crate) instances: Vec<TypedInstance>,
    pub(crate) issues: Vec<DerivedInstanceIssue>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DerivedInstanceIssue {
    UnknownTrait {
        trait_name: String,
        primary: ByteSpan,
        declaration: ByteSpan,
    },
    UnsupportedDerivedMember {
        trait_name: String,
        member_type: TypedType,
        primary: ByteSpan,
        declaration: ByteSpan,
    },
    AmbiguousInstance {
        trait_name: String,
        type_identity: String,
        provider_module: String,
        primary: ByteSpan,
    },
    DuplicateLocalInstance {
        trait_name: String,
        identity: String,
        primary: ByteSpan,
        first: ByteSpan,
    },
    OverlappingLocalInstance {
        trait_name: String,
        identity: String,
        primary: ByteSpan,
        first: ByteSpan,
    },
    UnprovedInstanceCoherence {
        trait_name: String,
        primary: ByteSpan,
        first: Option<ByteSpan>,
        provider_module: Option<String>,
    },
    OverlappingStandardInstance {
        trait_name: String,
        standard_identity: String,
        primary: ByteSpan,
    },
    OrphanInstance {
        identity: String,
        primary: ByteSpan,
    },
    UnresolvedInstanceHead {
        primary: ByteSpan,
    },
}

pub(crate) fn analyze_instances(
    resolved: &ResolvedModule,
    resolution: &TypedResolution<'_>,
) -> InstanceAnalysis {
    let mut issues = traits::unknown_trait_issues(&resolved.declarations);
    let show = show::analyze_derived_show(resolved, resolution);
    let mut instances = show.instances;
    instances.extend(user::analyze_user_defined_instances(resolved, resolution));
    issues.extend(show.issues);
    issues.extend(local_instance_conflicts(resolved, &instances));
    issues.extend(standard_instance_conflicts(&instances));
    issues.extend(local_dependency_conflicts(resolved, &instances));
    for instance in &instances {
        let erased_parameter = resolved.declarations.iter().any(|declaration| {
            matches!(declaration, seseragi_syntax::SurfaceDecl::Instance { span, type_parameters, arguments, .. }
                if *span == instance.origin && contracts::has_unrepresented_parameters(resolved, *span, type_parameters, arguments))
        });
        if erased_parameter || instance.arguments.iter().any(|argument| matches!(argument, TypedType::Hole)) {
            issues.push(DerivedInstanceIssue::UnresolvedInstanceHead { primary: instance.origin });
        } else if !owns_instance_head(resolved, instance)
            && !issues.iter().any(|issue| matches!(issue,
                DerivedInstanceIssue::OverlappingStandardInstance { primary, .. } if *primary == instance.origin))
        {
            issues.push(DerivedInstanceIssue::OrphanInstance {
                identity: instance.identity.clone(), primary: instance.origin,
            });
        }
    }
    InstanceAnalysis { instances, issues }
}

fn owns_instance_head(resolved: &ResolvedModule, instance: &TypedInstance) -> bool {
    resolved.declarations.iter().any(|declaration| {
        let (origin, is_trait) = match declaration {
            seseragi_syntax::SurfaceDecl::Trait { name_span, .. } => (*name_span, true),
            seseragi_syntax::SurfaceDecl::Type { name_span, .. }
            | seseragi_syntax::SurfaceDecl::Newtype { name_span, .. }
            | seseragi_syntax::SurfaceDecl::Struct { name_span, .. } => (*name_span, false),
            _ => return false,
        };
        let namespace = if is_trait {
            crate::SymbolNamespace::Trait
        } else {
            crate::SymbolNamespace::Type
        };
        let Some(owner) = resolved
            .symbols
            .iter()
            .find(|symbol| symbol.namespace == namespace && symbol.origin == origin)
            .and_then(|symbol| symbol.canonical.as_deref())
        else {
            return false;
        };
        if is_trait {
            instance.trait_identity == owner
        } else {
            instance.argument_identities.iter().any(|argument| {
                argument == owner
                    || argument
                        .strip_prefix(owner)
                        .is_some_and(|suffix| suffix.starts_with('<'))
            })
        }
    })
}

fn local_instance_conflicts(
    resolved: &ResolvedModule,
    local_instances: &[TypedInstance],
) -> Vec<DerivedInstanceIssue> {
    let heads = local_instances
        .iter()
        .map(|instance| contracts::local_head(resolved, instance))
        .collect::<Vec<_>>();
    let mut issues = Vec::new();
    for (index, instance) in local_instances.iter().enumerate() {
        let first = local_instances[..index]
            .iter()
            .enumerate()
            .find_map(|(previous, first)| {
                let exact =
                    canonical_instance_head_key(&first.trait_identity, &first.argument_identities)
                        == canonical_instance_head_key(
                            &instance.trait_identity,
                            &instance.argument_identities,
                        );
                if !same_trait(&first.trait_identity, &instance.trait_identity) {
                    return None;
                }
                let relation = if exact {
                    HeadRelation::Overlap
                } else {
                    match (&heads[previous], &heads[index]) {
                        (Some(left), Some(right)) => {
                            left.relation(right, &canonical_trait(&instance.trait_identity))
                        }
                        _ => HeadRelation::Unproved,
                    }
                };
                (relation != HeadRelation::Disjoint).then_some((first, relation))
            });
        if let Some((first, relation)) = first {
            let issue = if relation == HeadRelation::Unproved {
                DerivedInstanceIssue::UnprovedInstanceCoherence {
                    trait_name: instance.trait_name.clone(),
                    primary: instance.origin,
                    first: Some(first.origin),
                    provider_module: None,
                }
            } else if first.argument_identities == instance.argument_identities {
                DerivedInstanceIssue::DuplicateLocalInstance {
                    trait_name: instance.trait_name.clone(),
                    identity: instance.identity.clone(),
                    primary: instance.origin,
                    first: first.origin,
                }
            } else {
                DerivedInstanceIssue::OverlappingLocalInstance {
                    trait_name: instance.trait_name.clone(),
                    identity: instance.identity.clone(),
                    primary: instance.origin,
                    first: first.origin,
                }
            };
            issues.push(issue);
        }
    }
    issues
}

pub(crate) fn same_trait(left: &str, right: &str) -> bool {
    canonical_instance_head_key(left, &[]) == canonical_instance_head_key(right, &[])
}

pub(crate) fn canonical_trait(identity: &str) -> String {
    if !identity.contains("::") && crate::prelude::trait_by_name(identity).is_some() {
        format!("std/prelude::{identity}")
    } else {
        identity.to_owned()
    }
}

fn standard_instance_conflicts(local_instances: &[TypedInstance]) -> Vec<DerivedInstanceIssue> {
    local_instances
        .iter()
        .filter_map(|local| {
            if crate::prelude::trait_by_canonical(&local.trait_identity)
                .is_some_and(|trait_spec| trait_spec.name == local.trait_name)
            {
                let constraint = crate::TypedConstraint {
                    name: local.trait_name.clone(),
                    arguments: local.arguments.clone(),
                };
                if let Some(standard) = crate::prelude::special_standard_instance(&constraint) {
                    return Some(DerivedInstanceIssue::OverlappingStandardInstance {
                        trait_name: local.trait_name.clone(),
                        standard_identity: standard.identity.to_owned(),
                        primary: local.origin,
                    });
                }
            }
            let [argument] = local.arguments.as_slice() else {
                return None;
            };
            let [canonical_argument] = local.argument_identities.as_slice() else {
                return None;
            };
            if let Some(identity) = crate::prelude::structural_standard_instance_identity(
                &local.trait_identity,
                argument,
            ) {
                return Some(DerivedInstanceIssue::OverlappingStandardInstance {
                    trait_name: local.trait_name.clone(),
                    standard_identity: identity.to_owned(),
                    primary: local.origin,
                });
            }
            crate::prelude::overlapping_standard_instance(
                &local.trait_identity,
                argument,
                canonical_argument,
            )
            .map(
                |standard| DerivedInstanceIssue::OverlappingStandardInstance {
                    trait_name: local.trait_name.clone(),
                    standard_identity: standard.identity.to_owned(),
                    primary: local.origin,
                },
            )
        })
        .collect()
}

fn local_dependency_conflicts(
    resolved: &ResolvedModule,
    local_instances: &[TypedInstance],
) -> Vec<DerivedInstanceIssue> {
    let imported_heads = resolved
        .dependency_instances
        .iter()
        .map(dependency_head)
        .collect::<Vec<_>>();
    local_instances
        .iter()
        .filter_map(|local| {
            let local_head = contracts::local_head(resolved, local);
            resolved
                .dependency_instances
                .iter()
                .zip(&imported_heads)
                .find_map(|(imported, imported_head)| {
                    if !same_trait(&imported.trait_identity, &local.trait_identity) {
                        return None;
                    }
                    let exact = canonical_instance_head_key(
                        &imported.trait_identity,
                        &imported.argument_identities,
                    ) == canonical_instance_head_key(
                        &local.trait_identity,
                        &local.argument_identities,
                    );
                    let relation = if exact {
                        HeadRelation::Overlap
                    } else {
                        match (&local_head, imported_head) {
                            (Some(left), Some(right)) => {
                                left.relation(right, &canonical_trait(&local.trait_identity))
                            }
                            _ => HeadRelation::Unproved,
                        }
                    };
                    match relation {
                        HeadRelation::Disjoint => None,
                        HeadRelation::Unproved => {
                            Some(DerivedInstanceIssue::UnprovedInstanceCoherence {
                                trait_name: local.trait_name.clone(),
                                primary: local.origin,
                                first: None,
                                provider_module: Some(imported.provider_module.clone()),
                            })
                        }
                        HeadRelation::Overlap => Some(DerivedInstanceIssue::AmbiguousInstance {
                            trait_name: local.trait_name.clone(),
                            type_identity: local
                                .type_identity
                                .clone()
                                .unwrap_or_else(|| local.argument_identities.join(",")),
                            provider_module: imported.provider_module.clone(),
                            primary: local.origin,
                        }),
                    }
                })
        })
        .collect()
}
