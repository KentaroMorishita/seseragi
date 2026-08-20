use std::collections::BTreeMap;

use crate::prelude_ops::runtime_prelude_dictionary_for_identity;
use crate::{
    display_ops::runtime_display_dictionary_for_identity, CoreInstanceEvidence, TypeScriptExpr,
};

use super::super::{push_import_unique, push_unique, TypeScriptImport};
use super::TypeScriptShowDictionaryReference;

pub(super) fn resolve_show_dictionary(
    evidence: &CoreInstanceEvidence,
    dictionary_exports: &BTreeMap<&str, String>,
    imported_instance_names: &BTreeMap<(String, String), String>,
    expression_value_names: &BTreeMap<String, String>,
    imported_type_names: &BTreeMap<String, String>,
    runtime_requirements: &mut Vec<String>,
    imports: &mut Vec<TypeScriptImport>,
) -> TypeScriptShowDictionaryReference {
    match evidence {
        CoreInstanceEvidence::Local {
            identity,
            type_arguments,
            evidence_arguments,
        } if type_arguments.is_empty() && evidence_arguments.is_empty() => {
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
            type_arguments,
            evidence_arguments,
        } if type_arguments.is_empty() && evidence_arguments.is_empty() => {
            let local = imported_instance_names
                .get(&(provider_module.clone(), identity.clone()))
                .expect("planned imported instance must have a source import local");
            TypeScriptShowDictionaryReference::Imported {
                identity: identity.clone(),
                provider_module: provider_module.clone(),
                local: local.clone(),
            }
        }
        CoreInstanceEvidence::Standard {
            identity,
            type_arguments,
            evidence_arguments,
        } if type_arguments.is_empty() && evidence_arguments.is_empty() => {
            let dictionary = runtime_display_dictionary_for_identity(identity)
                .map(|dictionary| (dictionary.runtime_feature, dictionary.local_name))
                .or_else(|| {
                    runtime_prelude_dictionary_for_identity(identity)
                        .map(|dictionary| (dictionary.runtime_feature, dictionary.local_name))
                })
                .expect("selected standard derived identity must be registered");
            push_unique(runtime_requirements, dictionary.0);
            push_import_unique(
                imports,
                TypeScriptImport {
                    feature: dictionary.0.to_owned(),
                    local: dictionary.1.to_owned(),
                },
            );
            TypeScriptShowDictionaryReference::Runtime {
                identity: identity.clone(),
                feature: dictionary.0.to_owned(),
                local: dictionary.1.to_owned(),
            }
        }
        CoreInstanceEvidence::Local {
            evidence_arguments, ..
        }
        | CoreInstanceEvidence::Imported {
            evidence_arguments, ..
        }
        | CoreInstanceEvidence::Standard {
            evidence_arguments, ..
        } => {
            collect_evidence_runtime(evidence_arguments, runtime_requirements, imports);
            if let CoreInstanceEvidence::Standard { identity, .. } = evidence {
                collect_standard_runtime(identity, runtime_requirements, imports);
            }
            let expression = super::super::dictionaries::local_dictionary_expression(
                evidence,
                expression_value_names,
                imported_type_names,
            )
            .expect("selected derived display evidence must materialize a dictionary");
            TypeScriptShowDictionaryReference::Expression {
                expression: Box::new(expression),
            }
        }
        CoreInstanceEvidence::Parameter { index } => {
            TypeScriptShowDictionaryReference::Expression {
                expression: Box::new(TypeScriptExpr::Identifier {
                    name: super::evidence_parameter_name(*index),
                }),
            }
        }
    }
}

fn collect_evidence_runtime(
    evidence: &[crate::CoreCallEvidence],
    runtime_requirements: &mut Vec<String>,
    imports: &mut Vec<TypeScriptImport>,
) {
    for selected in evidence {
        match &selected.evidence {
            CoreInstanceEvidence::Local {
                evidence_arguments, ..
            }
            | CoreInstanceEvidence::Imported {
                evidence_arguments, ..
            } => collect_evidence_runtime(evidence_arguments, runtime_requirements, imports),
            CoreInstanceEvidence::Standard {
                identity,
                evidence_arguments,
                ..
            } => {
                collect_evidence_runtime(evidence_arguments, runtime_requirements, imports);
                collect_standard_runtime(identity, runtime_requirements, imports);
            }
            CoreInstanceEvidence::Parameter { .. } => {}
        }
    }
}

fn collect_standard_runtime(
    identity: &str,
    runtime_requirements: &mut Vec<String>,
    imports: &mut Vec<TypeScriptImport>,
) {
    let dictionary = runtime_display_dictionary_for_identity(identity)
        .map(|dictionary| (dictionary.runtime_feature, dictionary.local_name))
        .or_else(|| {
            runtime_prelude_dictionary_for_identity(identity)
                .map(|dictionary| (dictionary.runtime_feature, dictionary.local_name))
        })
        .expect("selected standard derived identity must be registered");
    push_unique(runtime_requirements, dictionary.0);
    push_import_unique(
        imports,
        TypeScriptImport {
            feature: dictionary.0.to_owned(),
            local: dictionary.1.to_owned(),
        },
    );
}
