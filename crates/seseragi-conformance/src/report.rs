use crate::suite::Suite;
use serde_json::json;
use std::path::PathBuf;

pub(crate) struct Failure {
    pub(crate) kind: &'static str,
    pub(crate) case: PathBuf,
    pub(crate) error: String,
}

pub(crate) fn print_list_text(suite: &Suite) {
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
}

pub(crate) fn print_list_json(suite: &Suite) {
    println!(
        "{}",
        json!({
            "schema": 1,
            "kind": "conformance-list",
            "cases": {
                "tokenStream": paths(&suite.token_cases),
                "losslessCst": paths(&suite.cst_cases),
                "diagnostics": paths(&suite.diagnostics_cases),
                "surfaceAst": paths(&suite.surface_ast_cases),
                "moduleInterface": paths(&suite.interface_cases),
                "resolvedAst": paths(&suite.resolved_ast_cases),
                "typedHir": paths(&suite.typed_hir_cases),
                "coreIr": paths(&suite.core_ir_cases),
                "typescriptIr": paths(&suite.typescript_ir_cases),
                "generatedModule": paths(&suite.generated_module_cases),
                "execution": paths(&suite.execution_cases),
                "runtimeAbi": paths(&suite.runtime_abi_cases),
            }
        })
    );
}

pub(crate) fn print_success_text(suite: &Suite) {
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

pub(crate) fn print_run_json(suite: &Suite, failures: &[Failure]) {
    println!(
        "{}",
        json!({
            "schema": 1,
            "kind": "conformance-run",
            "passed": failures.is_empty(),
            "counts": counts(suite),
            "failures": failures.iter().map(|failure| {
                json!({
                    "kind": failure.kind,
                    "case": failure.case.display().to_string(),
                    "error": failure.error,
                })
            }).collect::<Vec<_>>(),
        })
    );
}

fn counts(suite: &Suite) -> serde_json::Value {
    json!({
        "tokenStream": suite.token_cases.len(),
        "losslessCst": suite.cst_cases.len(),
        "diagnostics": suite.diagnostics_cases.len(),
        "surfaceAst": suite.surface_ast_cases.len(),
        "moduleInterface": suite.interface_cases.len(),
        "resolvedAst": suite.resolved_ast_cases.len(),
        "typedHir": suite.typed_hir_cases.len(),
        "coreIr": suite.core_ir_cases.len(),
        "typescriptIr": suite.typescript_ir_cases.len(),
        "generatedModule": suite.generated_module_cases.len(),
        "execution": suite.execution_cases.len(),
        "runtimeAbi": suite.runtime_abi_cases.len(),
    })
}

fn paths(cases: &[PathBuf]) -> Vec<String> {
    cases
        .iter()
        .map(|case| case.display().to_string())
        .collect()
}
