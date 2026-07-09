use crate::checks::{
    check_core_ir_json, check_cst, check_diagnostics_json, check_interface_json,
    check_resolved_ast_json, check_surface_ast, check_tokens, check_typed_hir_json,
    check_typescript_ir_json,
};
use crate::execution_case::check_execution_case;
use crate::generated_module::check_generated_module;
use crate::runtime_abi::check_runtime_abi_case;
use crate::suite::Suite;
use std::path::PathBuf;

pub(crate) fn run(root: PathBuf, list: bool) {
    let artifacts = root.join("examples/spec/artifacts");
    let suite = Suite::discover(&artifacts);

    if list {
        println!("TokenStream fixtures:");
        for case in &suite.token_cases {
            println!("{}", case.display());
        }
        println!("LosslessCst fixtures:");
        for case in &suite.cst_cases {
            println!("{}", case.display());
        }
        println!("Diagnostics fixtures: {}", suite.diagnostics_cases.len());
        for case in &suite.diagnostics_cases {
            println!("{}", case.display());
        }
        println!("SurfaceAst fixtures:");
        for case in &suite.surface_ast_cases {
            println!("{}", case.display());
        }
        println!("ModuleInterface fixtures:");
        for case in &suite.interface_cases {
            println!("{}", case.display());
        }
        println!("ResolvedAst fixtures: {}", suite.resolved_ast_cases.len());
        for case in &suite.resolved_ast_cases {
            println!("{}", case.display());
        }
        println!("TypedHir fixtures: {}", suite.typed_hir_cases.len());
        for case in &suite.typed_hir_cases {
            println!("{}", case.display());
        }
        println!("CoreIr fixtures: {}", suite.core_ir_cases.len());
        for case in &suite.core_ir_cases {
            println!("{}", case.display());
        }
        println!("TypeScriptIr fixtures: {}", suite.typescript_ir_cases.len());
        for case in &suite.typescript_ir_cases {
            println!("{}", case.display());
        }
        println!(
            "GeneratedModule fixtures: {}",
            suite.generated_module_cases.len()
        );
        for case in &suite.generated_module_cases {
            println!("{}", case.display());
        }
        println!("Execution fixtures: {}", suite.execution_cases.len());
        for case in &suite.execution_cases {
            println!("{}", case.display());
        }
        println!("Runtime ABI fixtures: {}", suite.runtime_abi_cases.len());
        for case in &suite.runtime_abi_cases {
            println!("{}", case.display());
        }
        return;
    }

    let mut failed = 0;
    for case in &suite.token_cases {
        if let Err(error) = check_tokens(case) {
            failed += 1;
            eprintln!("{}: {error}", case.display());
        }
    }
    for case in &suite.cst_cases {
        if let Err(error) = check_cst(case) {
            failed += 1;
            eprintln!("{}: {error}", case.display());
        }
    }
    for case in &suite.surface_ast_cases {
        if let Err(error) = check_surface_ast(case) {
            failed += 1;
            eprintln!("{}: {error}", case.display());
        }
    }
    for case in &suite.diagnostics_cases {
        if let Err(error) = check_diagnostics_json(case) {
            failed += 1;
            eprintln!("{}: {error}", case.display());
        }
    }
    for case in &suite.interface_cases {
        if let Err(error) = check_interface_json(case) {
            failed += 1;
            eprintln!("{}: {error}", case.display());
        }
    }
    for case in &suite.resolved_ast_cases {
        if let Err(error) = check_resolved_ast_json(case) {
            failed += 1;
            eprintln!("{}: {error}", case.display());
        }
    }
    for case in &suite.typed_hir_cases {
        if let Err(error) = check_typed_hir_json(case) {
            failed += 1;
            eprintln!("{}: {error}", case.display());
        }
    }
    for case in &suite.core_ir_cases {
        if let Err(error) = check_core_ir_json(case) {
            failed += 1;
            eprintln!("{}: {error}", case.display());
        }
    }
    for case in &suite.typescript_ir_cases {
        if let Err(error) = check_typescript_ir_json(case) {
            failed += 1;
            eprintln!("{}: {error}", case.display());
        }
    }
    for case in &suite.generated_module_cases {
        if let Err(error) = check_generated_module(&root, case) {
            failed += 1;
            eprintln!("{}: {error}", case.display());
        }
    }
    for case in &suite.execution_cases {
        if let Err(error) = check_execution_case(&root, case) {
            failed += 1;
            eprintln!("{}: {error}", case.display());
        }
    }
    for case in &suite.runtime_abi_cases {
        if let Err(error) = check_runtime_abi_case(&root, case) {
            failed += 1;
            eprintln!("{}: {error}", case.display());
        }
    }

    if failed > 0 {
        std::process::exit(1);
    }
    println!("TokenStream fixtures: {} passed", suite.token_cases.len());
    println!("LosslessCst fixtures: {} passed", suite.cst_cases.len());
    println!(
        "Diagnostics fixtures: {} passed",
        suite.diagnostics_cases.len()
    );
    println!(
        "SurfaceAst fixtures: {} passed",
        suite.surface_ast_cases.len()
    );
    println!(
        "ModuleInterface fixtures: {} passed",
        suite.interface_cases.len()
    );
    println!(
        "ResolvedAst fixtures: {} passed",
        suite.resolved_ast_cases.len()
    );
    println!("TypedHir fixtures: {} passed", suite.typed_hir_cases.len());
    println!("CoreIr fixtures: {} passed", suite.core_ir_cases.len());
    println!(
        "TypeScriptIr fixtures: {} passed",
        suite.typescript_ir_cases.len()
    );
    println!(
        "GeneratedModule fixtures: {} passed",
        suite.generated_module_cases.len()
    );
    println!("Execution fixtures: {} passed", suite.execution_cases.len());
    println!(
        "Runtime ABI fixtures: {} passed",
        suite.runtime_abi_cases.len()
    );
}
