use crate::CoreModule;
use std::collections::BTreeMap;

use super::names::safe_identifier;
use super::{
    TypeScriptLoweringError, TypeScriptOutputPlan, TypeScriptSourceImport,
    TypeScriptSourceImportBinding,
};

mod instances;
mod names;
mod references;
mod types;

use instances::imported_instance_evidence;
use names::{fresh_name, local_type_names, local_value_names};
use references::{referenced_types, referenced_value_symbols};
use types::lower_external_type_imports;

pub(super) struct LoweredModuleImports {
    pub(super) imports: Vec<TypeScriptSourceImport>,
    pub(super) value_names: BTreeMap<String, String>,
    pub(super) type_names: BTreeMap<String, String>,
    pub(super) instance_names: BTreeMap<(String, String), String>,
}

pub(super) fn lower_module_imports(
    module: &CoreModule,
    plan: &TypeScriptOutputPlan,
) -> Result<LoweredModuleImports, TypeScriptLoweringError> {
    let referenced_values = referenced_value_symbols(module);
    let referenced_types = referenced_types(module);
    let imported_instances = imported_instance_evidence(module);
    let mut used_values = local_value_names(module);
    let mut used_types = local_type_names(module);
    let mut value_names = BTreeMap::new();
    let mut instance_names = BTreeMap::new();
    let mut imports: Vec<TypeScriptSourceImport> = Vec::new();

    for dependency in &module.module_dependencies {
        let Some(specifier) = plan.specifier_for(&dependency.module) else {
            return Err(TypeScriptLoweringError::MissingOutputSpecifier {
                module: dependency.module.clone(),
                source_specifier: dependency.specifier.clone(),
            });
        };
        let index = imports
            .iter()
            .position(|import| import.module == dependency.module)
            .unwrap_or_else(|| {
                imports.push(TypeScriptSourceImport {
                    module: dependency.module.clone(),
                    specifier: specifier.to_owned(),
                    runtime_edge: true,
                    bindings: Vec::new(),
                    origin: dependency.origin.clone(),
                });
                imports.len() - 1
            });
        let group = &mut imports[index];

        for import in &dependency.imports {
            match import.namespace.as_str() {
                "value" if referenced_values.contains(&import.canonical) => {
                    let local = value_names
                        .entry(import.canonical.clone())
                        .or_insert_with(|| {
                            fresh_name(&safe_identifier(&import.local), &used_values)
                        })
                        .clone();
                    used_values.insert(local.clone());
                    push_binding(
                        group,
                        TypeScriptSourceImportBinding {
                            imported: safe_identifier(&import.imported),
                            local,
                            source_local: import.local.clone(),
                            canonical: import.canonical.clone(),
                            type_only: false,
                            origin: import.origin.clone(),
                        },
                    );
                }
                "type" if referenced_types.names.contains(&import.local) => {
                    let local = safe_identifier(&import.local);
                    if !used_types.insert(local.clone())
                        && !group.bindings.iter().any(|binding| {
                            binding.local == local && binding.canonical == import.canonical
                        })
                    {
                        return Err(TypeScriptLoweringError::ImportNameCollision { local });
                    }
                    push_binding(
                        group,
                        TypeScriptSourceImportBinding {
                            imported: safe_identifier(&import.imported),
                            local,
                            source_local: import.local.clone(),
                            canonical: import.canonical.clone(),
                            type_only: true,
                            origin: import.origin.clone(),
                        },
                    );
                }
                _ => {}
            }
        }

        for (_, identity) in imported_instances
            .iter()
            .filter(|(provider_module, _)| provider_module == &dependency.module)
        {
            let key = (dependency.module.clone(), identity.clone());
            if instance_names.contains_key(&key) {
                continue;
            }
            let dictionary_export = plan
                .instance_export_for(&dependency.module, identity)
                .ok_or_else(|| TypeScriptLoweringError::MissingInstanceOutput {
                    module: dependency.module.clone(),
                    identity: identity.clone(),
                })?;
            let local = fresh_name(&safe_identifier(dictionary_export), &used_values);
            used_values.insert(local.clone());
            instance_names.insert(key, local.clone());
            push_binding(
                group,
                TypeScriptSourceImportBinding {
                    imported: dictionary_export.to_owned(),
                    local,
                    source_local: identity.clone(),
                    canonical: identity.clone(),
                    type_only: false,
                    origin: dependency.origin.clone(),
                },
            );
        }
    }

    let type_names = lower_external_type_imports(
        module,
        &referenced_types.external,
        plan,
        &mut imports,
        &mut used_types,
    )?;

    Ok(LoweredModuleImports {
        imports,
        value_names,
        type_names,
        instance_names,
    })
}

fn push_binding(group: &mut TypeScriptSourceImport, binding: TypeScriptSourceImportBinding) {
    if let Some(existing) = group.bindings.iter_mut().find(|existing| {
        existing.imported == binding.imported
            && existing.local == binding.local
            && existing.canonical == binding.canonical
    }) {
        existing.type_only &= binding.type_only;
        return;
    }
    group.bindings.push(binding);
}
