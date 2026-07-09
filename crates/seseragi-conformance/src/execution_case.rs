use crate::execution;
use std::fs;
use std::path::Path;

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

    let trace_stdout = expected_trace_stdout(&run)?;
    if trace_stdout != stdout {
        return Err("execution stdout trace does not match stdout snapshot".to_owned());
    }
    let expected_exit_code = run
        .pointer("/expected/process/exitCode")
        .and_then(|value| value.as_i64())
        .ok_or_else(|| "run.json expected.process.exitCode is missing".to_owned())?;

    let actual =
        execution::run_generated_typescript(root, case, &compiled_typescript, entry_export)?;
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

    Ok(())
}

fn expected_trace_stdout(run: &serde_json::Value) -> Result<String, String> {
    let trace = run
        .pointer("/expected/trace")
        .and_then(|value| value.as_array())
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
    Ok(stdout)
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
