use crate::pipeline::{
    emit_generated_module, interface_source_name, parse_core_ir_json, parse_module_interface_json,
    parse_resolved_ast_json, parse_typed_hir_json, parse_typescript_ir_json,
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

pub(crate) fn check_generated_module(case: &Path) -> Result<(), String> {
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
