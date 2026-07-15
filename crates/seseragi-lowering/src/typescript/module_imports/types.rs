use crate::{CoreModule, SourceSpan};
use std::collections::{BTreeMap, BTreeSet};

use super::super::names::safe_identifier;
use super::names::fresh_name;
use super::{push_binding, TypeScriptLoweringError, TypeScriptOutputPlan};
use crate::{TypeScriptSourceImport, TypeScriptSourceImportBinding};

pub(super) fn lower_external_type_imports(
    module: &CoreModule,
    referenced: &BTreeSet<String>,
    plan: &TypeScriptOutputPlan,
    imports: &mut Vec<TypeScriptSourceImport>,
    used_types: &mut BTreeSet<String>,
) -> Result<BTreeMap<String, String>, TypeScriptLoweringError> {
    let mut names = BTreeMap::new();
    for canonical in referenced {
        if let Some(existing) = imports
            .iter()
            .flat_map(|group| &group.bindings)
            .find(|binding| binding.type_only && binding.canonical == *canonical)
        {
            names.insert(canonical.clone(), existing.local.clone());
            continue;
        }

        let candidates = module
            .external_type_bindings
            .iter()
            .filter(|binding| binding.canonical == *canonical)
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            return Err(TypeScriptLoweringError::MissingExternalTypeBinding {
                canonical: canonical.clone(),
            });
        }
        let providers = candidates
            .iter()
            .filter_map(|binding| binding.provider.as_ref())
            .map(|provider| (provider.module.as_str(), provider.export.as_str()))
            .collect::<BTreeSet<_>>();
        let mut providers = providers.into_iter();
        let (provider_module, provider_export) = match (providers.next(), providers.next()) {
            (None, _) => {
                return Err(TypeScriptLoweringError::MissingSourceTypeProvider {
                    canonical: canonical.clone(),
                });
            }
            (Some(provider), None) => provider,
            (Some(_), Some(_)) => {
                return Err(TypeScriptLoweringError::AmbiguousSourceTypeProvider {
                    canonical: canonical.clone(),
                });
            }
        };
        let specifier = plan.specifier_for(provider_module).ok_or_else(|| {
            TypeScriptLoweringError::MissingTypeOutputSpecifier {
                module: provider_module.to_owned(),
                canonical: canonical.clone(),
            }
        })?;
        let spelling = candidates
            .iter()
            .find(|binding| binding.provider.is_some())
            .map(|binding| binding.spelling.as_str())
            .expect("a source provider candidate was selected");
        let local = fresh_name(&safe_identifier(spelling), used_types);
        used_types.insert(local.clone());
        names.insert(canonical.clone(), local.clone());

        let index = imports
            .iter()
            .position(|group| group.module == provider_module)
            .unwrap_or_else(|| {
                imports.push(TypeScriptSourceImport {
                    module: provider_module.to_owned(),
                    specifier: specifier.to_owned(),
                    runtime_edge: false,
                    bindings: Vec::new(),
                    origin: inferred_import_origin(module),
                });
                imports.len() - 1
            });
        let origin = imports[index].origin.clone();
        push_binding(
            &mut imports[index],
            TypeScriptSourceImportBinding {
                imported: safe_identifier(provider_export),
                local,
                source_local: spelling.to_owned(),
                canonical: canonical.clone(),
                type_only: true,
                origin,
            },
        );
    }
    Ok(names)
}

pub(super) fn inferred_import_origin(module: &CoreModule) -> SourceSpan {
    module
        .adts
        .first()
        .map(|declaration| declaration.origin.clone())
        .or_else(|| {
            module
                .instances
                .first()
                .map(|instance| instance.origin.clone())
        })
        .or_else(|| {
            module
                .bindings
                .first()
                .map(|binding| binding.origin.clone())
        })
        .or_else(|| {
            module
                .functions
                .first()
                .map(|function| function.origin.clone())
        })
        .unwrap_or_else(|| SourceSpan {
            source: module.module.clone(),
            start: 0,
            end: 0,
        })
}

#[cfg(test)]
mod tests;
