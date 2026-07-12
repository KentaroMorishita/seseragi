use super::model::load_project_execution_case;
use crate::execution::{self, StagedExecutionRequest};
use crate::execution_case::environment::EnvironmentPlan;
use crate::project_compile::{
    compile_project_compile_case, stage_project_typescript, CompiledProjectCompileCase,
};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// Compiles, stages, and runs one pure entry from a closed project fixture.
pub(crate) fn check_project_execution_case(root: &Path, case: &Path) -> Result<(), String> {
    let execution_case = load_project_execution_case(case)?;
    let compiled_case = compile_project_compile_case(case)?;
    let compiled = compiled_case
        .compiled
        .modules
        .get(&execution_case.entry_module)
        .ok_or_else(|| {
            format!(
                "execution.json entry module is not declared by project.json: {}",
                execution_case.entry_module
            )
        })?;
    if compiled.generated.metadata.module != execution_case.entry_module {
        return Err("project execution entry module metadata does not match descriptor".to_owned());
    }
    if !compiled
        .generated
        .metadata
        .exports
        .iter()
        .any(|export| export == &execution_case.entry_export)
    {
        return Err(format!(
            "execution.json entry export is missing from generated module: {}",
            execution_case.entry_export
        ));
    }
    let runtime_requirements = project_runtime_requirements(&compiled_case)?;
    if runtime_requirements != execution_case.runtime_requirements {
        return Err(format!(
            "project execution runtime requirements mismatch: expected {:?}, got {:?}",
            execution_case.runtime_requirements, runtime_requirements
        ));
    }
    let entry_output = compiled_case
        .descriptor
        .modules
        .iter()
        .find(|module| module.id == execution_case.entry_module)
        .expect("compiled project entry must have a declared descriptor module");
    let execution_dir = execution::prepare_execution_dir(root, "project", case)?;
    stage_project_typescript(&execution_dir, &compiled_case)?;
    let environment = EnvironmentPlan::empty();
    let entry_module_specifier = format!("./{}", entry_output.output);
    let actual = execution::run_staged_typescript(
        root,
        &execution_dir,
        StagedExecutionRequest {
            entry_module_specifier: &entry_module_specifier,
            entry_export: &execution_case.entry_export,
            invocation: execution_case.invocation.clone(),
            failure_renderer: None,
            environment: &environment,
            stdin: "",
        },
    )?;
    compare_process_output(case, &execution_case, actual)
}

fn project_runtime_requirements(
    compiled_case: &CompiledProjectCompileCase,
) -> Result<Vec<String>, String> {
    let mut unique = BTreeSet::new();
    let mut requirements = Vec::new();
    for module_id in &compiled_case.compiled.order {
        let module = compiled_case
            .compiled
            .modules
            .get(module_id)
            .ok_or_else(|| format!("project compiler omitted runtime module {module_id}"))?;
        for requirement in &module.generated.metadata.runtime.requirements {
            if unique.insert(requirement.clone()) {
                requirements.push(requirement.clone());
            }
        }
    }
    Ok(requirements)
}

fn compare_process_output(
    case: &Path,
    expected: &super::model::ProjectExecutionCase,
    actual: execution::ExecutionOutput,
) -> Result<(), String> {
    let stdout = fs::read_to_string(case.join(&expected.stdout))
        .map_err(|error| format!("failed to read project expected stdout: {error}"))?;
    let stderr = fs::read_to_string(case.join(&expected.stderr))
        .map_err(|error| format!("failed to read project expected stderr: {error}"))?;
    if actual.exit_code != expected.exit_code {
        return Err(format!(
            "project execution exit code mismatch: expected {}, got {}",
            expected.exit_code, actual.exit_code
        ));
    }
    if actual.stdout != stdout {
        return Err("project execution stdout mismatch".to_owned());
    }
    if actual.stderr != stderr {
        return Err("project execution stderr mismatch".to_owned());
    }
    if actual.effect_exit.is_some() || actual.operation_trace.is_some() {
        return Err("pure project execution unexpectedly produced Effect observations".to_owned());
    }
    Ok(())
}
