use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::execution_case::{environment::EnvironmentPlan, FailureRenderer};
use crate::runtime_stage::stage_runtime;

mod entry;
mod exit;
mod trace;

pub(crate) use entry::{Invocation, InvocationArgument};

pub(crate) struct ExecutionOutput {
    pub(crate) exit_code: i64,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) effect_exit: Option<serde_json::Value>,
    pub(crate) operation_trace: Option<serde_json::Value>,
}

pub(crate) struct ExecutionRequest<'a> {
    pub(crate) compiled_typescript: &'a Path,
    pub(crate) entry_export: &'a str,
    pub(crate) invocation: Invocation,
    pub(crate) failure_renderer: Option<&'a FailureRenderer>,
    pub(crate) environment: &'a EnvironmentPlan,
    pub(crate) stdin: &'a str,
}

pub(crate) fn resolve_compiled_typescript(
    case: &Path,
    compiled_module: &str,
) -> Result<PathBuf, String> {
    let (compiled_module_path, compiled_module) = read_compiled_module(case, compiled_module)?;
    let typescript = compiled_module
        .pointer("/outputs/typescript")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "compiled generated-module.json outputs.typescript is missing".to_owned())?;
    let module_dir = compiled_module_path
        .parent()
        .ok_or_else(|| "compiled generated-module.json has no parent directory".to_owned())?;
    let typescript_path = module_dir.join(typescript);
    if !typescript_path.is_file() {
        return Err("compiled TypeScript output is missing".to_owned());
    }
    Ok(typescript_path)
}

pub(crate) fn resolve_compiled_runtime_requirements(
    case: &Path,
    compiled_module: &str,
) -> Result<Vec<String>, String> {
    let (_, compiled_module) = read_compiled_module(case, compiled_module)?;
    compiled_module
        .pointer("/runtime/requirements")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "compiled generated-module.json runtime.requirements is missing".to_owned())?
        .iter()
        .map(|requirement| {
            requirement.as_str().map(str::to_owned).ok_or_else(|| {
                "compiled generated-module.json runtime requirement must be a string".to_owned()
            })
        })
        .collect()
}

pub(crate) fn resolve_compiled_module_name(
    case: &Path,
    compiled_module: &str,
) -> Result<String, String> {
    let (_, compiled_module) = read_compiled_module(case, compiled_module)?;
    compiled_module
        .pointer("/module")
        .and_then(|value| value.as_str())
        .map(str::to_owned)
        .ok_or_else(|| "compiled generated-module.json module is missing".to_owned())
}

pub(crate) fn resolve_compiled_exports(
    case: &Path,
    compiled_module: &str,
) -> Result<Vec<String>, String> {
    let (_, compiled_module) = read_compiled_module(case, compiled_module)?;
    compiled_module
        .pointer("/exports")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "compiled generated-module.json exports is missing".to_owned())?
        .iter()
        .map(|export| {
            export.as_str().map(str::to_owned).ok_or_else(|| {
                "compiled generated-module.json export name must be a string".to_owned()
            })
        })
        .collect()
}

fn read_compiled_module(
    case: &Path,
    compiled_module: &str,
) -> Result<(PathBuf, serde_json::Value), String> {
    let compiled_module_path = case.join(compiled_module);
    let compiled_module_raw = fs::read_to_string(&compiled_module_path)
        .map_err(|error| format!("failed to read compiled generated-module.json: {error}"))?;
    let compiled_module: serde_json::Value = serde_json::from_str(&compiled_module_raw)
        .map_err(|error| format!("failed to parse compiled generated-module.json: {error}"))?;
    Ok((compiled_module_path, compiled_module))
}

pub(crate) fn run_generated_typescript(
    root: &Path,
    case: &Path,
    request: ExecutionRequest<'_>,
) -> Result<ExecutionOutput, String> {
    let execution_dir = prepare_execution_dir(root, case)?;
    fs::copy(request.compiled_typescript, execution_dir.join("main.ts"))
        .map_err(|error| format!("failed to stage compiled main.ts: {error}"))?;
    stage_runtime(root, &execution_dir)?;
    let observes_effect_exit = matches!(&request.invocation, Invocation::Effect { .. });
    let observes_operation_trace = observes_effect_exit && request.environment.captures_console();
    entry::write_entry(
        &execution_dir,
        request.entry_export,
        request.invocation,
        request.failure_renderer,
        request.environment,
    )?;

    let mut child = Command::new("bun")
        .arg("run")
        .arg(execution_dir.join("entry.ts"))
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to run bun: {error}"))?;
    let mut process_stdin = child
        .stdin
        .take()
        .ok_or_else(|| "failed to open bun stdin".to_owned())?;
    process_stdin
        .write_all(request.stdin.as_bytes())
        .map_err(|error| format!("failed to write bun stdin: {error}"))?;
    drop(process_stdin);

    let output = child
        .wait_with_output()
        .map_err(|error| format!("failed to wait for bun: {error}"))?;
    let effect_exit = observes_effect_exit
        .then(|| exit::read_observation(&execution_dir))
        .transpose()?;
    let operation_trace = observes_operation_trace
        .then(|| trace::read_observation(&execution_dir))
        .transpose()?;
    let exit_code = output
        .status
        .code()
        .ok_or_else(|| "bun process terminated without an exit code".to_owned())?;
    Ok(ExecutionOutput {
        exit_code: i64::from(exit_code),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        effect_exit,
        operation_trace,
    })
}

fn prepare_execution_dir(root: &Path, case: &Path) -> Result<PathBuf, String> {
    let case_name = case
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "execution case has no directory name".to_owned())?;
    let execution_dir = root
        .join("target/seseragi-conformance/execution")
        .join(case_name);
    if execution_dir.exists() {
        fs::remove_dir_all(&execution_dir)
            .map_err(|error| format!("failed to reset execution temp dir: {error}"))?;
    }
    fs::create_dir_all(&execution_dir)
        .map_err(|error| format!("failed to create execution temp dir: {error}"))?;
    Ok(execution_dir)
}
