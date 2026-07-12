use super::model::{
    load_project_execution_cases, LoadedProjectExecutionCase, ProjectExecutionKind,
};
use crate::execution::{self, StagedExecutionRequest};
use crate::execution_case::{
    compare_observation, compare_trace, trace_stdout, validate_effect_entry_contract_in_memory,
    validate_final_interface_invocation,
};
use crate::project_compile::{
    compile_project_compile_case, stage_project_typescript, CompiledProjectCompileCase,
};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// Compiles, stages, and runs one entry from a closed project fixture.
pub(crate) fn check_project_execution_case(root: &Path, case: &Path) -> Result<(), String> {
    let execution_cases = load_project_execution_cases(case)?;
    let compiled_case = compile_project_compile_case(case)?;
    let runtime_requirements = project_runtime_requirements(&compiled_case)?;
    let project_id = case
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "project execution directory has no valid name".to_owned())?;
    for execution_case in &execution_cases {
        check_compiled_project_execution(
            root,
            case,
            project_id,
            &compiled_case,
            &runtime_requirements,
            execution_case,
        )
        .map_err(|error| format!("project execution case {}: {error}", execution_case.id))?;
    }
    Ok(())
}

fn check_compiled_project_execution(
    root: &Path,
    project_root: &Path,
    project_id: &str,
    compiled_case: &CompiledProjectCompileCase,
    runtime_requirements: &[String],
    loaded: &LoadedProjectExecutionCase,
) -> Result<(), String> {
    let execution_case = &loaded.case;
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
    validate_final_interface_invocation(
        &compiled.typed_interface,
        &execution_case.entry_export,
        &execution_case.invocation,
    )?;
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
    if runtime_requirements != execution_case.runtime_requirements {
        return Err(format!(
            "project execution runtime requirements mismatch: expected {:?}, got {:?}",
            execution_case.runtime_requirements, runtime_requirements
        ));
    }
    let entry_module_specifier = format!("./{}", compiled.generated.metadata.outputs.typescript);
    let effect_contract = (execution_case.kind == ProjectExecutionKind::Effect)
        .then(|| {
            validate_effect_entry_contract_in_memory(
                &compiled.typed_interface,
                &compiled.generated.metadata,
                &execution_case.entry_export,
                &entry_module_specifier,
                &execution_case.required_environment,
            )
        })
        .transpose()?;
    let execution_kind = format!("project/{project_id}");
    let execution_dir = execution::prepare_execution_dir(root, &execution_kind, &loaded.directory)?;
    stage_project_typescript(&execution_dir, compiled_case)?;
    let actual = execution::run_staged_typescript(
        root,
        &execution_dir,
        StagedExecutionRequest {
            entry_module_specifier: &entry_module_specifier,
            entry_export: &execution_case.entry_export,
            invocation: execution_case.invocation.clone(),
            failure_renderer: effect_contract
                .as_ref()
                .map(|contract| &contract.failure_renderer),
            environment: &execution_case.environment,
            stdin: &execution_case.stdin,
        },
    )?;
    compare_process_output(&loaded.directory, execution_case, actual)
        .map_err(|error| format!("{} in project {}", error, project_root.display()))
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
    if let Some(trace_stdout) = trace_stdout(expected.expected_operation_trace.as_ref())? {
        if trace_stdout != stdout {
            return Err("project execution stdout trace does not match stdout snapshot".to_owned());
        }
    }
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
    compare_observation(
        expected.expected_effect_exit.as_ref(),
        actual.effect_exit.as_ref(),
    )?;
    compare_trace(
        expected.expected_operation_trace.as_ref(),
        actual.operation_trace.as_ref(),
    )?;
    Ok(())
}
