use std::collections::BTreeMap;

use crate::{CoreAdt, CoreInstance, CoreInstanceImplementation, CoreInstanceMethod, SourceSpan};
use serde::{Deserialize, Serialize};

use super::names::{local_name, safe_identifier};
use super::runtime::{
    collect_expr_runtime_imports, collect_expr_runtime_requirements,
    collect_type_runtime_requirement,
};
use super::types::{lower_core_parameter_to_typescript, type_ref_from_core_type};
use super::{
    expr::{lower_core_expr_to_typescript, typescript_expr_contains_await},
    push_unique, TypeScriptExpr, TypeScriptImport, TypeScriptParameter, TypeScriptType,
    TypeScriptTypeImport,
};

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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_parameters: Vec<String>,
    pub arguments: Vec<TypeScriptType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_identity: Option<String>,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arguments: Vec<TypeScriptType>,
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
    UserDefined {
        methods: Vec<TypeScriptInstanceMethod>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptInstanceMethod {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_parameters: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<TypeScriptInstanceConstraint>,
    #[serde(default, skip_serializing_if = "super::is_false")]
    pub is_async: bool,
    pub parameters: Vec<TypeScriptParameter>,
    pub body: TypeScriptExpr,
    pub origin: SourceSpan,
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
    imported_value_names: &BTreeMap<String, String>,
    imported_type_names: &BTreeMap<String, String>,
    runtime_requirements: &mut Vec<String>,
    imports: &mut Vec<TypeScriptImport>,
    type_imports: &mut Vec<TypeScriptTypeImport>,
) -> Vec<TypeScriptInstance> {
    if instances.is_empty() {
        return Vec::new();
    }

    if instances.iter().any(|instance| {
        matches!(
            instance.implementation,
            CoreInstanceImplementation::DerivedShow { .. }
        )
    }) {
        push_unique(runtime_requirements, SHOW_DICTIONARY_FEATURE);
        push_type_import_unique(
            type_imports,
            TypeScriptTypeImport {
                feature: SHOW_DICTIONARY_FEATURE.to_owned(),
                local: SHOW_DICTIONARY_TYPE_LOCAL.to_owned(),
            },
        );
    }

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
    let context = InstanceLoweringContext {
        adts,
        dictionary_exports: &dictionary_exports,
        imported_instance_names,
        imported_value_names,
        imported_type_names,
    };

    instances
        .iter()
        .enumerate()
        .map(|(index, instance)| {
            lower_instance(index, instance, &context, runtime_requirements, imports)
        })
        .collect()
}

struct InstanceLoweringContext<'a> {
    adts: &'a [CoreAdt],
    dictionary_exports: &'a BTreeMap<&'a str, String>,
    imported_instance_names: &'a BTreeMap<(String, String), String>,
    imported_value_names: &'a BTreeMap<String, String>,
    imported_type_names: &'a BTreeMap<String, String>,
}

fn lower_instance(
    index: usize,
    instance: &CoreInstance,
    context: &InstanceLoweringContext<'_>,
    runtime_requirements: &mut Vec<String>,
    imports: &mut Vec<TypeScriptImport>,
) -> TypeScriptInstance {
    let implementation = match &instance.implementation {
        CoreInstanceImplementation::DerivedShow {
            adt_symbol,
            payload_evidence,
        } => {
            let adt = context
                .adts
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
                                type_ref: type_ref_from_core_type(
                                    payload,
                                    context.imported_type_names,
                                ),
                                dictionary: resolve_show_dictionary(
                                    &evidence.evidence,
                                    context.dictionary_exports,
                                    context.imported_instance_names,
                                    runtime_requirements,
                                    imports,
                                ),
                            }
                        }),
                    })
                    .collect(),
            }
        }
        CoreInstanceImplementation::UserDefined { methods } => {
            TypeScriptInstanceImplementation::UserDefined {
                methods: methods
                    .iter()
                    .map(|method| lower_method(method, context, runtime_requirements, imports))
                    .collect(),
            }
        }
    };

    TypeScriptInstance {
        identity: instance.identity.clone(),
        trait_name: instance.trait_name.clone(),
        type_parameters: instance.type_parameters.clone(),
        arguments: instance
            .arguments
            .iter()
            .map(|argument| type_ref_from_core_type(argument, context.imported_type_names))
            .collect(),
        type_identity: instance.type_identity.clone(),
        constraints: instance
            .constraints
            .iter()
            .map(|constraint| TypeScriptInstanceConstraint {
                name: constraint.name.clone(),
                arguments: constraint
                    .arguments
                    .iter()
                    .map(|argument| type_ref_from_core_type(argument, context.imported_type_names))
                    .collect(),
            })
            .collect(),
        origin: instance.origin.clone(),
        dictionary_export: dictionary_export_name(&instance.trait_name, index),
        implementation,
    }
}

fn lower_method(
    method: &CoreInstanceMethod,
    context: &InstanceLoweringContext<'_>,
    runtime_requirements: &mut Vec<String>,
    imports: &mut Vec<TypeScriptImport>,
) -> TypeScriptInstanceMethod {
    for parameter in &method.parameters {
        collect_type_runtime_requirement(&parameter.type_ref, runtime_requirements);
    }
    collect_expr_runtime_requirements(&method.body, runtime_requirements);
    collect_expr_runtime_imports(&method.body, imports);
    let body = lower_core_expr_to_typescript(
        method.body.clone(),
        context.imported_value_names,
        context.imported_type_names,
    );
    TypeScriptInstanceMethod {
        name: method.name.clone(),
        type_parameters: method.type_parameters.clone(),
        constraints: method
            .constraints
            .iter()
            .map(|constraint| TypeScriptInstanceConstraint {
                name: constraint.name.clone(),
                arguments: constraint
                    .arguments
                    .iter()
                    .map(|argument| type_ref_from_core_type(argument, context.imported_type_names))
                    .collect(),
            })
            .collect(),
        is_async: typescript_expr_contains_await(&body),
        parameters: method
            .parameters
            .iter()
            .cloned()
            .map(|parameter| {
                lower_core_parameter_to_typescript(parameter, context.imported_type_names)
            })
            .collect(),
        body,
        origin: method.origin.clone(),
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
