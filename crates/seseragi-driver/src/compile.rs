use crate::{CompileInput, CompiledModule};
use seseragi_lowering::{
    emit_typescript_module, emit_typescript_module_with_output_paths,
    lower_core_module_to_typescript_ir, lower_core_module_to_typescript_ir_with_plan,
    lower_typed_module, GeneratedOutputPaths, TypeScriptLoweringError, TypeScriptOutputPlan,
};
use seseragi_semantics::{analyze_linked_module, analyze_module_interface};
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LinkedCompileError {
    Diagnostics(DiagnosticArtifact),
    TypeScriptPlan(TypeScriptLoweringError),
}

/// Compiles a module after the project layer has fixed its dependency graph,
/// public dependency interfaces, and generated TypeScript output specifiers.
pub fn compile_linked_module(
    linked: seseragi_project::LinkedModule,
    source: &str,
    output_plan: &TypeScriptOutputPlan,
) -> Result<CompiledModule, LinkedCompileError> {
    compile_linked_module_with_output_paths(
        linked,
        source,
        output_plan,
        GeneratedOutputPaths::default(),
    )
}

/// Like [`compile_linked_module`], while preserving project-selected generated
/// artifact paths in metadata and source maps.
pub fn compile_linked_module_with_output_paths(
    linked: seseragi_project::LinkedModule,
    source: &str,
    output_plan: &TypeScriptOutputPlan,
    output_paths: GeneratedOutputPaths,
) -> Result<CompiledModule, LinkedCompileError> {
    let diagnostics = parse_diagnostics(linked.interface.source.clone(), source);
    let analyzed = analyze_linked_module(diagnostics, linked, source)
        .map_err(LinkedCompileError::Diagnostics)?;
    let core_ir = lower_typed_module(analyzed.typed_hir.clone());
    let typescript_ir = lower_core_module_to_typescript_ir_with_plan(core_ir.clone(), output_plan)
        .map_err(LinkedCompileError::TypeScriptPlan)?;
    let generated =
        emit_typescript_module_with_output_paths(typescript_ir.clone(), source, output_paths);

    Ok(CompiledModule {
        diagnostics: analyzed.diagnostics,
        typed_hir: analyzed.typed_hir,
        typed_interface: analyzed.typed_interface,
        core_ir,
        typescript_ir,
        generated,
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stops_unknown_custom_operators_before_lowering() {
        let source = "fn invalid value: Int -> Int = value <^> 1\n";
        let diagnostics = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/custom-operator-unknown",
            source,
        ))
        .expect_err("unknown custom operator must reject compilation");

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-P0101");
        assert_eq!(diagnostics.diagnostics[0].message_key, "operator.unknown");
    }

    #[test]
    fn stops_non_referenceable_operator_sections_before_lowering() {
        let source = "pub let invalid = (&&)\n";
        let diagnostics = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/operator-section-forbidden",
            source,
        ))
        .expect_err("non-referenceable operator sections must reject compilation");

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-P0001");
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "parser.expected-expression"
        );
        assert_eq!(diagnostics.diagnostics[0].primary.start, 19);
        assert_eq!(diagnostics.diagnostics[0].primary.end, 21);
    }

    #[test]
    fn lowers_local_custom_infix_calls_without_raw_typescript_operators() {
        let source = "operator infixr 4 <.> left: Int -> right: Int -> Int = left - right\n\
                      pub fn calculate unit: Unit -> Int = 10 <.> 3 <.> 2\n";
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/custom-operator",
            source,
        ))
        .expect("valid custom operator should compile");

        assert!(compiled
            .generated
            .typescript
            .contains("__ssrg$operator$3c2e3e"));
        assert!(!compiled.generated.typescript.contains(" <.> "));
    }

    #[test]
    fn stops_non_binary_custom_operator_declarations_before_lowering() {
        let source = "operator infixl 4 <^> value: Int -> Int = value\n";
        let diagnostics = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/custom-operator-invalid-arity",
            source,
        ))
        .expect_err("non-binary custom operator must reject compilation");

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-P0001");
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "operator.invalid-arity"
        );
    }
}
