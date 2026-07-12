use std::collections::BTreeMap;

use crate::{CoreAdt, CoreInstance, CoreInstanceImplementation, SourceSpan};
use serde::{Deserialize, Serialize};

use super::names::{local_name, safe_identifier};
use super::types::type_ref_from_core_type;
use super::{push_unique, TypeScriptImport, TypeScriptType, TypeScriptTypeImport};

mod evidence;

use evidence::resolve_show_dictionary;

const SHOW_DICTIONARY_FEATURE: &str = "core.show.dictionary";
const SHOW_DICTIONARY_TYPE_LOCAL: &str = "_ssrg_show_Show";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptInstance {
    pub identity: String,
    #[serde(rename = "trait")]
    pub trait_name: String,
    pub head: TypeScriptType,
    pub type_identity: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<TypeScriptInstanceConstraint>,
    pub origin: SourceSpan,
    pub dictionary_export: String,
    pub implementation: TypeScriptInstanceImplementation,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptInstanceConstraint {
    pub name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypeScriptInstanceImplementation {
    DerivedShow {
        adt_name: String,
        variants: Vec<TypeScriptDerivedShowVariant>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptDerivedShowVariant {
    pub name: String,
    pub tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<TypeScriptDerivedShowPayload>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptDerivedShowPayload {
    #[serde(rename = "type")]
    pub type_ref: TypeScriptType,
    pub dictionary: TypeScriptShowDictionaryReference,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypeScriptShowDictionaryReference {
    Runtime {
        identity: String,
        feature: String,
        local: String,
    },
    Local {
        identity: String,
        dictionary_export: String,
    },
    Imported {
        identity: String,
        provider_module: String,
        local: String,
    },
}

pub(super) fn lower_core_instances_to_typescript(
    instances: &[CoreInstance],
    adts: &[CoreAdt],
    imported_instance_names: &BTreeMap<(String, String), String>,
    runtime_requirements: &mut Vec<String>,
    imports: &mut Vec<TypeScriptImport>,
    type_imports: &mut Vec<TypeScriptTypeImport>,
) -> Vec<TypeScriptInstance> {
    if instances.is_empty() {
        return Vec::new();
    }

    push_unique(runtime_requirements, SHOW_DICTIONARY_FEATURE);
    push_type_import_unique(
        type_imports,
        TypeScriptTypeImport {
            feature: SHOW_DICTIONARY_FEATURE.to_owned(),
            local: SHOW_DICTIONARY_TYPE_LOCAL.to_owned(),
        },
    );

    let dictionary_exports = instances
        .iter()
        .enumerate()
        .map(|(index, instance)| {
            (
                instance.identity.as_str(),
                dictionary_export_name(&instance.trait_name, index),
            )
        })
        .collect::<BTreeMap<_, _>>();

    instances
        .iter()
        .enumerate()
        .map(|(index, instance)| {
            lower_instance(
                index,
                instance,
                adts,
                &dictionary_exports,
                imported_instance_names,
                runtime_requirements,
                imports,
            )
        })
        .collect()
}

fn lower_instance(
    index: usize,
    instance: &CoreInstance,
    adts: &[CoreAdt],
    dictionary_exports: &BTreeMap<&str, String>,
    imported_instance_names: &BTreeMap<(String, String), String>,
    runtime_requirements: &mut Vec<String>,
    imports: &mut Vec<TypeScriptImport>,
) -> TypeScriptInstance {
    let implementation = match &instance.implementation {
        CoreInstanceImplementation::DerivedShow {
            adt_symbol,
            payload_evidence,
        } => {
            let adt = adts
                .iter()
                .find(|adt| adt.symbol == *adt_symbol)
                .expect("selected DerivedShow instance must reference a lowered ADT");
            TypeScriptInstanceImplementation::DerivedShow {
                adt_name: local_name(&adt.symbol),
                variants: adt
                    .variants
                    .iter()
                    .map(|variant| TypeScriptDerivedShowVariant {
                        name: local_name(&variant.symbol),
                        tag: variant.name.clone(),
                        payload: variant.payload.as_ref().map(|payload| {
                            let evidence = payload_evidence
                                .iter()
                                .find(|evidence| evidence.variant_symbol == variant.symbol)
                                .expect("selected DerivedShow payload must retain typed evidence");
                            TypeScriptDerivedShowPayload {
                                type_ref: type_ref_from_core_type(payload),
                                dictionary: resolve_show_dictionary(
                                    &evidence.evidence,
                                    dictionary_exports,
                                    imported_instance_names,
                                    runtime_requirements,
                                    imports,
                                ),
                            }
                        }),
                    })
                    .collect(),
            }
        }
    };

    TypeScriptInstance {
        identity: instance.identity.clone(),
        trait_name: instance.trait_name.clone(),
        head: type_ref_from_core_type(&instance.head),
        type_identity: instance.type_identity.clone(),
        constraints: instance
            .constraints
            .iter()
            .map(|constraint| TypeScriptInstanceConstraint {
                name: constraint.name.clone(),
            })
            .collect(),
        origin: instance.origin.clone(),
        dictionary_export: dictionary_export_name(&instance.trait_name, index),
        implementation,
    }
}

fn dictionary_export_name(trait_name: &str, index: usize) -> String {
    format!("__ssrg$instance${}${index}", safe_identifier(trait_name))
}

fn push_type_import_unique(imports: &mut Vec<TypeScriptTypeImport>, import: TypeScriptTypeImport) {
    if !imports
        .iter()
        .any(|existing| existing.feature == import.feature && existing.local == import.local)
    {
        imports.push(import);
    }
}

#[cfg(test)]
mod tests;
