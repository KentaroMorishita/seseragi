#[path = "project_compile/check.rs"]
mod check;
#[path = "project_compile/compile.rs"]
mod compile;
#[path = "project_compile/model.rs"]
mod model;
#[path = "project_compile/stage.rs"]
mod stage;
#[cfg(test)]
#[path = "project_compile/tests.rs"]
mod tests;
#[path = "project_compile/typecheck.rs"]
mod typecheck;

pub(crate) use check::check_project_compile_case;
pub(crate) use compile::{compile_project_compile_case, CompiledProjectCompileCase};
pub(crate) use stage::stage_project_typescript;
