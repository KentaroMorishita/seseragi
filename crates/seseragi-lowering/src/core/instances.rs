use crate::{source_span, CoreType, SourceSpan};
use serde::{Deserialize, Serialize};
use seseragi_semantics::{TypedConstraint, TypedInstance, TypedInstanceImplementation};

use super::types::lower_typed_type;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreInstance {
    #[serde(rename = "trait")]
    pub trait_name: String,
    pub head: CoreType,
    pub type_identity: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<CoreInstanceConstraint>,
    pub origin: SourceSpan,
    pub implementation: CoreInstanceImplementation,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreInstanceConstraint {
    pub name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum CoreInstanceImplementation {
    DerivedShow { adt_symbol: String },
}

pub(super) fn lower_instances(source: &str, instances: Vec<TypedInstance>) -> Vec<CoreInstance> {
    instances
        .into_iter()
        .map(|instance| lower_instance(source, instance))
        .collect()
}

fn lower_instance(source: &str, instance: TypedInstance) -> CoreInstance {
    CoreInstance {
        trait_name: instance.trait_name,
        head: lower_typed_type(instance.head),
        type_identity: instance.type_identity,
        constraints: instance
            .constraints
            .into_iter()
            .map(lower_constraint)
            .collect(),
        origin: source_span(source, instance.origin),
        implementation: match instance.implementation {
            TypedInstanceImplementation::DerivedShow { adt_symbol } => {
                CoreInstanceImplementation::DerivedShow { adt_symbol }
            }
        },
    }
}

fn lower_constraint(constraint: TypedConstraint) -> CoreInstanceConstraint {
    CoreInstanceConstraint {
        name: constraint.name,
    }
}

#[cfg(test)]
mod tests;
