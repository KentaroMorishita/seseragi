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

pub use compile::{compile_linked_module, compile_module, LinkedCompileError};
pub use input::CompileInput;
pub use output::CompiledModule;
pub use output_plan::{plan_typescript_outputs, TypeScriptModuleOutput, TypeScriptOutputPlanError};
