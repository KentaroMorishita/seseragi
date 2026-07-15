use crate::{SymbolNamespace, TypedInstanceEvidence, TypedShowPayloadEvidence};
use std::collections::BTreeSet;

use super::ShowPayload;
use crate::typed::instances::canonical_instance_identity;
use crate::typed::TypedResolution;

const STANDARD_SHOW_TYPES: &[&str] = &[
    "std/prelude::String",
    "std/prelude::ConsoleError",
    "std/prelude::StdinError",
];

pub(super) enum PayloadSupport {
    Supported(TypedShowPayloadEvidence),
    Unsupported,
    Unresolved,
}

pub(super) fn payload_evidence(
    resolution: &TypedResolution<'_>,
    payload: &ShowPayload,
    local_show_instances: &BTreeSet<String>,
) -> PayloadSupport {
    let seseragi_syntax::TypeRef::Named {
        arguments, span, ..
    } = &payload.type_ref
    else {
        return PayloadSupport::Unsupported;
    };
    if !arguments.is_empty() {
        return PayloadSupport::Unsupported;
    }
    let Some(type_identity) = resolution
        .target(*span, SymbolNamespace::Type)
        .and_then(|target| resolution.symbol(target))
        .and_then(|symbol| symbol.canonical.clone())
    else {
        return PayloadSupport::Unresolved;
    };
    let evidence = if STANDARD_SHOW_TYPES.contains(&type_identity.as_str()) {
        TypedInstanceEvidence::Standard {
            identity: canonical_instance_identity("Show", &type_identity),
        }
    } else if local_show_instances.contains(&type_identity) {
        TypedInstanceEvidence::Local {
            identity: canonical_instance_identity("Show", &type_identity),
            type_arguments: Vec::new(),
            evidence_arguments: Vec::new(),
        }
    } else if let Some(instance) = resolution.dependency_instance("Show", &type_identity) {
        TypedInstanceEvidence::Imported {
            identity: instance.identity.clone(),
            provider_module: instance.provider_module.clone(),
        }
    } else {
        return PayloadSupport::Unsupported;
    };
    PayloadSupport::Supported(TypedShowPayloadEvidence {
        variant_symbol: payload.variant_symbol.clone(),
        type_identity,
        evidence,
    })
}
