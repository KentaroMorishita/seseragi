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
    emit_body(modules, entry_module, Some(entry), options, None)
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
        None,
    )
}

pub(crate) fn emit_tests(
    modules: &[&CoreModule],
    tests: &[(String, String)],
) -> Result<RustArtifact, RustBackendError> {
    let root = CoreModule {
        schema: 1,
        stage: "core-ir".into(),
        module: "\0native-tests".into(),
        foreign_modules: vec![],
        external_type_bindings: vec![],
        module_dependencies: modules
            .iter()
            .map(|module| seseragi_lowering::CoreModuleDependency {
                specifier: module.module.clone(),
                module: module.module.clone(),
                imports: vec![],
                origin: SourceSpan {
                    source: module.module.clone(),
                    start: 0,
                    end: 0,
                },
            })
            .collect(),
        adts: vec![],
        aliases: vec![],
        structs: vec![],
        instances: vec![],
        trait_methods: vec![],
        bindings: vec![],
        functions: vec![],
    };
    let mut graph = modules.to_vec();
    graph.push(&root);
    let mut artifact = emit_body(
        &graph,
        &root.module,
        None,
        &crate::RustTargetOptions::default(),
        Some(tests),
    )?;
    artifact
        .metadata
        .modules
        .retain(|module| module != &root.module);
    Ok(artifact)
}

fn emit_body(
    modules: &[&CoreModule],
    entry_module: &str,
    entry: Option<&str>,
    options: &crate::RustTargetOptions,
    tests: Option<&[(String, String)]>,
) -> Result<RustArtifact, RustBackendError> {
    let entry_core = modules
        .iter()
        .find(|m| m.module == entry_module)
        .ok_or("Rust backend: entry module not found")?;
    let entry = entry
        .map(|name| {
            let symbol = format!("{entry_module}::{name}");
            if entry_core
                .functions
                .iter()
                .any(|function| function.symbol == symbol)
            {
                return Ok(symbol);
            }
            let source = entry_core
                .functions
                .first()
                .map(|function| &function.origin.source)
                .or_else(|| {
                    entry_core
                        .bindings
                        .first()
                        .map(|binding| &binding.origin.source)
                })
                .or_else(|| {
                    entry_core
                        .module_dependencies
                        .first()
                        .map(|dependency| &dependency.origin.source)
                })
                .unwrap_or(&entry_core.module);
            Err(at(
                &format!("Rust backend: entry function `{symbol}` not found"),
                &SourceSpan {
                    source: source.clone(),
                    start: 0,
                    end: 0,
                },
                entry_module,
            ))
        })
        .transpose()?;
    let LinkedProject {
        linked,
        owners,
        scalar_globals,
        type_names,
        trait_names,
        initialization,
    } = link(modules, entry_module)?;
    let mut artifact = if let Some(tests) = tests {
        emit::emit_tests(
            &linked,
            tests,
            &owners,
            &scalar_globals,
            &type_names,
            &trait_names,
            options,
        )
    } else if let Some(entry) = &entry {
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

struct LinkedProject {
    linked: CoreModule,
    owners: BTreeMap<String, String>,
    scalar_globals: seseragi_lowering::CoreScalarGlobals,
    type_names: BTreeMap<(String, String), String>,
    trait_names: BTreeMap<(String, String), String>,
    initialization: Vec<String>,
}

fn link(modules: &[&CoreModule], entry_module: &str) -> Result<LinkedProject, RustBackendError> {
    let entry_core = modules
        .iter()
        .find(|module| module.module == entry_module)
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
    Ok(LinkedProject {
        linked,
        owners,
        scalar_globals,
        type_names,
        trait_names,
        initialization,
    })
}

pub(crate) fn diagnose(
    modules: &[&CoreModule],
    entry_module: &str,
    target: crate::RustDiagnosticTarget,
    options: &crate::RustTargetOptions,
) -> Vec<RustBackendError> {
    let mut diagnostics = Vec::new();
    let modules = modules
        .iter()
        .map(|module| {
            let mut module = (*module).clone();
            for foreign in module.foreign_modules.drain(..) {
                diagnostics.push(at(
                    "Rust backend: foreign modules are unsupported",
                    &foreign.origin,
                    &module.module,
                ));
            }
            module
        })
        .collect::<Vec<_>>();
    match link(&modules.iter().collect::<Vec<_>>(), entry_module) {
        Ok(project) => diagnostics.extend(emit::diagnose(
            &project.linked,
            &project.owners,
            &project.scalar_globals,
            &project.type_names,
            &project.trait_names,
            options,
            target,
        )),
        Err(error) => diagnostics.push(error),
    }
    diagnostics
}
