use seseragi_lowering::{CoreModule, GeneratedBundle, TypeScriptModule};
use seseragi_semantics::{TypedModule, TypedModuleInterface};
use seseragi_syntax::DiagnosticArtifact;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledModule {
    /// Non-error diagnostics are retained for driver adapters to report.
    pub diagnostics: DiagnosticArtifact,
    pub typed_hir: TypedModule,
    pub typed_interface: TypedModuleInterface,
    pub core_ir: CoreModule,
    pub typescript_ir: TypeScriptModule,
    pub generated: GeneratedBundle,
}
