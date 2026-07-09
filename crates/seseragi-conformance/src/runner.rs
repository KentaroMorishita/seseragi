use crate::checks::{
    check_core_ir_json, check_cst, check_diagnostics_json, check_interface_json,
    check_resolved_ast_json, check_semantic_diagnostics_json, check_surface_ast, check_tokens,
    check_typed_hir_json, check_typed_interface_json,
};
use crate::execution_case::check_execution_case;
use crate::generated_module::check_generated_module;
use crate::report::{
    print_list_json, print_list_text, print_run_json, print_success_text, Failure,
};
use crate::runtime_abi::check_runtime_abi_case;
use crate::suite::Suite;
use crate::typescript_ir::check_typescript_ir_json;
use std::path::{Path, PathBuf};

pub(crate) fn run(root: PathBuf, list: bool, json: bool) {
    let artifacts = root.join("examples/spec/artifacts");
    let suite = Suite::discover(&artifacts);

    if list {
        if json {
            print_list_json(&suite);
        } else {
            print_list_text(&suite);
        }
        return;
    }

    let mut failures = Vec::new();
    for case in &suite.token_cases {
        record_failure("tokenStream", case, check_tokens(case), json, &mut failures);
    }
    for case in &suite.cst_cases {
        record_failure("losslessCst", case, check_cst(case), json, &mut failures);
    }
    for case in &suite.surface_ast_cases {
        record_failure(
            "surfaceAst",
            case,
            check_surface_ast(case),
            json,
            &mut failures,
        );
    }
    for case in &suite.diagnostics_cases {
        record_failure(
            "diagnostics",
            case,
            check_diagnostics_json(case),
            json,
            &mut failures,
        );
    }
    for case in &suite.semantic_diagnostics_cases {
        record_failure(
            "semanticDiagnostics",
            case,
            check_semantic_diagnostics_json(case),
            json,
            &mut failures,
        );
    }
    for case in &suite.interface_cases {
        record_failure(
            "moduleInterface",
            case,
            check_interface_json(case),
            json,
            &mut failures,
        );
    }
    for case in &suite.resolved_ast_cases {
        record_failure(
            "resolvedAst",
            case,
            check_resolved_ast_json(case),
            json,
            &mut failures,
        );
    }
    for case in &suite.typed_hir_cases {
        record_failure(
            "typedHir",
            case,
            check_typed_hir_json(case),
            json,
            &mut failures,
        );
    }
    for case in &suite.typed_interface_cases {
        record_failure(
            "typedInterface",
            case,
            check_typed_interface_json(case),
            json,
            &mut failures,
        );
    }
    for case in &suite.core_ir_cases {
        record_failure(
            "coreIr",
            case,
            check_core_ir_json(case),
            json,
            &mut failures,
        );
    }
    for case in &suite.typescript_ir_cases {
        record_failure(
            "typescriptIr",
            case,
            check_typescript_ir_json(&root, case),
            json,
            &mut failures,
        );
    }
    for case in &suite.generated_module_cases {
        record_failure(
            "generatedModule",
            case,
            check_generated_module(&root, case),
            json,
            &mut failures,
        );
    }
    for case in &suite.execution_cases {
        record_failure(
            "execution",
            case,
            check_execution_case(&root, case),
            json,
            &mut failures,
        );
    }
    for case in &suite.runtime_abi_cases {
        record_failure(
            "runtimeAbi",
            case,
            check_runtime_abi_case(&root, case),
            json,
            &mut failures,
        );
    }

    if json {
        print_run_json(&suite, &failures);
    }
    if !failures.is_empty() {
        std::process::exit(1);
    }
    if !json {
        print_success_text(&suite);
    }
}

fn record_failure(
    kind: &'static str,
    case: &Path,
    result: Result<(), String>,
    json: bool,
    failures: &mut Vec<Failure>,
) {
    if let Err(error) = result {
        if !json {
            eprintln!("{}: {error}", case.display());
        }
        failures.push(Failure {
            kind,
            case: case.to_path_buf(),
            error,
        });
    }
}
