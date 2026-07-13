//! Browser-facing adapter over the shared single-file compiler driver.
//!
//! This crate owns only the stable JSON boundary used by the playground. It
//! does not parse, resolve, type-check, lower, or reinterpret Effect entry
//! contracts.

use serde::Serialize;
use seseragi_driver::{compile_module, CompileInput};
use seseragi_lowering::GeneratedBundle;
use seseragi_runtime::{main_contract, MainContract};
use seseragi_syntax::DiagnosticArtifact;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
enum CompileResponse {
    Success {
        schema: u32,
        diagnostics: DiagnosticArtifact,
        generated: Box<GeneratedBundle>,
        #[serde(skip_serializing_if = "Option::is_none")]
        entry: Option<MainContract>,
        #[serde(rename = "entryError", skip_serializing_if = "Option::is_none")]
        entry_error: Option<String>,
    },
    Failure {
        schema: u32,
        diagnostics: DiagnosticArtifact,
    },
}

/// Compiles one already-identified source with the same driver used by the
/// native CLI and LSP, returning a versioned JSON envelope for JavaScript.
#[wasm_bindgen]
pub fn compile_single_file(source_name: &str, module_id: &str, source: &str) -> String {
    let response = match compile_module(CompileInput::new(source_name, module_id, source)) {
        Ok(compiled) => {
            let entry_result = main_contract(&compiled);
            let (entry, entry_error) = match entry_result {
                Ok(contract) => (Some(contract), None),
                Err(error) => (None, Some(error)),
            };
            CompileResponse::Success {
                schema: 1,
                diagnostics: compiled.diagnostics,
                generated: Box::new(compiled.generated),
                entry,
                entry_error,
            }
        }
        Err(diagnostics) => CompileResponse::Failure {
            schema: 1,
            diagnostics,
        },
    };
    serde_json::to_string(&response).expect("playground compile response must serialize")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn returns_generated_code_and_the_shared_main_contract() {
        let source = "pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  println \"Hello, Seseragi!\"\n";
        let response: Value =
            serde_json::from_str(&compile_single_file("main.ssrg", "playground/main", source))
                .unwrap();

        assert_eq!(response["status"], "success");
        assert!(response["generated"]["typescript"]
            .as_str()
            .unwrap()
            .contains("export const main"));
        assert_eq!(response["entry"]["environment"][0]["service"], "console");
    }

    #[test]
    fn returns_structured_driver_diagnostics_without_a_fallback_parser() {
        let response: Value = serde_json::from_str(&compile_single_file(
            "broken.ssrg",
            "playground/broken",
            "pub let broken: Int =\n",
        ))
        .unwrap();

        assert_eq!(response["status"], "failure");
        assert!(!response["diagnostics"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty());
    }
}
