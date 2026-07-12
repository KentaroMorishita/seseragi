use crate::{CoreInstanceEvidence, CoreInstanceImplementation, CoreModule};
use std::collections::BTreeSet;

pub(super) fn imported_instance_evidence(module: &CoreModule) -> BTreeSet<(String, String)> {
    module
        .instances
        .iter()
        .flat_map(|instance| match &instance.implementation {
            CoreInstanceImplementation::DerivedShow {
                payload_evidence, ..
            } => payload_evidence.as_slice(),
        })
        .filter_map(|payload| match &payload.evidence {
            CoreInstanceEvidence::Imported {
                identity,
                provider_module,
            } => Some((provider_module.clone(), identity.clone())),
            CoreInstanceEvidence::Local { .. } | CoreInstanceEvidence::Standard { .. } => None,
        })
        .collect()
}
