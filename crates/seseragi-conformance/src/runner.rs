use crate::checks::{
    check_core_ir_json, check_cst, check_diagnostics_json, check_execution_case,
    check_generated_module, check_interface_json, check_resolved_ast_json, check_surface_ast,
    check_tokens, check_typed_hir_json, check_typescript_ir_json,
};
use crate::discovery::{
    discover_artifact_cases, discover_cases, discover_interface_cases, discover_resolved_ast_cases,
};
use crate::runtime_abi::check_runtime_abi_case;
use std::path::PathBuf;

pub(crate) fn run(root: PathBuf, list: bool) {
    let artifacts = root.join("examples/spec/artifacts");
    let frontend_cases = discover_cases(&artifacts.join("schema-1"));
    let mut token_cases = frontend_cases.clone();
    token_cases.extend(discover_cases(&artifacts.join("token-schema-1")));
    let token_total = token_cases.len();
    let cst_total = frontend_cases.len();
    let diagnostics_cases =
        discover_artifact_cases(&artifacts.join("schema-1"), "diagnostics.json");
    let diagnostics_total = diagnostics_cases.len();
    let mut surface_ast_cases =
        discover_artifact_cases(&artifacts.join("schema-1"), "surface-ast.json");
    surface_ast_cases.extend(discover_artifact_cases(
        &artifacts.join("interface-schema-1"),
        "surface-ast.json",
    ));
    surface_ast_cases.sort();
    let surface_ast_total = surface_ast_cases.len();
    let mut interface_cases = discover_interface_cases(&artifacts.join("schema-1"));
    interface_cases.extend(discover_interface_cases(
        &artifacts.join("interface-schema-1"),
    ));
    let interface_total = interface_cases.len();
    let resolved_ast_cases = discover_resolved_ast_cases(&artifacts.join("schema-1"));
    let resolved_ast_total = resolved_ast_cases.len();
    let typed_hir_cases = discover_artifact_cases(&artifacts, "typed-hir.json");
    let typed_hir_total = typed_hir_cases.len();
    let core_ir_cases = discover_artifact_cases(&artifacts, "core-ir.json");
    let core_ir_total = core_ir_cases.len();
    let typescript_ir_cases = discover_artifact_cases(&artifacts, "typescript-ir.json");
    let typescript_ir_total = typescript_ir_cases.len();
    let generated_module_cases = discover_artifact_cases(&artifacts, "generated-module.json");
    let generated_module_total = generated_module_cases.len();
    let execution_cases = discover_artifact_cases(&artifacts, "run.json");
    let execution_total = execution_cases.len();
    let runtime_abi_cases =
        discover_artifact_cases(&artifacts.join("runtime-schema-1"), "abi.json");
    let runtime_abi_total = runtime_abi_cases.len();

    if list {
        println!("TokenStream fixtures:");
        for case in &token_cases {
            println!("{}", case.display());
        }
        println!("LosslessCst fixtures:");
        for case in &frontend_cases {
            println!("{}", case.display());
        }
        println!("Diagnostics fixtures: {diagnostics_total}");
        for case in &diagnostics_cases {
            println!("{}", case.display());
        }
        println!("SurfaceAst fixtures:");
        for case in &surface_ast_cases {
            println!("{}", case.display());
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
        println!("GeneratedModule fixtures: {generated_module_total}");
        for case in &generated_module_cases {
            println!("{}", case.display());
        }
        println!("Execution fixtures: {execution_total}");
        for case in &execution_cases {
            println!("{}", case.display());
        }
        println!("Runtime ABI fixtures: {runtime_abi_total}");
        for case in &runtime_abi_cases {
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
    }
    for case in &surface_ast_cases {
        if let Err(error) = check_surface_ast(case) {
            failed += 1;
            eprintln!("{}: {error}", case.display());
        }
    }
    for case in &diagnostics_cases {
        if let Err(error) = check_diagnostics_json(case) {
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
    for case in &generated_module_cases {
        if let Err(error) = check_generated_module(case) {
            failed += 1;
            eprintln!("{}: {error}", case.display());
        }
    }
    for case in &execution_cases {
        if let Err(error) = check_execution_case(&root, case) {
            failed += 1;
            eprintln!("{}: {error}", case.display());
        }
    }
    for case in &runtime_abi_cases {
        if let Err(error) = check_runtime_abi_case(case) {
            failed += 1;
            eprintln!("{}: {error}", case.display());
        }
    }

    if failed > 0 {
        std::process::exit(1);
    }
    println!("TokenStream fixtures: {token_total} passed");
    println!("LosslessCst fixtures: {cst_total} passed");
    println!("Diagnostics fixtures: {diagnostics_total} passed");
    println!("SurfaceAst fixtures: {surface_ast_total} passed");
    println!("ModuleInterface fixtures: {interface_total} passed");
    println!("ResolvedAst fixtures: {resolved_ast_total} passed");
    println!("TypedHir fixtures: {typed_hir_total} passed");
    println!("CoreIr fixtures: {core_ir_total} passed");
    println!("TypeScriptIr fixtures: {typescript_ir_total} passed");
    println!("GeneratedModule fixtures: {generated_module_total} passed");
    println!("Execution fixtures: {execution_total} passed");
    println!("Runtime ABI fixtures: {runtime_abi_total} passed");
}
