use crate::execution;
use crate::pipeline::{
    emit_generated_module, interface_source_name, parse_core_ir_json, parse_diagnostics_json,
    parse_module_interface_json, parse_resolved_ast_json, parse_typed_hir_json,
    parse_typescript_ir_json,
};
use std::fs;
use std::path::Path;

pub(crate) fn check_tokens(case: &Path) -> Result<(), String> {
    let source_path = case.join("main.ssrg");
    let expected_path = case.join("tokens.json");
    let source = fs::read_to_string(&source_path)
        .map_err(|error| format!("failed to read source: {error}"))?;
    let expected = fs::read_to_string(&expected_path)
        .map_err(|error| format!("failed to read expected tokens: {error}"))?;
    let stream = seseragi_syntax::lex("main.ssrg", &source);
    let actual_value = serde_json::to_value(&stream)
        .map_err(|error| format!("failed to encode tokens: {error}"))?;
    let expected_value: serde_json::Value = serde_json::from_str(&expected)
        .map_err(|error| format!("failed to parse expected tokens: {error}"))?;

    if actual_value != expected_value {
        return Err("token artifact mismatch".to_owned());
    }
    if stream.reconstructed_text() != source {
        return Err("token raw text does not reconstruct source".to_owned());
    }
    Ok(())
}

pub(crate) fn check_interface_json(case: &Path) -> Result<(), String> {
    let source_path = case.join("main.ssrg");
    let expected_path = case.join("interface.json");
    let source = fs::read_to_string(&source_path)
        .map_err(|error| format!("failed to read source: {error}"))?;
    let expected = fs::read_to_string(&expected_path)
        .map_err(|error| format!("failed to read expected ModuleInterface: {error}"))?;
    let actual_value = parse_module_interface_json(interface_source_name(case)?, &source)?;
    let expected_value: serde_json::Value = serde_json::from_str(&expected)
        .map_err(|error| format!("failed to parse expected ModuleInterface: {error}"))?;

    if actual_value != expected_value {
        return Err("ModuleInterface artifact mismatch".to_owned());
    }
    Ok(())
}

pub(crate) fn check_diagnostics_json(case: &Path) -> Result<(), String> {
    let source_path = case.join("main.ssrg");
    let expected_path = case.join("diagnostics.json");
    let source = fs::read_to_string(&source_path)
        .map_err(|error| format!("failed to read source: {error}"))?;
    let expected = fs::read_to_string(&expected_path)
        .map_err(|error| format!("failed to read expected Diagnostics: {error}"))?;
    let actual_value = parse_diagnostics_json("main.ssrg", &source)?;
    let expected_value: serde_json::Value = serde_json::from_str(&expected)
        .map_err(|error| format!("failed to parse expected Diagnostics: {error}"))?;

    if actual_value != expected_value {
        return Err("Diagnostics artifact mismatch".to_owned());
    }
    Ok(())
}

pub(crate) fn check_resolved_ast_json(case: &Path) -> Result<(), String> {
    let source_path = case.join("main.ssrg");
    let expected_path = case.join("resolved-ast.json");
    let source = fs::read_to_string(&source_path)
        .map_err(|error| format!("failed to read source: {error}"))?;
    let expected = fs::read_to_string(&expected_path)
        .map_err(|error| format!("failed to read expected ResolvedAst: {error}"))?;
    let actual_value = parse_resolved_ast_json(interface_source_name(case)?, &source)?;
    let expected_value: serde_json::Value = serde_json::from_str(&expected)
        .map_err(|error| format!("failed to parse expected ResolvedAst: {error}"))?;

    if actual_value != expected_value {
        return Err("ResolvedAst artifact mismatch".to_owned());
    }
    Ok(())
}

pub(crate) fn check_typed_hir_json(case: &Path) -> Result<(), String> {
    let source_path = case.join("main.ssrg");
    let expected_path = case.join("typed-hir.json");
    let source = fs::read_to_string(&source_path)
        .map_err(|error| format!("failed to read source: {error}"))?;
    let expected = fs::read_to_string(&expected_path)
        .map_err(|error| format!("failed to read expected TypedHir: {error}"))?;
    let actual_value = parse_typed_hir_json(interface_source_name(case)?, &source)?;
    let expected_value: serde_json::Value = serde_json::from_str(&expected)
        .map_err(|error| format!("failed to parse expected TypedHir: {error}"))?;

    if actual_value != expected_value {
        return Err("TypedHir artifact mismatch".to_owned());
    }
    Ok(())
}

pub(crate) fn check_core_ir_json(case: &Path) -> Result<(), String> {
    let source_path = case.join("main.ssrg");
    let expected_path = case.join("core-ir.json");
    let source = fs::read_to_string(&source_path)
        .map_err(|error| format!("failed to read source: {error}"))?;
    let expected = fs::read_to_string(&expected_path)
        .map_err(|error| format!("failed to read expected CoreIr: {error}"))?;
    let actual_value = parse_core_ir_json(interface_source_name(case)?, &source)?;
    let expected_value: serde_json::Value = serde_json::from_str(&expected)
        .map_err(|error| format!("failed to parse expected CoreIr: {error}"))?;

    if actual_value != expected_value {
        return Err("CoreIr artifact mismatch".to_owned());
    }
    Ok(())
}

pub(crate) fn check_typescript_ir_json(case: &Path) -> Result<(), String> {
    let source_path = case.join("main.ssrg");
    let expected_path = case.join("typescript-ir.json");
    let source = fs::read_to_string(&source_path)
        .map_err(|error| format!("failed to read source: {error}"))?;
    let expected = fs::read_to_string(&expected_path)
        .map_err(|error| format!("failed to read expected TypeScriptIr: {error}"))?;
    let actual_value = parse_typescript_ir_json(interface_source_name(case)?, &source)?;
    let expected_value: serde_json::Value = serde_json::from_str(&expected)
        .map_err(|error| format!("failed to parse expected TypeScriptIr: {error}"))?;

    if actual_value != expected_value {
        return Err("TypeScriptIr artifact mismatch".to_owned());
    }
    Ok(())
}

pub(crate) fn check_generated_module(root: &Path, case: &Path) -> Result<(), String> {
    let source_path = case.join("main.ssrg");
    let expected_metadata_path = case.join("generated-module.json");
    let expected_typescript_path = case.join("main.ts");
    let expected_source_map_path = case.join("main.ts.map");
    let source = fs::read_to_string(&source_path)
        .map_err(|error| format!("failed to read source: {error}"))?;
    let expected_metadata = fs::read_to_string(&expected_metadata_path)
        .map_err(|error| format!("failed to read expected GeneratedModule: {error}"))?;
    let expected_typescript = fs::read_to_string(&expected_typescript_path)
        .map_err(|error| format!("failed to read expected main.ts: {error}"))?;
    let expected_source_map = fs::read_to_string(&expected_source_map_path)
        .map_err(|error| format!("failed to read expected main.ts.map: {error}"))?;
    let bundle = emit_generated_module(interface_source_name(case)?, &source)?;

    let actual_metadata_value = serde_json::to_value(&bundle.metadata)
        .map_err(|error| format!("failed to encode GeneratedModule: {error}"))?;
    let expected_metadata_value: serde_json::Value = serde_json::from_str(&expected_metadata)
        .map_err(|error| format!("failed to parse expected GeneratedModule: {error}"))?;
    if actual_metadata_value != expected_metadata_value {
        return Err("GeneratedModule artifact mismatch".to_owned());
    }
    check_generated_runtime_requirements(root, &actual_metadata_value)?;
    if bundle.typescript != expected_typescript {
        return Err("main.ts artifact mismatch".to_owned());
    }

    let actual_source_map_value = serde_json::to_value(&bundle.source_map)
        .map_err(|error| format!("failed to encode SourceMap: {error}"))?;
    let expected_source_map_value: serde_json::Value =
        serde_json::from_str(&expected_source_map)
            .map_err(|error| format!("failed to parse expected main.ts.map: {error}"))?;
    if actual_source_map_value != expected_source_map_value {
        return Err("main.ts.map artifact mismatch".to_owned());
    }
    Ok(())
}

fn check_generated_runtime_requirements(
    root: &Path,
    generated_module: &serde_json::Value,
) -> Result<(), String> {
    let abi_path = root.join("examples/spec/artifacts/runtime-schema-1/core/abi.json");
    let abi_raw = fs::read_to_string(&abi_path)
        .map_err(|error| format!("failed to read runtime ABI for generated module: {error}"))?;
    let abi: serde_json::Value = serde_json::from_str(&abi_raw)
        .map_err(|error| format!("failed to parse runtime ABI for generated module: {error}"))?;
    let features = abi
        .get("features")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "runtime ABI features must be an array".to_owned())?;
    let available = features
        .iter()
        .filter_map(|feature| feature.get("id").and_then(|value| value.as_str()))
        .collect::<std::collections::BTreeSet<_>>();
    let requirements = generated_module
        .pointer("/runtime/requirements")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "generated module runtime.requirements must be an array".to_owned())?;

    for requirement in requirements {
        let requirement = requirement
            .as_str()
            .ok_or_else(|| "generated module runtime requirement must be a string".to_owned())?;
        if !available.contains(requirement) {
            return Err(format!(
                "generated module requires unknown runtime feature {requirement}"
            ));
        }
    }
    Ok(())
}

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

    let trace_stdout = run
        .pointer("/expected/trace/0/stdout")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "run.json expected.trace[0].stdout is missing".to_owned())?;
    if trace_stdout != stdout {
        return Err("execution stdout trace does not match stdout snapshot".to_owned());
    }
    let expected_exit_code = run
        .pointer("/expected/process/exitCode")
        .and_then(|value| value.as_i64())
        .ok_or_else(|| "run.json expected.process.exitCode is missing".to_owned())?;

    let actual = execution::run_generated_typescript(root, case, &compiled_typescript)?;
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

pub(crate) fn check_cst(case: &Path) -> Result<(), String> {
    let source_path = case.join("main.ssrg");
    let expected_path = case.join("cst.json");
    let source = fs::read_to_string(&source_path)
        .map_err(|error| format!("failed to read source: {error}"))?;
    let expected = fs::read_to_string(&expected_path)
        .map_err(|error| format!("failed to read expected CST: {error}"))?;
    let cst = seseragi_syntax::parse_cst("main.ssrg", &source);
    let actual_value =
        serde_json::to_value(&cst).map_err(|error| format!("failed to encode CST: {error}"))?;
    let expected_value: serde_json::Value = serde_json::from_str(&expected)
        .map_err(|error| format!("failed to parse expected CST: {error}"))?;

    if actual_value != expected_value {
        return Err("CST artifact mismatch".to_owned());
    }
    Ok(())
}

pub(crate) fn check_surface_ast(case: &Path) -> Result<(), String> {
    let source_path = case.join("main.ssrg");
    let expected_path = case.join("surface-ast.json");
    if !expected_path.is_file() {
        return Ok(());
    }

    let source = fs::read_to_string(&source_path)
        .map_err(|error| format!("failed to read source: {error}"))?;
    let expected = fs::read_to_string(&expected_path)
        .map_err(|error| format!("failed to read expected SurfaceAst: {error}"))?;
    let surface_ast = seseragi_syntax::parse_surface_ast("main.ssrg", &source);
    let actual_value = serde_json::to_value(&surface_ast)
        .map_err(|error| format!("failed to encode SurfaceAst: {error}"))?;
    let expected_value: serde_json::Value = serde_json::from_str(&expected)
        .map_err(|error| format!("failed to parse expected SurfaceAst: {error}"))?;

    if actual_value != expected_value {
        return Err("SurfaceAst artifact mismatch".to_owned());
    }
    Ok(())
}
