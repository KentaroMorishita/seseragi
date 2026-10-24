use crate::{project::at, RustBackendError};
use seseragi_lowering::{CoreModule, SourceSpan};
use std::{collections::BTreeMap, sync::OnceLock};

type Names = BTreeMap<(String, String), String>;

struct StandardTypes {
    arities: BTreeMap<String, usize>,
    providers: Names,
}

fn standard_types() -> &'static StandardTypes {
    static TYPES: OnceLock<StandardTypes> = OnceLock::new();
    TYPES.get_or_init(|| {
        let mut types = StandardTypes {
            arities: BTreeMap::new(),
            providers: Names::new(),
        };
        for module in seseragi_project::standard_module_interfaces() {
            for export in module.exports.into_iter().filter(|e| e.namespace == "type") {
                types
                    .arities
                    .insert(export.symbol.clone(), export.scheme.type_parameters.len());
                types
                    .providers
                    .insert((module.module.clone(), export.name), export.symbol);
            }
        }
        types
    })
}

fn error(module: &CoreModule, spelling: &str, message: &str) -> RustBackendError {
    let origin = module
        .module_dependencies
        .iter()
        .flat_map(|dependency| &dependency.imports)
        .find(|import| import.namespace == "type" && import.local == spelling)
        .map(|import| &import.origin)
        .or_else(|| module.module_dependencies.first().map(|d| &d.origin))
        .or_else(|| module.functions.first().map(|f| &f.origin))
        .or_else(|| module.bindings.first().map(|b| &b.origin))
        .or_else(|| module.structs.first().map(|s| &s.origin))
        .or_else(|| module.adts.first().map(|a| &a.origin))
        .or_else(|| module.aliases.first().map(|a| &a.origin));
    match origin {
        Some(origin) => at(message, origin, &module.module),
        None => {
            let mut error = RustBackendError::from(message);
            error.module = Some(module.module.clone());
            error
        }
    }
}

/// A declaration/import gives a spelling lexical meaning. An imported value's
/// scheme also carries provider types, whose spellings are not new imports.
/// Keep both canonical identities without letting scheme metadata rebind a
/// checked source alias. ExternalNamed remains authoritative at each use site.
pub(crate) fn for_modules(modules: &[&CoreModule]) -> Result<Names, RustBackendError> {
    collect(modules, true)
}

pub(crate) fn for_boundary(module: &CoreModule) -> Result<Names, RustBackendError> {
    // Boundary codecs inspect one module before the closed graph is linked.
    // Its checked import edges still prove lexical aliases and provider names;
    // the project linker additionally checks source owners and declared kinds.
    collect(&[module], false)
}

fn collect(modules: &[&CoreModule], closed_graph: bool) -> Result<Names, RustBackendError> {
    let mut declared = BTreeMap::new();
    let mut names = Names::new();
    for module in modules {
        for (name, symbol, arity, origin) in declarations(module) {
            if symbol.rsplit_once("::").map(|(owner, _)| owner) != Some(module.module.as_str())
                || declared.insert(symbol.clone(), arity).is_some()
                || names
                    .insert((module.module.clone(), name.clone()), symbol.clone())
                    .is_some()
            {
                return Err(at(
                    "Rust backend: invalid or duplicate canonical type declaration",
                    origin,
                    &module.module,
                ));
            }
        }
    }
    for module in modules {
        let mut lexical = BTreeMap::new();
        for (name, symbol, _, _) in declarations(module) {
            lexical.insert(name.clone(), symbol.clone());
        }
        for dependency in &module.module_dependencies {
            for import in dependency.imports.iter().filter(|i| i.namespace == "type") {
                if lexical
                    .insert(import.local.clone(), import.canonical.clone())
                    .is_some_and(|old| old != import.canonical)
                {
                    return Err(at(
                        "Rust backend: ambiguous canonical type import",
                        &import.origin,
                        &module.module,
                    ));
                }
                // Bind the source alias through its checked import edge, not
                // through whichever same-spelling scheme binding comes first.
                let proven = module.external_type_bindings.iter().any(|binding| {
                    binding.spelling == import.local
                        && binding.canonical == import.canonical
                        && binding.provider.as_ref().is_some_and(|provider| {
                            provider.module == dependency.module
                                && provider.export == import.imported
                        })
                });
                if !proven {
                    return Err(at(
                        "Rust backend: unproven canonical type import provider",
                        &import.origin,
                        &module.module,
                    ));
                }
            }
        }
        let mut arities = BTreeMap::new();
        for binding in &module.external_type_bindings {
            let Some((owner, export)) = binding.canonical.rsplit_once("::") else {
                return Err(error(
                    module,
                    &binding.spelling,
                    "Rust backend: invalid canonical type binding owner",
                ));
            };
            let known_arity = declared
                .get(&binding.canonical)
                .copied()
                .or_else(|| {
                    binding
                        .canonical
                        .strip_prefix("std/prelude::")
                        .and_then(seseragi_semantics::prelude_type_constructor_arity)
                        .map(|arity| arity as usize)
                })
                .or_else(|| standard_types().arities.get(&binding.canonical).copied());
            if (closed_graph || owner.starts_with("std/")) && known_arity.is_none() {
                return Err(error(
                    module,
                    &binding.spelling,
                    "Rust backend: unproven canonical type binding owner",
                ));
            }
            if owner.is_empty() || export.is_empty() || binding.spelling.is_empty() {
                return Err(error(
                    module,
                    &binding.spelling,
                    "Rust backend: invalid canonical type binding owner",
                ));
            }
            if let Some(arity) = binding.arity {
                if known_arity.is_some_and(|expected| expected != arity as usize)
                    || arities
                        .insert(binding.canonical.clone(), arity)
                        .is_some_and(|previous| previous != arity)
                {
                    return Err(error(
                        module,
                        &binding.spelling,
                        "Rust backend: conflicting canonical type binding kind",
                    ));
                }
            }
            if let Some(provider) = &binding.provider {
                let direct = provider.module == owner && provider.export == export;
                let standard = standard_types()
                    .providers
                    .get(&(provider.module.clone(), provider.export.clone()))
                    == Some(&binding.canonical);
                let reexported = modules.iter().any(|source| {
                    source.module == provider.module
                        && source
                            .module_dependencies
                            .iter()
                            .flat_map(|d| &d.imports)
                            .any(|import| {
                                import.namespace == "type"
                                    && import.reexported_as.as_ref() == Some(&provider.export)
                                    && import.canonical == binding.canonical
                            })
                });
                let checked_import = module.module_dependencies.iter().any(|dependency| {
                    dependency.module == provider.module
                        && dependency.imports.iter().any(|import| {
                            import.namespace == "type"
                                && import.imported == provider.export
                                && import.canonical == binding.canonical
                        })
                });
                // A closed graph must prove the actual defining/reexporting
                // provider. Matching two forged metadata records is not proof.
                // A single-module codec check only has its checked import edge;
                // the project link subsequently verifies the complete graph.
                if !direct && !standard && !reexported && (closed_graph || !checked_import) {
                    return Err(error(
                        module,
                        &binding.spelling,
                        "Rust backend: conflicting canonical type provider owner",
                    ));
                }
            }
            if lexical.contains_key(&binding.spelling) {
                continue;
            }
            let key = (module.module.clone(), binding.spelling.clone());
            if names
                .insert(key, binding.canonical.clone())
                .is_some_and(|old| old != binding.canonical)
            {
                return Err(error(
                    module,
                    &binding.spelling,
                    "Rust backend: ambiguous canonical type binding",
                ));
            }
        }
        for (spelling, canonical) in lexical {
            names.insert((module.module.clone(), spelling), canonical);
        }
    }
    Ok(names)
}

fn declarations(
    module: &CoreModule,
) -> impl Iterator<Item = (&String, &String, usize, &SourceSpan)> {
    module
        .adts
        .iter()
        .map(|d| (&d.name, &d.symbol, d.type_parameters.len(), &d.origin))
        .chain(
            module
                .structs
                .iter()
                .map(|d| (&d.name, &d.symbol, d.type_parameters.len(), &d.origin)),
        )
        .chain(
            module
                .aliases
                .iter()
                .map(|d| (&d.name, &d.symbol, d.type_parameters.len(), &d.origin)),
        )
}
