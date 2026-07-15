use crate::{source_span, CoreCallEvidence, CoreType, SourceSpan};
use serde::{Deserialize, Serialize};
use seseragi_semantics::{
    TypedConstraint, TypedInstance, TypedInstanceEvidence, TypedInstanceImplementation,
    TypedInstanceMethod, TypedShowPayloadEvidence,
};

use super::expr::{lower_expr, lower_parameter};
use super::types::lower_typed_type;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreInstance {
    pub identity: String,
    #[serde(rename = "trait")]
    pub trait_name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_parameters: Vec<String>,
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
    UserDefined {
        methods: Vec<CoreInstanceMethod>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreInstanceMethod {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_parameters: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<CoreInstanceConstraint>,
    pub parameters: Vec<super::CoreParameter>,
    pub body: super::CoreExpr,
    pub origin: SourceSpan,
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
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        type_arguments: Vec<CoreType>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        evidence_arguments: Vec<CoreCallEvidence>,
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
        type_parameters: instance.type_parameters,
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
            TypedInstanceImplementation::UserDefined { methods } => {
                CoreInstanceImplementation::UserDefined {
                    methods: methods
                        .into_iter()
                        .map(|method| lower_method(source, method))
                        .collect(),
                }
            }
        },
    }
}

fn lower_method(source: &str, method: TypedInstanceMethod) -> CoreInstanceMethod {
    CoreInstanceMethod {
        name: method.name,
        type_parameters: method.scheme.type_parameters,
        constraints: method
            .scheme
            .constraints
            .into_iter()
            .map(lower_constraint)
            .collect(),
        parameters: method.parameters.iter().map(lower_parameter).collect(),
        body: lower_expr(source, method.body),
        origin: source_span(source, method.origin),
    }
}

fn lower_show_payload_evidence(evidence: TypedShowPayloadEvidence) -> CoreShowPayloadEvidence {
    CoreShowPayloadEvidence {
        variant_symbol: evidence.variant_symbol,
        type_identity: evidence.type_identity,
        evidence: lower_instance_evidence(evidence.evidence),
    }
}

pub(super) fn lower_instance_evidence(evidence: TypedInstanceEvidence) -> CoreInstanceEvidence {
    match evidence {
        TypedInstanceEvidence::Local {
            identity,
            type_arguments,
            evidence_arguments,
        } => CoreInstanceEvidence::Local {
            identity,
            type_arguments: type_arguments.into_iter().map(lower_typed_type).collect(),
            evidence_arguments: evidence_arguments
                .into_iter()
                .map(|evidence| CoreCallEvidence {
                    constraint: lower_constraint(evidence.constraint),
                    evidence: lower_instance_evidence(evidence.evidence),
                })
                .collect(),
        },
        TypedInstanceEvidence::Imported {
            identity,
            provider_module,
        } => CoreInstanceEvidence::Imported {
            identity,
            provider_module,
        },
        TypedInstanceEvidence::Standard { identity } => CoreInstanceEvidence::Standard { identity },
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
