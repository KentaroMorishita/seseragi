use std::collections::BTreeMap;

use crate::{show_ops::runtime_show_dictionary_for_identity, CoreInstanceEvidence};

use super::super::{push_import_unique, push_unique, TypeScriptImport};
use super::TypeScriptShowDictionaryReference;

pub(super) fn resolve_show_dictionary(
    evidence: &CoreInstanceEvidence,
    dictionary_exports: &BTreeMap<&str, String>,
    imported_instance_names: &BTreeMap<(String, String), String>,
    runtime_requirements: &mut Vec<String>,
    imports: &mut Vec<TypeScriptImport>,
) -> TypeScriptShowDictionaryReference {
    match evidence {
        CoreInstanceEvidence::Local { identity, .. } => {
            let dictionary_export = dictionary_exports
                .get(identity.as_str())
                .expect("selected local instance identity must have a dictionary export");
            TypeScriptShowDictionaryReference::Local {
                identity: identity.clone(),
                dictionary_export: dictionary_export.clone(),
            }
        }
        CoreInstanceEvidence::Imported {
            identity,
            provider_module,
        } => {
            let local = imported_instance_names
                .get(&(provider_module.clone(), identity.clone()))
                .expect("planned imported instance must have a source import local");
            TypeScriptShowDictionaryReference::Imported {
                identity: identity.clone(),
                provider_module: provider_module.clone(),
                local: local.clone(),
            }
        }
        CoreInstanceEvidence::Standard { identity } => {
            let dictionary = runtime_show_dictionary_for_identity(identity)
                .expect("selected standard Show identity must be registered");
            push_unique(runtime_requirements, dictionary.runtime_feature);
            push_import_unique(
                imports,
                TypeScriptImport {
                    feature: dictionary.runtime_feature.to_owned(),
                    local: dictionary.local_name.to_owned(),
                },
            );
            TypeScriptShowDictionaryReference::Runtime {
                identity: identity.clone(),
                feature: dictionary.runtime_feature.to_owned(),
                local: dictionary.local_name.to_owned(),
            }
        }
        CoreInstanceEvidence::Parameter { .. } => {
            unreachable!("derived Show payload evidence cannot reference a scoped parameter")
        }
    }
}
