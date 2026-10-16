use crate::{emit, RustArtifact, RustBackendError};
use seseragi_lowering::{core_module_initialization_order, CoreModule, SourceSpan};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn at(message: &str, origin: &SourceSpan, module: &str) -> RustBackendError {
    let mut error = RustBackendError::from(message);
    error.origin = Some(origin.clone());
    error.module = Some(module.to_owned());
    error
}

pub(crate) fn emit(
    modules: &[&CoreModule],
    entry_module: &str,
    entry: &str,
) -> Result<RustArtifact, RustBackendError> {
    let entry_core = modules
        .iter()
        .find(|m| m.module == entry_module)
        .ok_or("Rust backend: entry module not found")?;
    let mut linked = (*entry_core).clone();
    linked.adts.clear();
    linked.aliases.clear();
    linked.structs.clear();
    linked.instances.clear();
    linked.functions.clear();
    linked.bindings.clear();
    linked.module_dependencies.clear();
    let mut seen = BTreeSet::new();
    let mut owners = BTreeMap::new();
    let mut symbols = BTreeSet::new();
    let mut type_names = BTreeMap::new();
    for module in modules {
        if module.module.starts_with("std/") || !seen.insert(module.module.clone()) {
            return Err("Rust backend: duplicate or reserved source module identity".into());
        }
        for binding in &module.external_type_bindings {
            let key = (module.module.clone(), binding.spelling.clone());
            if type_names
                .insert(key, binding.canonical.clone())
                .is_some_and(|old| old != binding.canonical)
            {
                return Err("Rust backend: ambiguous canonical type binding".into());
            }
        }
        for dependency in &module.module_dependencies {
            if !dependency.module.starts_with("std/")
                && (dependency.module == module.module || !seen.contains(&dependency.module))
            {
                return Err(at(
                    "Rust backend: source dependency is missing or out of order",
                    &dependency.origin,
                    &module.module,
                ));
            }
        }
        if let Some(foreign) = module.foreign_modules.first() {
            return Err(at(
                "Rust backend: foreign modules are unsupported",
                &foreign.origin,
                &module.module,
            ));
        }
        for (symbol, origin) in module
            .functions
            .iter()
            .map(|f| (&f.symbol, &f.origin))
            .chain(module.bindings.iter().map(|b| (&b.symbol, &b.origin)))
            .chain(
                module
                    .adts
                    .iter()
                    .flat_map(|a| a.variants.iter().map(|v| (&v.symbol, &v.origin))),
            )
        {
            if symbol.rsplit_once("::").map(|(owner, _)| owner) != Some(module.module.as_str())
                || !symbols.insert(symbol.clone())
            {
                return Err(at(
                    "Rust backend: invalid or duplicate canonical symbol owner",
                    origin,
                    &module.module,
                ));
            }
            owners.insert(symbol.clone(), module.module.clone());
        }
        // Instance identities follow semantic trait/type identity; a standard
        // trait's user implementation need not start with its source module.
        // Keep the defining owner from the checked graph rather than parsing it.
        for instance in &module.instances {
            if !symbols.insert(instance.identity.clone()) {
                return Err(at(
                    "Rust backend: duplicate canonical instance identity",
                    &instance.origin,
                    &module.module,
                ));
            }
            owners.insert(instance.identity.clone(), module.module.clone());
        }
        linked.adts.extend(module.adts.clone());
        linked.aliases.extend(module.aliases.clone());
        linked.structs.extend(module.structs.clone());
        linked.instances.extend(module.instances.clone());
        linked.functions.extend(module.functions.clone());
    }
    let initialization = core_module_initialization_order(modules, entry_module)
        .ok_or("Rust backend: invalid source initialization graph")?;
    if initialization.len() != modules.len() {
        return Err("Rust backend: source module is outside the entry dependency graph".into());
    }
    for identity in &initialization {
        let module = modules
            .iter()
            .find(|m| &m.module == identity)
            .expect("checked initialization owner");
        linked.bindings.extend(module.bindings.clone());
    }
    let scalar_globals =
        seseragi_lowering::CoreScalarGlobals::from_modules(modules.iter().copied());
    let mut artifact = emit::emit(&linked, entry, &owners, &scalar_globals, &type_names)?;
    artifact.metadata.modules = initialization;
    Ok(artifact)
}
