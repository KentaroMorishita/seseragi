#[path = "project_compile/check.rs"]
mod check;
#[path = "project_compile/compile.rs"]
mod compile;
#[path = "project_compile/model.rs"]
mod model;
#[cfg(test)]
#[path = "project_compile/tests.rs"]
mod tests;

pub(crate) use check::check_project_compile_case;
