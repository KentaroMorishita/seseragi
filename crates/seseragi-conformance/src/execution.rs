use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(crate) struct ExecutionOutput {
    pub(crate) exit_code: i64,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
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
    compiled_typescript: &Path,
) -> Result<ExecutionOutput, String> {
    let execution_dir = prepare_execution_dir(root, case)?;
    fs::copy(compiled_typescript, execution_dir.join("main.ts"))
        .map_err(|error| format!("failed to stage compiled main.ts: {error}"))?;
    stage_runtime(root, &execution_dir)?;
    write_entry(&execution_dir)?;

    let output = Command::new("bun")
        .arg("run")
        .arg(execution_dir.join("entry.ts"))
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to run bun: {error}"))?;
    let exit_code = output
        .status
        .code()
        .ok_or_else(|| "bun process terminated without an exit code".to_owned())?;
    Ok(ExecutionOutput {
        exit_code: i64::from(exit_code),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
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

fn stage_runtime(root: &Path, execution_dir: &Path) -> Result<(), String> {
    let runtime_source = root.join("runtime/ts");
    if !runtime_source.is_dir() {
        return Err("runtime/ts is missing".to_owned());
    }
    let runtime_target = execution_dir.join("node_modules/@seseragi/runtime");
    let runtime_parent = runtime_target
        .parent()
        .ok_or_else(|| "runtime target has no parent directory".to_owned())?;
    fs::create_dir_all(runtime_parent)
        .map_err(|error| format!("failed to create execution node_modules: {error}"))?;
    copy_dir(&runtime_source, &runtime_target)
        .map_err(|error| format!("failed to stage @seseragi/runtime: {error}"))
}

fn write_entry(execution_dir: &Path) -> Result<(), String> {
    fs::write(
        execution_dir.join("entry.ts"),
        "import { main } from \"./main.ts\";\nawait main(undefined);\n",
    )
    .map_err(|error| format!("failed to write execution entry.ts: {error}"))
}

fn copy_dir(source: &Path, target: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir(&source_path, &target_path)?;
        } else if file_type.is_file() {
            fs::copy(&source_path, &target_path)?;
        }
    }
    Ok(())
}
