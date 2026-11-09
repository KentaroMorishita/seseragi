use crate::features::DocumentState;
use seseragi_backend_rust::{
    diagnose_rust_project, emit_program_wasm, RustBackendError, RustDiagnosticTarget,
    RustTargetOptions,
};
use seseragi_driver::{compile_core_project, ProjectModuleInput};
use seseragi_project::{
    logical_module_id, LoadedLocalProject, ModuleGraph, ModuleIdentity, ModuleRoot,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone)]
pub(crate) struct TargetDiagnostic {
    pub(crate) target: &'static str,
    pub(crate) error: RustBackendError,
}

pub(crate) fn append(
    project: &LoadedLocalProject,
    graph: &ModuleGraph<String>,
    inputs: &[ProjectModuleInput],
    documents: &mut BTreeMap<String, DocumentState>,
) {
    let root = project.packages().root();
    let manifest = project.packages().package(root).unwrap().manifest();
    let mut targets = Vec::new();
    if manifest
        .run
        .as_ref()
        .and_then(|run| run.target.as_ref())
        .is_some_and(|target| target.as_str() == "native")
    {
        targets.push((
            logical_module_id(project.entry()),
            RustDiagnosticTarget::Native,
            Vec::new(),
        ));
    }
    // A wasm-target package builds `run.entry` with `run.exports`, as a
    // `web.programs` entry builds its module with its exports.
    let run_program = manifest
        .run
        .as_ref()
        .filter(|run| {
            run.target
                .as_ref()
                .is_some_and(|target| target.as_str() == "wasm")
        })
        .map(|run| (&run.entry, &run.exports));
    let web_programs = manifest
        .web
        .as_ref()
        .into_iter()
        .flat_map(|web| web.programs.values())
        .map(|program| (&program.module, &program.exports));
    for (module, selected) in run_program.into_iter().chain(web_programs) {
        let identity = ModuleIdentity::new(root.clone(), ModuleRoot::Source, module.clone());
        if project.module(&identity).is_some() {
            // An export at type arguments (`label<Int>`) is checked by the
            // build, which adds its binding before type checking.
            let exports = selected
                .iter()
                .filter_map(|text| seseragi_project::ProgramExport::parse(text).ok())
                .filter(|export| export.type_arguments.is_none())
                .map(|export| export.name)
                .collect();
            targets.push((
                logical_module_id(&identity),
                RustDiagnosticTarget::ProgramWasm,
                exports,
            ));
        }
    }
    for (entry, target, exports) in targets {
        let Some((graph, inputs)) = reachable_inputs(graph, inputs, &entry) else {
            continue;
        };
        let Ok(core) = compile_core_project(graph, inputs) else {
            continue;
        };
        let modules = core
            .order
            .iter()
            .map(|id| &core.modules[id].core_ir)
            .collect::<Vec<_>>();
        let mut errors =
            diagnose_rust_project(&modules, &entry, target, &RustTargetOptions::default());
        if target == RustDiagnosticTarget::ProgramWasm && !exports.is_empty() {
            if let Err(error) = emit_program_wasm(&modules, &entry, &exports) {
                if !errors.contains(&error) {
                    errors.push(error);
                }
            }
        }
        for error in errors {
            let Some(origin) = &error.origin else {
                continue;
            };
            let Some(document) = documents.values_mut().find(|document| match &error.module {
                Some(module) => document.analysis.module == *module,
                None => document.analysis.diagnostics.source == origin.source,
            }) else {
                continue;
            };
            let target = match target {
                RustDiagnosticTarget::Native => "native",
                RustDiagnosticTarget::ProgramWasm => "WASM",
            };
            if !document
                .target_diagnostics
                .iter()
                .any(|existing| existing.target == target && existing.error == error)
            {
                document
                    .target_diagnostics
                    .push(TargetDiagnostic { target, error });
            }
        }
    }
}

fn reachable_inputs(
    graph: &ModuleGraph<String>,
    inputs: &[ProjectModuleInput],
    entry: &str,
) -> Option<(ModuleGraph<String>, Vec<ProjectModuleInput>)> {
    let mut reachable = BTreeSet::new();
    let mut pending = vec![entry.to_owned()];
    while let Some(module) = pending.pop() {
        if reachable.insert(module.clone()) {
            pending.extend(
                graph
                    .dependencies_for(&module)?
                    .into_iter()
                    .map(|(_, id)| id),
            );
        }
    }
    let mut selected = ModuleGraph::new();
    for id in &reachable {
        selected
            .add_module(id.clone(), graph.dependencies_for(id)?)
            .ok()?;
    }
    let inputs = inputs
        .iter()
        .filter(|input| reachable.contains(&input.module_id))
        .cloned()
        .collect();
    Some((selected, inputs))
}
