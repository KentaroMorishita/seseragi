use crate::execution::Invocation;
use crate::execution_case::parse_invocation_document;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectExecutionCase {
    pub(crate) entry_module: String,
    pub(crate) entry_export: String,
    pub(crate) invocation: Invocation,
    pub(crate) runtime_requirements: Vec<String>,
    pub(crate) exit_code: i64,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectExecutionDescriptor {
    schema: u32,
    kind: String,
    target: String,
    entry: ProjectExecutionEntry,
    invocation: serde_json::Value,
    expected: ProjectExecutionExpected,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectExecutionEntry {
    module: String,
    export: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectExecutionExpected {
    #[serde(rename = "runtimeRequirements")]
    runtime_requirements: Vec<String>,
    process: ProjectExecutionProcess,
    stdout: String,
    stderr: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectExecutionProcess {
    #[serde(rename = "exitCode")]
    exit_code: i64,
}

/// Reads a pure project execution envelope stored beside `project.json`.
pub(crate) fn load_project_execution_case(case: &Path) -> Result<ProjectExecutionCase, String> {
    let descriptor_path = case.join("execution.json");
    let raw = fs::read_to_string(&descriptor_path)
        .map_err(|error| format!("failed to read execution.json: {error}"))?;
    let descriptor: ProjectExecutionDescriptor = serde_json::from_str(&raw)
        .map_err(|error| format!("failed to parse execution.json: {error}"))?;
    if descriptor.schema != 1 {
        return Err("execution.json must use schema 1".to_owned());
    }
    if descriptor.kind != "pure-project-execution" {
        return Err("execution.json kind must be pure-project-execution".to_owned());
    }
    if descriptor.target != "node-process" {
        return Err("execution.json target must be node-process".to_owned());
    }
    if descriptor.entry.module.is_empty() {
        return Err("execution.json entry.module is missing".to_owned());
    }
    if descriptor.entry.export.is_empty() {
        return Err("execution.json entry.export is missing".to_owned());
    }
    let invocation = parse_invocation_document(
        "execution.json",
        &serde_json::json!({ "invocation": descriptor.invocation }),
    )?;
    if !matches!(invocation, Invocation::PureJson { .. }) {
        return Err("execution.json only supports pure invocation".to_owned());
    }
    validate_fixture_path("expected.stdout", &descriptor.expected.stdout)?;
    validate_fixture_path("expected.stderr", &descriptor.expected.stderr)?;
    if descriptor.expected.stdout == descriptor.expected.stderr {
        return Err("execution.json expected stdout and stderr must be distinct".to_owned());
    }
    if descriptor.expected.process.exit_code != 0 {
        return Err("pure project execution must expect process exitCode 0".to_owned());
    }
    let mut requirements = BTreeSet::new();
    for requirement in &descriptor.expected.runtime_requirements {
        if requirement.is_empty() || !requirements.insert(requirement) {
            return Err(
                "execution.json expected.runtimeRequirements must contain unique non-empty strings"
                    .to_owned(),
            );
        }
    }
    Ok(ProjectExecutionCase {
        entry_module: descriptor.entry.module,
        entry_export: descriptor.entry.export,
        invocation,
        runtime_requirements: descriptor.expected.runtime_requirements,
        exit_code: descriptor.expected.process.exit_code,
        stdout: descriptor.expected.stdout,
        stderr: descriptor.expected.stderr,
    })
}

fn validate_fixture_path(label: &str, value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.starts_with('/')
        || value.contains('\\')
        || value
            .split('/')
            .any(|segment| matches!(segment, "" | "." | ".."))
    {
        return Err(format!(
            "execution.json {label} must be a canonical relative path: {value}"
        ));
    }
    Ok(())
}
