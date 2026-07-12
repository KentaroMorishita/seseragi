use super::{load_project_execution_case, ProjectExecutionCase};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LoadedProjectExecutionCase {
    pub(crate) id: String,
    pub(crate) directory: PathBuf,
    pub(crate) case: ProjectExecutionCase,
}

pub(crate) fn has_project_execution_layout(project_root: &Path) -> bool {
    project_root.join("execution.json").exists() || project_root.join("executions").exists()
}

/// Loads either the legacy root execution or a sorted set of nested project
/// executions. A project must choose exactly one layout.
pub(crate) fn load_project_execution_cases(
    project_root: &Path,
) -> Result<Vec<LoadedProjectExecutionCase>, String> {
    let root_descriptor = project_root.join("execution.json");
    let nested_root = project_root.join("executions");
    let has_root = root_descriptor.exists();
    let has_nested = nested_root.exists();
    if has_root && has_nested {
        return Err(
            "project execution layout must not mix execution.json with executions/".to_owned(),
        );
    }
    if has_root {
        if !root_descriptor.is_file() {
            return Err("project execution.json must be a file".to_owned());
        }
        return Ok(vec![LoadedProjectExecutionCase {
            id: "default".to_owned(),
            directory: project_root.to_path_buf(),
            case: load_project_execution_case(project_root)?,
        }]);
    }
    if !has_nested {
        return Err("project execution layout is missing".to_owned());
    }
    if !nested_root.is_dir() {
        return Err("project executions must be a directory".to_owned());
    }

    let mut entries = fs::read_dir(&nested_root)
        .map_err(|error| format!("failed to read project executions directory: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to read project execution entry: {error}"))?;
    entries.sort_by_key(|entry| entry.file_name());
    if entries.is_empty() {
        return Err("project executions directory must not be empty".to_owned());
    }

    entries
        .into_iter()
        .map(|entry| load_nested_case(entry.path()))
        .collect()
}

fn load_nested_case(directory: PathBuf) -> Result<LoadedProjectExecutionCase, String> {
    if !directory.is_dir() {
        return Err("project executions entries must be case directories".to_owned());
    }
    let id = directory
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .ok_or_else(|| "project execution case id must be valid UTF-8".to_owned())?
        .to_owned();
    if !directory.join("execution.json").is_file() {
        return Err(format!(
            "project execution case {id} is missing execution.json"
        ));
    }
    let case = load_project_execution_case(&directory)
        .map_err(|error| format!("project execution case {id}: {error}"))?;
    Ok(LoadedProjectExecutionCase {
        id,
        directory,
        case,
    })
}
