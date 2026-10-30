use crate::{emit, RustArtifact, RustBackendError};
use seseragi_lowering::{core_module_initialization_order, CoreModule, SourceSpan};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn at(message: &str, origin: &SourceSpan, module: &str) -> RustBackendError {
    let mut error = RustBackendError::from(message);
    error.origin = Some(origin.clone());
    error.module = Some(module.to_owned());
    error
}

fn function_owner<'a>(module: &'a CoreModule, symbol: &'a str) -> Option<&'a str> {
    let (owner, name) = symbol.rsplit_once("::")?;
    if name.is_empty() {
        return None;
    }
    // Inherent methods use Type::method identities. Resolve their defining
    // module through a declared nominal type, never through an arbitrary prefix.
    if module
        .structs
        .iter()
        .map(|s| &s.symbol)
        .chain(module.adts.iter().map(|a| &a.symbol))
        .any(|symbol| symbol == owner)
    {
        owner.rsplit_once("::").map(|(module, _)| module)
    } else {
        Some(owner)
    }
}

pub(crate) fn emit(
    modules: &[&CoreModule],
    entry_module: &str,
    entry: &str,
) -> Result<RustArtifact, RustBackendError> {
    emit_with_options(
        modules,
        entry_module,
        entry,
        &crate::RustTargetOptions::default(),
    )
}

pub(crate) fn emit_with_options(
    modules: &[&CoreModule],
    entry_module: &str,
    entry: &str,
    options: &crate::RustTargetOptions,
) -> Result<RustArtifact, RustBackendError> {
    emit_body(modules, entry_module, Some(entry), options)
}

pub(crate) fn emit_library(
    modules: &[&CoreModule],
    entry_module: &str,
) -> Result<RustArtifact, RustBackendError> {
    emit_body(
        modules,
        entry_module,
        None,
        &crate::RustTargetOptions::default(),
    )
}

fn emit_body(
    modules: &[&CoreModule],
    entry_module: &str,
    entry: Option<&str>,
    options: &crate::RustTargetOptions,
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
    linked.trait_methods.clear();
    linked.functions.clear();
    linked.bindings.clear();
    linked.module_dependencies.clear();
    let mut seen = BTreeSet::new();
    let mut owners = BTreeMap::new();
    let mut symbols = BTreeSet::new();
    let mut trait_names = BTreeMap::new();
    let mut method_names = BTreeSet::new();
    for module in modules {
        if module.module.starts_with("std/") || !seen.insert(module.module.clone()) {
            return Err("Rust backend: duplicate or reserved source module identity".into());
        }
        for dependency in &module.module_dependencies {
            for import in dependency
                .imports
                .iter()
                .filter(|item| item.namespace == "trait")
            {
                let key = (module.module.clone(), import.local.clone());
                if trait_names
                    .insert(key, import.canonical.clone())
                    .is_some_and(|previous| previous != import.canonical)
                {
                    return Err(at(
                        "Rust backend: ambiguous canonical trait alias",
                        &import.origin,
                        &module.module,
                    ));
                }
            }
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
        for (symbol, origin, owner) in module
            .functions
            .iter()
            .map(|f| (&f.symbol, &f.origin, function_owner(module, &f.symbol)))
            .chain(module.bindings.iter().map(|b| {
                (
                    &b.symbol,
                    &b.origin,
                    b.symbol.rsplit_once("::").map(|(owner, _)| owner),
                )
            }))
            .chain(module.adts.iter().flat_map(|a| {
                a.variants.iter().map(|v| {
                    (
                        &v.symbol,
                        &v.origin,
                        v.symbol.rsplit_once("::").map(|(owner, _)| owner),
                    )
                })
            }))
        {
            if owner != Some(module.module.as_str()) || !symbols.insert(symbol.clone()) {
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
        for method in &module.trait_methods {
            if method
                .trait_identity
                .rsplit_once("::")
                .map(|(owner, _)| owner)
                != Some(module.module.as_str())
                || method.method.is_empty()
                || !method_names.insert((method.trait_identity.clone(), method.method.clone()))
            {
                return Err(at(
                    "Rust backend: invalid or duplicate trait method declaration owner",
                    &method.origin,
                    &module.module,
                ));
            }
            linked.trait_methods.push(method.clone());
        }
        linked.functions.extend(module.functions.clone());
    }
    let type_names = crate::type_names::for_modules(modules)?;
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
    let mut artifact = if let Some(entry) = entry {
        emit::emit(
            &linked,
            entry,
            &owners,
            &scalar_globals,
            &type_names,
            &trait_names,
            options,
        )
    } else {
        emit::emit_library(&linked, &owners, &scalar_globals, &type_names, &trait_names)
    }?;
    artifact.metadata.modules = initialization;
    Ok(artifact)
}
