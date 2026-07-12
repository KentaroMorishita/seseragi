use crate::{CompileInput, CompiledModule};
use seseragi_lowering::{
    emit_typescript_module, lower_core_module_to_typescript_ir, lower_typed_module,
};
use seseragi_semantics::analyze_module_interface;
use seseragi_syntax::{
    parse_diagnostics, parse_import_free_module_interface, DiagnosticArtifact, DiagnosticSeverity,
};

/// Compiles one source using an explicit logical module identity. This is a
/// pure, currently unlinked single-module pipeline; imports require a future
/// project/module-graph driver and are rejected before typing and lowering.
pub fn compile_module(input: CompileInput<'_>) -> Result<CompiledModule, DiagnosticArtifact> {
    let mut diagnostics = parse_diagnostics(input.source_name(), input.source());
    if has_errors(&diagnostics) {
        return Err(diagnostics);
    }

    let interface = match parse_import_free_module_interface(
        input.source_name(),
        input.module_id(),
        input.source(),
    ) {
        Ok(interface) => interface,
        Err(imports) => {
            crate::dependencies::append_unlinked_dependency_diagnostics(imports, &mut diagnostics);
            return Err(diagnostics);
        }
    };
    let analyzed = analyze_module_interface(diagnostics, interface, input.source())?;
    Ok(finish_compilation(
        analyzed.diagnostics,
        analyzed.typed_hir,
        analyzed.typed_interface,
        input.source(),
    ))
}

fn has_errors(diagnostics: &DiagnosticArtifact) -> bool {
    diagnostics
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
}

fn finish_compilation(
    diagnostics: DiagnosticArtifact,
    typed_hir: seseragi_semantics::TypedModule,
    typed_interface: seseragi_semantics::TypedModuleInterface,
    source: &str,
) -> CompiledModule {
    let core_ir = lower_typed_module(typed_hir.clone());
    let typescript_ir = lower_core_module_to_typescript_ir(core_ir.clone());
    let generated = emit_typescript_module(typescript_ir.clone(), source);

    CompiledModule {
        diagnostics,
        typed_hir,
        typed_interface,
        core_ir,
        typescript_ir,
        generated,
    }
}
