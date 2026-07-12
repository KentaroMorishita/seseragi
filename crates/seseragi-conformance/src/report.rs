use crate::suite::Suite;
use serde_json::json;
use std::path::PathBuf;

struct Category<'suite> {
    json_key: &'static str,
    text_label: &'static str,
    cases: &'suite [PathBuf],
}

pub(crate) struct Failure {
    pub(crate) kind: &'static str,
    pub(crate) case: PathBuf,
    pub(crate) error: String,
}

pub(crate) fn print_list_text(suite: &Suite) {
    for category in categories(suite) {
        if matches!(
            category.json_key,
            "tokenStream" | "losslessCst" | "surfaceAst" | "moduleInterface"
        ) {
            println!("{} fixtures:", category.text_label);
        } else {
            println!("{} fixtures: {}", category.text_label, category.cases.len());
        }
        for case in category.cases {
            println!("{}", case.display());
        }
    }
}

pub(crate) fn print_list_json(suite: &Suite) {
    let cases = categories(suite)
        .into_iter()
        .map(|category| (category.json_key.to_owned(), json!(paths(category.cases))))
        .collect::<serde_json::Map<_, _>>();
    println!(
        "{}",
        json!({
            "schema": 1,
            "kind": "conformance-list",
            "cases": cases,
        })
    );
}

pub(crate) fn print_success_text(suite: &Suite) {
    for category in categories(suite) {
        println!(
            "{} fixtures: {} passed",
            category.text_label,
            category.cases.len()
        );
    }
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
    categories(suite)
        .into_iter()
        .map(|category| (category.json_key.to_owned(), json!(category.cases.len())))
        .collect::<serde_json::Map<_, _>>()
        .into()
}

fn categories(suite: &Suite) -> Vec<Category<'_>> {
    vec![
        Category {
            json_key: "tokenStream",
            text_label: "TokenStream",
            cases: &suite.token_cases,
        },
        Category {
            json_key: "losslessCst",
            text_label: "LosslessCst",
            cases: &suite.cst_cases,
        },
        Category {
            json_key: "diagnostics",
            text_label: "Diagnostics",
            cases: &suite.diagnostics_cases,
        },
        Category {
            json_key: "semanticDiagnostics",
            text_label: "SemanticDiagnostics",
            cases: &suite.semantic_diagnostics_cases,
        },
        Category {
            json_key: "surfaceAst",
            text_label: "SurfaceAst",
            cases: &suite.surface_ast_cases,
        },
        Category {
            json_key: "moduleInterface",
            text_label: "ModuleInterface",
            cases: &suite.interface_cases,
        },
        Category {
            json_key: "resolvedAst",
            text_label: "ResolvedAst",
            cases: &suite.resolved_ast_cases,
        },
        Category {
            json_key: "typedHir",
            text_label: "TypedHir",
            cases: &suite.typed_hir_cases,
        },
        Category {
            json_key: "typedInterface",
            text_label: "TypedInterface",
            cases: &suite.typed_interface_cases,
        },
        Category {
            json_key: "coreIr",
            text_label: "CoreIr",
            cases: &suite.core_ir_cases,
        },
        Category {
            json_key: "typescriptIr",
            text_label: "TypeScriptIr",
            cases: &suite.typescript_ir_cases,
        },
        Category {
            json_key: "generatedModule",
            text_label: "GeneratedModule",
            cases: &suite.generated_module_cases,
        },
        Category {
            json_key: "projectCompile",
            text_label: "ProjectCompile",
            cases: &suite.project_compile_cases,
        },
        Category {
            json_key: "execution",
            text_label: "Execution",
            cases: &suite.execution_cases,
        },
        Category {
            json_key: "runtimeAbi",
            text_label: "Runtime ABI",
            cases: &suite.runtime_abi_cases,
        },
    ]
}

fn paths(cases: &[PathBuf]) -> Vec<String> {
    cases
        .iter()
        .map(|case| case.display().to_string())
        .collect()
}
