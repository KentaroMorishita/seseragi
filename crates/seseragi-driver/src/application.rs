use crate::CompiledModule;
use seseragi_lowering::{emit_typescript_module_with_output_paths, GeneratedOutputPaths};
use std::collections::BTreeMap;

pub use seseragi_lowering::ApplicationReachability;

/// Finalize an application output graph with explicitly selected product roots.
/// Analysis/HIR stays intact for diagnostics and entry contract consumers.
pub fn retain_application_outputs(
    modules: &mut BTreeMap<String, CompiledModule>,
    roots: &[(String, String)],
) -> Result<ApplicationReachability, seseragi_syntax::DiagnosticArtifact> {
    let mut outputs = modules
        .iter()
        .map(|(id, module)| (id.clone(), module.typescript_ir.clone()))
        .collect();
    let report = seseragi_lowering::retain_application(&mut outputs, roots);
    // Build the complete replacement before mutating caller-owned outputs.
    // An emission failure must not publish a partially retained application.
    let mut retained = BTreeMap::new();
    for (id, output) in outputs {
        let Some(compiled) = modules.get(&id) else {
            continue;
        };
        let mut compiled = compiled.clone();
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
        )
        .map_err(|error| {
            crate::lowering_panic::internal_error_diagnostics(compiled.diagnostics.clone(), &error)
        })?;
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
        retained.insert(id, compiled);
    }
    *modules = retained;
    Ok(report)
}

#[cfg(test)]
mod tests {
    #[test]
    fn failed_reemission_preserves_all_application_outputs() {
        let source =
            "type Box deriving Show = | Box Int\npub fn label value: Box -> String = show value\n";
        let mut compiled =
            crate::compile_module(crate::CompileInput::new("box.ssrg", "app/box", source)).unwrap();
        compiled
            .typescript_ir
            .type_imports
            .retain(|import| import.feature != "core.show.dictionary");
        let mut modules = std::collections::BTreeMap::from([("app/box".into(), compiled)]);
        let expected = modules.clone();
        let diagnostics =
            super::retain_application_outputs(&mut modules, &[("app/box".into(), "label".into())])
                .unwrap_err();
        assert_eq!(modules, expected);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-I0001");
        assert!(diagnostics.diagnostics[0].related[0]
            .message
            .contains("during TypeScript emission"));
    }
}
