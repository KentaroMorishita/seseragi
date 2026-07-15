use crate::{ResolvedModule, TypedInstance};
use seseragi_syntax::ByteSpan;

use super::TypedResolution;

mod contracts;
mod show;
#[cfg(test)]
mod tests;
mod traits;
mod user;

pub(crate) use crate::instance_identity::{
    canonical_instance_head_identity, canonical_instance_identity,
};
pub(crate) use contracts::{analyze_instance_contracts, InstanceContractIssue};
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
    UnsupportedGenericShow {
        type_name: String,
        primary: ByteSpan,
        declaration: ByteSpan,
    },
    UnsupportedShowPayload {
        payload_name: String,
        primary: ByteSpan,
        declaration: ByteSpan,
    },
    AmbiguousInstance {
        trait_name: String,
        type_identity: String,
        provider_module: String,
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
    issues.extend(local_dependency_conflicts(resolved, &instances));
    InstanceAnalysis { instances, issues }
}

fn local_dependency_conflicts(
    resolved: &ResolvedModule,
    local_instances: &[TypedInstance],
) -> Vec<DerivedInstanceIssue> {
    local_instances
        .iter()
        .filter_map(|local| {
            resolved
                .dependency_instances
                .iter()
                .find(|imported| {
                    imported.trait_name == local.trait_name
                        && local.type_identity.as_deref() == Some(imported.type_identity.as_str())
                })
                .map(|imported| DerivedInstanceIssue::AmbiguousInstance {
                    trait_name: local.trait_name.clone(),
                    type_identity: local
                        .type_identity
                        .clone()
                        .expect("derived Show instance must retain its primary type identity"),
                    provider_module: imported.provider_module.clone(),
                    primary: local.origin,
                })
        })
        .collect()
}
