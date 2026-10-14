use crate::{analyze::analyze_module_frontend, CompileInput};
use seseragi_lowering::{lower_typed_module, CoreModule};
use seseragi_semantics::{AnalyzedModule, TypedModule, TypedModuleInterface};
use seseragi_syntax::DiagnosticArtifact;

/// Checked, backend-independent input. No target lowering or emission runs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreCompilation {
    pub diagnostics: DiagnosticArtifact,
    pub typed_hir: TypedModule,
    pub typed_interface: TypedModuleInterface,
    pub core_ir: CoreModule,
}

pub fn compile_core_module(input: CompileInput<'_>) -> Result<CoreCompilation, DiagnosticArtifact> {
    analyze_module_frontend(input).map(lower_analyzed_module)
}

/// Project adapters may supply a checked module after resolving imports.
pub fn lower_analyzed_module(analyzed: AnalyzedModule) -> CoreCompilation {
    CoreCompilation {
        core_ir: lower_typed_module(analyzed.typed_hir.clone()),
        diagnostics: analyzed.diagnostics,
        typed_hir: analyzed.typed_hir,
        typed_interface: analyzed.typed_interface,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_backends_receive_identical_checked_core() {
        let input = CompileInput::new(
            "example.ssrg",
            "logical/example",
            "pub fn answer -> Int = 42\n",
        );
        let core = compile_core_module(input).unwrap();
        let ts = crate::compile_module(input).unwrap();
        assert_eq!(core.core_ir, ts.core_ir);
        assert_eq!(core.typed_interface, ts.typed_interface);
        assert!(compile_core_module(CompileInput::new(
            "bad.ssrg",
            "logical/bad",
            "pub fn answer -> Int = \"bad\"\n"
        ))
        .is_err());
    }
}
