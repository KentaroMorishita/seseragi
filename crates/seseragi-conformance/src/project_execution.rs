#[path = "project_execution/check.rs"]
mod check;
#[path = "project_execution/model.rs"]
mod model;
#[cfg(test)]
#[path = "project_execution/tests.rs"]
mod tests;

pub(crate) use check::check_project_execution_case;
pub(crate) use model::has_project_execution_layout;
