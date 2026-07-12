use crate::CoreModule;
use std::collections::BTreeMap;

use super::names::safe_identifier;
use super::{
    TypeScriptLoweringError, TypeScriptOutputPlan, TypeScriptSourceImport,
    TypeScriptSourceImportBinding,
};

mod names;
mod references;

use names::{fresh_name, local_type_names, local_value_names};
use references::{referenced_type_names, referenced_value_symbols};

pub(super) struct LoweredModuleImports {
    pub(super) imports: Vec<TypeScriptSourceImport>,
    pub(super) value_names: BTreeMap<String, String>,
}

pub(super) fn lower_module_imports(
    module: &CoreModule,
    plan: &TypeScriptOutputPlan,
) -> Result<LoweredModuleImports, TypeScriptLoweringError> {
    let referenced_values = referenced_value_symbols(module);
    let referenced_types = referenced_type_names(module);
    let mut used_values = local_value_names(module);
    let mut used_types = local_type_names(module);
    let mut value_names = BTreeMap::new();
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
                "type" if referenced_types.contains(&import.local) => {
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
    }

    Ok(LoweredModuleImports {
        imports,
        value_names,
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
