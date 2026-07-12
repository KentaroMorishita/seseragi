use crate::{ResolvedModule, TypedInstance};
use seseragi_syntax::ByteSpan;

use super::TypedResolution;

mod show;
#[cfg(test)]
mod tests;
mod traits;

pub(crate) use crate::instance_identity::canonical_instance_identity;

pub(crate) struct DerivedInstanceAnalysis {
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
}

pub(crate) fn analyze_derived_instances(
    resolved: &ResolvedModule,
    resolution: &TypedResolution<'_>,
) -> DerivedInstanceAnalysis {
    let mut issues = traits::unknown_trait_issues(&resolved.declarations);
    let show = show::analyze_derived_show(resolved, resolution);
    issues.extend(show.issues);
    DerivedInstanceAnalysis {
        instances: show.instances,
        issues,
    }
}
