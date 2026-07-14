use crate::{source_span, CoreType, SourceSpan};
use serde::{Deserialize, Serialize};
use seseragi_semantics::{
    TypedConstraint, TypedInstance, TypedInstanceEvidence, TypedInstanceImplementation,
    TypedShowPayloadEvidence,
};

use super::types::lower_typed_type;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreInstance {
    pub identity: String,
    #[serde(rename = "trait")]
    pub trait_name: String,
    pub arguments: Vec<CoreType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<CoreInstanceConstraint>,
    pub origin: SourceSpan,
    pub implementation: CoreInstanceImplementation,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreInstanceConstraint {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arguments: Vec<CoreType>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum CoreInstanceImplementation {
    DerivedShow {
        adt_symbol: String,
        payload_evidence: Vec<CoreShowPayloadEvidence>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreShowPayloadEvidence {
    pub variant_symbol: String,
    pub type_identity: String,
    pub evidence: CoreInstanceEvidence,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum CoreInstanceEvidence {
    Local {
        identity: String,
    },
    Imported {
        identity: String,
        provider_module: String,
    },
    Standard {
        identity: String,
    },
}

pub(super) fn lower_instances(source: &str, instances: Vec<TypedInstance>) -> Vec<CoreInstance> {
    instances
        .into_iter()
        .map(|instance| lower_instance(source, instance))
        .collect()
}

fn lower_instance(source: &str, instance: TypedInstance) -> CoreInstance {
    CoreInstance {
        identity: instance.identity,
        trait_name: instance.trait_name,
        arguments: instance
            .arguments
            .into_iter()
            .map(lower_typed_type)
            .collect(),
        type_identity: instance.type_identity,
        constraints: instance
            .constraints
            .into_iter()
            .map(lower_constraint)
            .collect(),
        origin: source_span(source, instance.origin),
        implementation: match instance.implementation {
            TypedInstanceImplementation::DerivedShow {
                adt_symbol,
                payload_evidence,
            } => CoreInstanceImplementation::DerivedShow {
                adt_symbol,
                payload_evidence: payload_evidence
                    .into_iter()
                    .map(lower_show_payload_evidence)
                    .collect(),
            },
        },
    }
}

fn lower_show_payload_evidence(evidence: TypedShowPayloadEvidence) -> CoreShowPayloadEvidence {
    CoreShowPayloadEvidence {
        variant_symbol: evidence.variant_symbol,
        type_identity: evidence.type_identity,
        evidence: match evidence.evidence {
            TypedInstanceEvidence::Local { identity } => CoreInstanceEvidence::Local { identity },
            TypedInstanceEvidence::Imported {
                identity,
                provider_module,
            } => CoreInstanceEvidence::Imported {
                identity,
                provider_module,
            },
            TypedInstanceEvidence::Standard { identity } => {
                CoreInstanceEvidence::Standard { identity }
            }
        },
    }
}

pub(super) fn lower_constraint(constraint: TypedConstraint) -> CoreInstanceConstraint {
    CoreInstanceConstraint {
        name: constraint.name,
        arguments: constraint
            .arguments
            .into_iter()
            .map(lower_typed_type)
            .collect(),
    }
}

#[cfg(test)]
mod tests;
