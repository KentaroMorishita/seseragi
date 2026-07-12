//! Pure compiler orchestration for one already-identified Seseragi module.
//!
//! Filesystem discovery, package identity, and import linking belong to the
//! project layer. This crate owns the ordered frontend-to-backend pipeline and
//! never derives logical identity from a physical path.

mod compile;
mod dependencies;
mod input;
mod output;
mod output_plan;
mod project_compile;

pub use compile::{
    compile_linked_module, compile_linked_module_with_output_paths, compile_module,
    LinkedCompileError,
};
pub use input::CompileInput;
pub use output::CompiledModule;
pub use output_plan::{
    generated_output_paths, plan_typescript_outputs, TypeScriptInstanceOutput,
    TypeScriptModuleOutput, TypeScriptOutputPlanError,
};
pub use project_compile::{
    compile_project, CompiledProject, ProjectCompileError, ProjectModuleInput,
};
