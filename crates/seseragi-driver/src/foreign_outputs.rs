//! Pure adapter-selected runtime output locators. Physical discovery remains
//! in the host adapter; re-emission preserves the compiler's source provenance.
use crate::CompiledModule;
use std::collections::BTreeMap;

pub fn relocate_foreign_outputs(
    compiled: &CompiledModule,
    specifiers: &BTreeMap<String, String>,
) -> Result<CompiledModule, seseragi_syntax::DiagnosticArtifact> {
    if specifiers.is_empty() {
        return Ok(compiled.clone());
    }
    let mut relocated = compiled.clone();
    for module in &mut relocated.typescript_ir.foreign_modules {
        if let Some(specifier) = specifiers.get(&module.specifier) {
            module.specifier = specifier.clone();
        }
    }
    let source = compiled
        .generated
        .source_map
        .sources_content
        .first()
        .map(String::as_str)
        .unwrap_or_default();
    let outputs = &compiled.generated.metadata.outputs;
    let mut generated = seseragi_lowering::emit_typescript_module_with_output_paths(
        relocated.typescript_ir.clone(),
        source,
        seseragi_lowering::GeneratedOutputPaths::new(&outputs.typescript, &outputs.source_map),
    )
    .map_err(|error| {
        crate::lowering_panic::internal_error_diagnostics(compiled.diagnostics.clone(), &error)
    })?;
    generated.metadata = compiled.generated.metadata.clone();
    relocated.generated = generated;
    Ok(relocated)
}

#[cfg(test)]
mod tests {
    #[test]
    fn failed_foreign_reemission_returns_located_internal_diagnostics() {
        let source = "type Box deriving Show = | Box Int\n";
        let mut compiled =
            crate::compile_module(crate::CompileInput::new("box.ssrg", "app/box", source)).unwrap();
        let origin = compiled.typescript_ir.instances[0].origin.clone();
        compiled
            .typescript_ir
            .type_imports
            .retain(|import| import.feature != "core.show.dictionary");
        let diagnostics = super::relocate_foreign_outputs(
            &compiled,
            &[("./host".into(), "./relocated".into())].into(),
        )
        .unwrap_err();
        let diagnostic = &diagnostics.diagnostics[0];
        assert_eq!(diagnostic.code, "SES-I0001");
        assert_eq!(
            (diagnostic.primary.start, diagnostic.primary.end),
            (origin.start, origin.end)
        );
        assert!(diagnostic.related[0]
            .message
            .contains("during TypeScript emission"));
    }
}
