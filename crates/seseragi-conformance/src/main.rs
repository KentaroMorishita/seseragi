use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let mut root = PathBuf::from(".");
    let mut list = false;
    for arg in std::env::args().skip(1) {
        if arg == "--list" {
            list = true;
        } else {
            root = PathBuf::from(arg);
        }
    }
    let artifacts = root.join("examples/spec/artifacts");
    let frontend_cases = discover_cases(&artifacts.join("schema-1"));
    let mut token_cases = frontend_cases.clone();
    token_cases.extend(discover_cases(&artifacts.join("token-schema-1")));
    let token_total = token_cases.len();
    let cst_total = frontend_cases.len();
    let surface_ast_total = frontend_cases
        .iter()
        .filter(|case| case.join("surface-ast.json").is_file())
        .count();
    let interface_cases = discover_interface_cases(&artifacts.join("schema-1"));
    let interface_total = interface_cases.len();
    let resolved_ast_cases = discover_resolved_ast_cases(&artifacts.join("schema-1"));
    let resolved_ast_total = resolved_ast_cases.len();
    let typed_hir_cases = discover_artifact_cases(&artifacts, "typed-hir.json");
    let typed_hir_total = typed_hir_cases.len();
    let core_ir_cases = discover_artifact_cases(&artifacts, "core-ir.json");
    let core_ir_total = core_ir_cases.len();
    let typescript_ir_cases = discover_artifact_cases(&artifacts, "typescript-ir.json");
    let typescript_ir_total = typescript_ir_cases.len();

    if list {
        println!("TokenStream fixtures:");
        for case in &token_cases {
            println!("{}", case.display());
        }
        println!("LosslessCst fixtures:");
        for case in &frontend_cases {
            println!("{}", case.display());
        }
        println!("SurfaceAst fixtures:");
        for case in &frontend_cases {
            if case.join("surface-ast.json").is_file() {
                println!("{}", case.display());
            }
        }
        println!("ModuleInterface fixtures:");
        for case in &interface_cases {
            println!("{}", case.display());
        }
        println!("ResolvedAst fixtures: {resolved_ast_total}");
        for case in &resolved_ast_cases {
            println!("{}", case.display());
        }
        println!("TypedHir fixtures: {typed_hir_total}");
        for case in &typed_hir_cases {
            println!("{}", case.display());
        }
        println!("CoreIr fixtures: {core_ir_total}");
        for case in &core_ir_cases {
            println!("{}", case.display());
        }
        println!("TypeScriptIr fixtures: {typescript_ir_total}");
        for case in &typescript_ir_cases {
            println!("{}", case.display());
        }
        return;
    }

    let mut failed = 0;
    for case in &token_cases {
        if let Err(error) = check_tokens(case) {
            failed += 1;
            eprintln!("{}: {error}", case.display());
        }
    }
    for case in &frontend_cases {
        if let Err(error) = check_cst(case) {
            failed += 1;
            eprintln!("{}: {error}", case.display());
        }
        if let Err(error) = check_surface_ast(case) {
            failed += 1;
            eprintln!("{}: {error}", case.display());
        }
    }
    for case in &interface_cases {
        if let Err(error) = check_interface_json(case) {
            failed += 1;
            eprintln!("{}: {error}", case.display());
        }
    }
    for case in &resolved_ast_cases {
        if let Err(error) = check_resolved_ast_json(case) {
            failed += 1;
            eprintln!("{}: {error}", case.display());
        }
    }
    for case in &typed_hir_cases {
        if let Err(error) = check_typed_hir_json(case) {
            failed += 1;
            eprintln!("{}: {error}", case.display());
        }
    }
    for case in &core_ir_cases {
        if let Err(error) = check_core_ir_json(case) {
            failed += 1;
            eprintln!("{}: {error}", case.display());
        }
    }
    for case in &typescript_ir_cases {
        if let Err(error) = check_typescript_ir_json(case) {
            failed += 1;
            eprintln!("{}: {error}", case.display());
        }
    }

    if failed > 0 {
        std::process::exit(1);
    }
    println!("TokenStream fixtures: {token_total} passed");
    println!("LosslessCst fixtures: {cst_total} passed");
    println!("SurfaceAst fixtures: {surface_ast_total} passed");
    println!("ModuleInterface fixtures: {interface_total} passed");
    println!("ResolvedAst fixtures: {resolved_ast_total} passed");
    println!("TypedHir fixtures: {typed_hir_total} passed");
    println!("CoreIr fixtures: {core_ir_total} passed");
    println!("TypeScriptIr fixtures: {typescript_ir_total} passed");
}

fn discover_cases(directory: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut cases = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    cases.sort();
    cases
}

fn discover_interface_cases(schema_directory: &Path) -> Vec<PathBuf> {
    let mut cases = discover_cases(schema_directory)
        .into_iter()
        .filter(|case| case.join("interface.json").is_file())
        .filter(|case| diagnostics_are_empty(case))
        .collect::<Vec<_>>();
    cases.sort();
    cases
}

fn discover_resolved_ast_cases(schema_directory: &Path) -> Vec<PathBuf> {
    let mut cases = discover_cases(schema_directory)
        .into_iter()
        .filter(|case| case.join("resolved-ast.json").is_file())
        .filter(|case| diagnostics_are_empty(case))
        .collect::<Vec<_>>();
    cases.sort();
    cases
}

fn discover_artifact_cases(root: &Path, artifact_name: &str) -> Vec<PathBuf> {
    let mut cases = Vec::new();
    collect_artifact_cases(root, artifact_name, &mut cases);
    cases.sort();
    cases
}

fn collect_artifact_cases(directory: &Path, artifact_name: &str, cases: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    let mut entries = entries.filter_map(Result::ok).collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            if path.join(artifact_name).is_file() {
                cases.push(path.clone());
            }
            collect_artifact_cases(&path, artifact_name, cases);
        }
    }
}

fn diagnostics_are_empty(case: &Path) -> bool {
    let diagnostics_path = case.join("diagnostics.json");
    let Ok(raw) = fs::read_to_string(diagnostics_path) else {
        return true;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return false;
    };
    value
        .get("diagnostics")
        .and_then(|diagnostics| diagnostics.as_array())
        .is_some_and(|diagnostics| diagnostics.is_empty())
}

fn check_tokens(case: &Path) -> Result<(), String> {
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

fn check_interface_json(case: &Path) -> Result<(), String> {
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

fn check_resolved_ast_json(case: &Path) -> Result<(), String> {
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

fn check_typed_hir_json(case: &Path) -> Result<(), String> {
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

fn check_core_ir_json(case: &Path) -> Result<(), String> {
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

fn check_typescript_ir_json(case: &Path) -> Result<(), String> {
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

fn interface_source_name(case: &Path) -> Result<String, String> {
    let name = case
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "interface case has no directory name".to_owned())?;
    Ok(format!("artifact/{name}/main.ssrg"))
}

fn parse_module_interface_json(
    source_name: impl Into<String>,
    source: &str,
) -> Result<serde_json::Value, String> {
    let interface = seseragi_syntax::parse_module_interface(source_name, source);
    serde_json::to_value(&interface)
        .map_err(|error| format!("failed to encode ModuleInterface: {error}"))
}

fn parse_resolved_ast_json(
    source_name: impl Into<String>,
    source: &str,
) -> Result<serde_json::Value, String> {
    let interface = seseragi_syntax::parse_module_interface(source_name, source);
    let resolved_ast = seseragi_semantics::resolve_module_interface(interface);
    serde_json::to_value(&resolved_ast)
        .map_err(|error| format!("failed to encode ResolvedAst: {error}"))
}

fn parse_typed_hir_json(
    source_name: impl Into<String>,
    source: &str,
) -> Result<serde_json::Value, String> {
    let typed_hir = seseragi_semantics::type_module(source_name, source);
    serde_json::to_value(&typed_hir).map_err(|error| format!("failed to encode TypedHir: {error}"))
}

fn parse_core_ir_json(
    source_name: impl Into<String>,
    source: &str,
) -> Result<serde_json::Value, String> {
    let typed_hir = seseragi_semantics::type_module(source_name, source);
    let core_ir = seseragi_lowering::lower_typed_module(typed_hir);
    serde_json::to_value(&core_ir).map_err(|error| format!("failed to encode CoreIr: {error}"))
}

fn parse_typescript_ir_json(
    source_name: impl Into<String>,
    source: &str,
) -> Result<serde_json::Value, String> {
    let typed_hir = seseragi_semantics::type_module(source_name, source);
    let core_ir = seseragi_lowering::lower_typed_module(typed_hir);
    let typescript_ir = seseragi_lowering::lower_core_module_to_typescript_ir(core_ir);
    serde_json::to_value(&typescript_ir)
        .map_err(|error| format!("failed to encode TypeScriptIr: {error}"))
}

fn check_cst(case: &Path) -> Result<(), String> {
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

fn check_surface_ast(case: &Path) -> Result<(), String> {
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
