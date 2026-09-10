use crate::CompiledModule;
use seseragi_lowering::{emit_typescript_module_with_output_paths, GeneratedOutputPaths};
use std::collections::BTreeMap;

pub use seseragi_lowering::ApplicationReachability;

/// Finalize an application output graph with explicitly selected product roots.
/// Analysis/HIR stays intact for diagnostics and entry contract consumers.
pub fn retain_application_outputs(
    modules: &mut BTreeMap<String, CompiledModule>,
    roots: &[(String, String)],
) -> ApplicationReachability {
    let mut outputs = modules
        .iter()
        .map(|(id, module)| (id.clone(), module.typescript_ir.clone()))
        .collect();
    let report = seseragi_lowering::retain_application(&mut outputs, roots);
    modules.retain(|id, compiled| {
        let Some(output) = outputs.remove(id) else {
            return false;
        };
        let previous = &compiled.generated;
        let mut generated = emit_typescript_module_with_output_paths(
            output.clone(),
            previous
                .source_map
                .sources_content
                .first()
                .map(String::as_str)
                .unwrap_or_default(),
            GeneratedOutputPaths::new(
                &previous.metadata.outputs.typescript,
                &previous.metadata.outputs.source_map,
            ),
        );
        generated.metadata.profile = previous.metadata.profile.clone();
        let functions = output
            .functions
            .iter()
            .map(|function| {
                let seseragi_lowering::TypeScriptFunction::ConstFunction { name, .. } = function;
                name.as_str()
            })
            .collect::<std::collections::BTreeSet<_>>();
        generated.metadata.inspection_roots = previous
            .metadata
            .inspection_roots
            .iter()
            .filter(|(_, name)| functions.contains(name.as_str()))
            .map(|(symbol, name)| (symbol.clone(), name.clone()))
            .collect();
        generated.metadata.effect_roots = previous
            .metadata
            .effect_roots
            .iter()
            .filter(|symbol| generated.metadata.inspection_roots.contains_key(*symbol))
            .cloned()
            .collect();
        generated.metadata.newtype_constructors = previous
            .metadata
            .newtype_constructors
            .iter()
            .filter(|name| {
                output
                    .adts
                    .iter()
                    .any(|adt| adt.variants.iter().any(|variant| &variant.name == *name))
                    || output.source_imports.iter().any(|import| {
                        import
                            .bindings
                            .iter()
                            .any(|binding| &binding.local == *name)
                    })
            })
            .cloned()
            .collect();
        // Inspection metadata describes surviving code; it never creates roots.
        compiled.typescript_ir = output;
        compiled.generated = generated;
        true
    });
    report
}
