use crate::execution;
use std::fs;
use std::path::Path;

mod effect_contract;
pub(crate) mod environment;
mod exit;
mod invocation;

use effect_contract::validate_effect_entry_contract;
use environment::parse_environment_plan;
use exit::{compare_observation, expected_observation};
use invocation::parse_invocation;

pub(crate) fn check_execution_case(root: &Path, case: &Path) -> Result<(), String> {
    let run_path = case.join("run.json");
    let run_raw = fs::read_to_string(&run_path)
        .map_err(|error| format!("failed to read expected run envelope: {error}"))?;
    let run: serde_json::Value = serde_json::from_str(&run_raw)
        .map_err(|error| format!("failed to parse expected run envelope: {error}"))?;

    if run.get("schema") != Some(&serde_json::Value::from(1)) {
        return Err("run.json must use schema 1".to_owned());
    }
    if run.pointer("/target").and_then(|value| value.as_str()) != Some("node-process") {
        return Err("run.json target must be node-process".to_owned());
    }

    let compiled_module = run
        .pointer("/entry/compiledModule")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "run.json entry.compiledModule is missing".to_owned())?;
    if !case.join(compiled_module).is_file() {
        return Err("compiled generated-module.json reference is missing".to_owned());
    }
    let entry_module = run
        .pointer("/entry/module")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "run.json entry.module is missing".to_owned())?;
    let compiled_module_name = execution::resolve_compiled_module_name(case, compiled_module)?;
    if compiled_module_name != entry_module {
        return Err(format!(
            "execution entry module mismatch: expected {entry_module}, got {compiled_module_name}"
        ));
    }
    let entry_export = run
        .pointer("/entry/export")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "run.json entry.export is missing".to_owned())?;
    let compiled_exports = execution::resolve_compiled_exports(case, compiled_module)?;
    if !compiled_exports.iter().any(|export| export == entry_export) {
        return Err(format!(
            "execution entry export {entry_export} is missing from compiled module exports"
        ));
    }
    let expected_runtime_requirements = expected_string_array(
        &run,
        "/expected/runtimeRequirements",
        "run.json expected.runtimeRequirements",
    )?;
    let actual_runtime_requirements =
        execution::resolve_compiled_runtime_requirements(case, compiled_module)?;
    if actual_runtime_requirements != expected_runtime_requirements {
        return Err(format!(
            "execution runtime requirements mismatch: expected {:?}, got {:?}",
            expected_runtime_requirements, actual_runtime_requirements
        ));
    }
    let compiled_typescript = execution::resolve_compiled_typescript(case, compiled_module)?;
    let invocation = parse_invocation(&run)?;
    if matches!(&invocation, execution::Invocation::Effect { .. }) {
        validate_effect_entry_contract(case, &run, entry_export)?;
    }
    let environment = parse_environment_plan(
        &run,
        matches!(&invocation, execution::Invocation::Effect { .. }),
    )?;
    let expected_effect_exit = expected_observation(&run, &invocation)?;
    let stdin = read_stdin_input(case, &run)?;

    let stdout_name = run
        .pointer("/expected/stdout")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "run.json expected.stdout is missing".to_owned())?;
    let stderr_name = run
        .pointer("/expected/stderr")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "run.json expected.stderr is missing".to_owned())?;
    let stdout = fs::read_to_string(case.join(stdout_name))
        .map_err(|error| format!("failed to read expected stdout snapshot: {error}"))?;
    let stderr = fs::read_to_string(case.join(stderr_name))
        .map_err(|error| format!("failed to read expected stderr snapshot: {error}"))?;

    if let Some(trace_stdout) = expected_trace_stdout(&run)? {
        if trace_stdout != stdout {
            return Err("execution stdout trace does not match stdout snapshot".to_owned());
        }
    }
    let expected_exit_code = run
        .pointer("/expected/process/exitCode")
        .and_then(|value| value.as_i64())
        .ok_or_else(|| "run.json expected.process.exitCode is missing".to_owned())?;

    let actual = execution::run_generated_typescript(
        root,
        case,
        &compiled_typescript,
        entry_export,
        invocation,
        &environment,
        &stdin,
    )?;
    if actual.exit_code != expected_exit_code {
        return Err(format!(
            "execution exit code mismatch: expected {expected_exit_code}, got {}",
            actual.exit_code
        ));
    }
    if actual.stdout != stdout {
        return Err("execution stdout mismatch".to_owned());
    }
    if actual.stderr != stderr {
        return Err("execution stderr mismatch".to_owned());
    }
    compare_observation(expected_effect_exit.as_ref(), actual.effect_exit.as_ref())?;

    Ok(())
}

fn read_stdin_input(case: &Path, run: &serde_json::Value) -> Result<String, String> {
    let Some(input) = run.get("input") else {
        return Ok(String::new());
    };
    let stdin_name = input
        .get("stdin")
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "run.json input.stdin must name a fixture file".to_owned())?;
    let stdin_path = case.join(stdin_name);
    if !stdin_path.is_file() {
        return Err("execution stdin fixture is missing".to_owned());
    }
    fs::read_to_string(&stdin_path)
        .map_err(|error| format!("failed to read execution stdin fixture: {error}"))
}

fn expected_trace_stdout(run: &serde_json::Value) -> Result<Option<String>, String> {
    let Some(trace_value) = run.pointer("/expected/trace") else {
        return Ok(None);
    };
    let trace = trace_value
        .as_array()
        .ok_or_else(|| "run.json expected.trace must be an array".to_owned())?;
    let mut stdout = String::new();
    for (index, event) in trace.iter().enumerate() {
        event
            .get("service")
            .and_then(|value| value.as_str())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("run.json expected.trace[{index}].service is missing"))?;
        event
            .get("operation")
            .and_then(|value| value.as_str())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("run.json expected.trace[{index}].operation is missing"))?;
        event
            .get("arguments")
            .and_then(|value| value.as_array())
            .ok_or_else(|| {
                format!("run.json expected.trace[{index}].arguments must be an array")
            })?;
        let event_stdout = event
            .get("stdout")
            .and_then(|value| value.as_str())
            .ok_or_else(|| format!("run.json expected.trace[{index}].stdout is missing"))?;
        stdout.push_str(event_stdout);
    }
    Ok(Some(stdout))
}

fn expected_string_array(
    value: &serde_json::Value,
    pointer: &str,
    label: &str,
) -> Result<Vec<String>, String> {
    value
        .pointer(pointer)
        .and_then(|value| value.as_array())
        .ok_or_else(|| format!("{label} must be an array"))?
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_owned)
                .ok_or_else(|| format!("{label} entries must be strings"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{expected_trace_stdout, read_stdin_input};
    use serde_json::json;
    use std::path::Path;

    #[test]
    fn defaults_execution_stdin_to_empty_when_input_is_omitted() {
        assert_eq!(read_stdin_input(Path::new("."), &json!({})).unwrap(), "");
    }

    #[test]
    fn rejects_input_without_a_stdin_fixture_name() {
        let error = read_stdin_input(Path::new("."), &json!({ "input": {} })).unwrap_err();

        assert!(error.contains("input.stdin"));
    }

    #[test]
    fn permits_process_smoke_cases_without_a_runtime_trace() {
        assert_eq!(
            expected_trace_stdout(&json!({ "expected": {} })).unwrap(),
            None
        );
    }

    #[test]
    fn rejects_malformed_runtime_trace_when_it_is_present() {
        let error = expected_trace_stdout(&json!({ "expected": { "trace": {} } })).unwrap_err();

        assert!(error.contains("expected.trace"));
    }
}
