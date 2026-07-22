//! Pure compiler orchestration for one already-identified Seseragi module.
//!
//! Filesystem discovery, package identity, and import linking belong to the
//! project layer. This crate owns the ordered frontend-to-backend pipeline and
//! never derives logical identity from a physical path.

mod analyze;
mod compile;
mod dependencies;
mod format;
mod input;
mod local_package;
mod local_project;
mod output;
mod output_plan;
mod project_compile;
mod reporting;

pub use analyze::analyze_module;
pub use compile::{
    compile_linked_module, compile_linked_module_with_output_paths, compile_module,
    LinkedCompileError,
};
pub use format::format_module;
pub use input::CompileInput;
pub use local_package::{compile_local_package, CompiledLocalPackage, LocalPackageCompileError};
pub use local_project::{compile_local_project, CompiledLocalProject, LocalProjectCompileError};
pub use output::CompiledModule;
pub use output_plan::{
    generated_output_paths, plan_typescript_outputs, TypeScriptInstanceOutput,
    TypeScriptModuleOutput, TypeScriptOutputPlanError,
};
pub use project_compile::{
    compile_project, CompiledProject, ProjectCompileError, ProjectModuleInput,
};
pub use reporting::render_terminal_diagnostics;
pub use seseragi_formatter::FormattedSource;
pub use seseragi_semantics::{
    AnalysisCallable, AnalysisCallableOccurrence, AnalysisDocument, AnalysisParameter,
    AnalysisReferenceItem, AnalysisSymbol, AnalysisSymbolOccurrence, AnalysisTypeOccurrence,
};
