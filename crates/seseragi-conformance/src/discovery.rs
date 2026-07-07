use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn discover_cases(directory: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut cases = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    cases.sort();
    cases
}

pub(crate) fn discover_interface_cases(schema_directory: &Path) -> Vec<PathBuf> {
    let mut cases = discover_cases(schema_directory)
        .into_iter()
        .filter(|case| case.join("interface.json").is_file())
        .filter(|case| diagnostics_are_empty(case))
        .collect::<Vec<_>>();
    cases.sort();
    cases
}

pub(crate) fn discover_resolved_ast_cases(schema_directory: &Path) -> Vec<PathBuf> {
    let mut cases = discover_cases(schema_directory)
        .into_iter()
        .filter(|case| case.join("resolved-ast.json").is_file())
        .filter(|case| diagnostics_are_empty(case))
        .collect::<Vec<_>>();
    cases.sort();
    cases
}

pub(crate) fn discover_artifact_cases(root: &Path, artifact_name: &str) -> Vec<PathBuf> {
    let mut cases = Vec::new();
    collect_artifact_cases(root, artifact_name, &mut cases);
    cases.sort();
    cases
}

fn collect_artifact_cases(directory: &Path, artifact_name: &str, cases: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    let mut entries = entries.filter_map(Result::ok).collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            if path.join(artifact_name).is_file() {
                cases.push(path.clone());
            }
            collect_artifact_cases(&path, artifact_name, cases);
        }
    }
}

fn diagnostics_are_empty(case: &Path) -> bool {
    let diagnostics_path = case.join("diagnostics.json");
    let Ok(raw) = fs::read_to_string(diagnostics_path) else {
        return true;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return false;
    };
    value
        .get("diagnostics")
        .and_then(|diagnostics| diagnostics.as_array())
        .is_some_and(|diagnostics| diagnostics.is_empty())
}
