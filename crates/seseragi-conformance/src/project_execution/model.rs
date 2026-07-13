use crate::execution::Invocation;
#[cfg(test)]
use crate::execution::InvocationArgument;
use crate::execution_case::{
    environment::{parse_environment_plan, parse_required_environment_fields, EnvironmentPlan},
    expected_observation, expected_trace, parse_invocation_document,
};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

#[path = "model/discovery.rs"]
mod discovery;
#[path = "model/schema.rs"]
mod schema;

pub(crate) use discovery::{
    has_project_execution_layout, load_project_execution_cases, LoadedProjectExecutionCase,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProjectExecutionKind {
    Pure,
    Effect,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectExecutionCase {
    pub(crate) kind: ProjectExecutionKind,
    pub(crate) entry_module: String,
    pub(crate) entry_export: String,
    pub(crate) invocation: Invocation,
    pub(crate) environment: EnvironmentPlan,
    pub(crate) required_environment: BTreeMap<String, String>,
    pub(crate) expected_effect_exit: Option<serde_json::Value>,
    pub(crate) expected_operation_trace: Option<serde_json::Value>,
    pub(crate) runtime_requirements: Vec<String>,
    pub(crate) exit_code: i64,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) stdin: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectExecutionDescriptor {
    schema: u32,
    kind: String,
    target: String,
    entry: ProjectExecutionEntry,
    invocation: serde_json::Value,
    #[serde(rename = "requiredEnvironment")]
    required_environment: Option<serde_json::Value>,
    #[serde(rename = "hostEnvironment")]
    host_environment: Option<serde_json::Value>,
    input: Option<ProjectExecutionInput>,
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
struct ProjectExecutionInput {
    stdin: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectExecutionExpected {
    #[serde(rename = "runtimeRequirements")]
    runtime_requirements: Vec<String>,
    process: ProjectExecutionProcess,
    stdout: String,
    stderr: String,
    exit: Option<serde_json::Value>,
    trace: Option<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectExecutionProcess {
    #[serde(rename = "exitCode")]
    exit_code: i64,
}

/// Reads a closed-project execution envelope stored beside `project.json`.
pub(crate) fn load_project_execution_case(case: &Path) -> Result<ProjectExecutionCase, String> {
    let descriptor_path = case.join("execution.json");
    let raw = fs::read_to_string(&descriptor_path)
        .map_err(|error| format!("failed to read execution.json: {error}"))?;
    let document: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|error| format!("failed to parse execution.json: {error}"))?;
    let descriptor: ProjectExecutionDescriptor = serde_json::from_value(document.clone())
        .map_err(|error| format!("failed to parse execution.json: {error}"))?;
    schema::validate_nested_fields(&document)?;
    if descriptor.schema != 1 {
        return Err("execution.json must use schema 1".to_owned());
    }
    let kind = parse_kind(&descriptor.kind)?;
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
    validate_mode(kind, &invocation)?;
    validate_mode_fields(kind, &descriptor)?;
    let is_effect = kind == ProjectExecutionKind::Effect;
    let environment = parse_environment_plan(&document, is_effect)?;
    let required_environment = if is_effect {
        parse_required_environment_fields(&document)?
    } else {
        BTreeMap::new()
    };
    let expected_effect_exit = expected_observation(&document, &invocation)?;
    let expected_operation_trace = expected_trace(&document)?;
    if environment.captures_console() && expected_operation_trace.is_none() {
        return Err(
            "effect-project-execution with capture-console requires expected.trace".to_owned(),
        );
    }
    let stdin = read_stdin(case, descriptor.input.as_ref())?;

    validate_fixture_path("expected.stdout", &descriptor.expected.stdout)?;
    validate_fixture_path("expected.stderr", &descriptor.expected.stderr)?;
    if descriptor.expected.stdout == descriptor.expected.stderr {
        return Err("execution.json expected stdout and stderr must be distinct".to_owned());
    }
    if kind == ProjectExecutionKind::Pure && descriptor.expected.process.exit_code != 0 {
        return Err("pure project execution must expect process exitCode 0".to_owned());
    }
    validate_runtime_requirements(&descriptor.expected.runtime_requirements)?;

    Ok(ProjectExecutionCase {
        kind,
        entry_module: descriptor.entry.module,
        entry_export: descriptor.entry.export,
        invocation,
        environment,
        required_environment,
        expected_effect_exit,
        expected_operation_trace,
        runtime_requirements: descriptor.expected.runtime_requirements,
        exit_code: descriptor.expected.process.exit_code,
        stdout: descriptor.expected.stdout,
        stderr: descriptor.expected.stderr,
        stdin,
    })
}

fn parse_kind(value: &str) -> Result<ProjectExecutionKind, String> {
    match value {
        "pure-project-execution" => Ok(ProjectExecutionKind::Pure),
        "effect-project-execution" => Ok(ProjectExecutionKind::Effect),
        _ => Err(
            "execution.json kind must be pure-project-execution or effect-project-execution"
                .to_owned(),
        ),
    }
}

fn validate_mode(kind: ProjectExecutionKind, invocation: &Invocation) -> Result<(), String> {
    match (kind, invocation) {
        (ProjectExecutionKind::Pure, Invocation::PureJson { .. })
        | (ProjectExecutionKind::Effect, Invocation::Effect { .. }) => Ok(()),
        (ProjectExecutionKind::Pure, Invocation::Effect { .. }) => {
            Err("pure-project-execution only supports pure invocation".to_owned())
        }
        (ProjectExecutionKind::Effect, Invocation::PureJson { .. }) => {
            Err("effect-project-execution requires Effect invocation".to_owned())
        }
    }
}

fn validate_mode_fields(
    kind: ProjectExecutionKind,
    descriptor: &ProjectExecutionDescriptor,
) -> Result<(), String> {
    if kind == ProjectExecutionKind::Effect {
        return Ok(());
    }
    if descriptor.required_environment.is_some()
        || descriptor.host_environment.is_some()
        || descriptor.input.is_some()
        || descriptor.expected.exit.is_some()
        || descriptor.expected.trace.is_some()
    {
        return Err("pure-project-execution must not declare Effect execution fields".to_owned());
    }
    Ok(())
}

fn read_stdin(case: &Path, input: Option<&ProjectExecutionInput>) -> Result<String, String> {
    let Some(input) = input else {
        return Ok(String::new());
    };
    validate_fixture_path("input.stdin", &input.stdin)?;
    let path = case.join(&input.stdin);
    if !path.is_file() {
        return Err("project execution stdin fixture is missing".to_owned());
    }
    fs::read_to_string(path)
        .map_err(|error| format!("failed to read project execution stdin fixture: {error}"))
}

fn validate_runtime_requirements(requirements: &[String]) -> Result<(), String> {
    let mut unique = BTreeSet::new();
    if requirements
        .iter()
        .any(|requirement| requirement.is_empty() || !unique.insert(requirement))
    {
        return Err(
            "execution.json expected.runtimeRequirements must contain unique non-empty strings"
                .to_owned(),
        );
    }
    Ok(())
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

#[cfg(test)]
pub(super) fn unit_effect_invocation() -> Invocation {
    Invocation::Effect {
        arguments: vec![InvocationArgument::Unit],
    }
}
