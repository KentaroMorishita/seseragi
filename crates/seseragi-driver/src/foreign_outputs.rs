//! Pure adapter-selected runtime output locators. Physical discovery remains
//! in the host adapter; re-emission preserves the compiler's source provenance.
use crate::CompiledModule;
use std::collections::BTreeMap;

pub fn relocate_foreign_outputs(
    compiled: &CompiledModule,
    specifiers: &BTreeMap<String, String>,
) -> CompiledModule {
    if specifiers.is_empty() {
        return compiled.clone();
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
    );
    generated.metadata = compiled.generated.metadata.clone();
    relocated.generated = generated;
    relocated
}
