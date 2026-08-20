//! Browser-facing adapter over the shared single-file compiler driver.
//!
//! This crate owns only the stable JSON boundary used by the playground. It
//! does not parse, resolve, type-check, lower, or reinterpret Effect entry
//! contracts.

use serde::Serialize;
use seseragi_driver::{
    analyze_module, compile_module, format_module, format_module_with_options, CompileInput,
    FormatOptions,
};
use seseragi_lowering::GeneratedBundle;
use seseragi_runtime::{main_contract, MainContract};
use seseragi_syntax::DiagnosticArtifact;
use wasm_bindgen::prelude::*;

mod project;

pub use project::{
    analyze_project, compile_project, format_project_file, format_project_file_with_options,
};

/// Returns stable metadata for the committed browser artifact.
///
/// Native CLI and LSP binaries expose their Git commit and dirty state. The
/// browser artifact intentionally omits those mutable fields so a WASM package
/// generated from identical sources stays fresh across worktrees and commits.
#[wasm_bindgen]
pub fn toolchain_version_json() -> String {
    serde_json::json!({
        "name": "seseragi-wasm",
        "version": env!("CARGO_PKG_VERSION"),
        "target": "wasm32-unknown-unknown",
    })
    .to_string()
}

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

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
enum FormatResponse {
    Success {
        schema: u32,
        source: String,
        changed: bool,
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

/// Analyzes one source without lowering, code generation, Effect execution,
/// or DOM mounting. The returned occurrence tables back hover and Reference
/// queries while diagnostics remain identical to compile responses.
#[wasm_bindgen]
pub fn analyze_single_file(source_name: &str, module_id: &str, source: &str) -> String {
    serde_json::to_string(&analyze_module(CompileInput::new(
        source_name,
        module_id,
        source,
    )))
    .expect("playground analysis response must serialize")
}

/// Formats one source snapshot with the same formatter used by the native CLI
/// and LSP, returning either the complete canonical source or shared parser
/// diagnostics. Invalid source is never returned as a rewritten document.
#[wasm_bindgen]
pub fn format_single_file(source_name: &str, source: &str) -> String {
    format_single_file_response(format_module(source_name, source))
}

/// Formats one source snapshot with an explicit source-column width.
#[wasm_bindgen]
pub fn format_single_file_with_options(source_name: &str, source: &str, line_width: u32) -> String {
    format_single_file_response(format_module_with_options(
        source_name,
        source,
        FormatOptions::new(line_width as usize),
    ))
}

fn format_single_file_response(
    result: Result<seseragi_driver::FormattedSource, DiagnosticArtifact>,
) -> String {
    let response = match result {
        Ok(formatted) => FormatResponse::Success {
            schema: 1,
            source: formatted.text,
            changed: formatted.changed,
        },
        Err(diagnostics) => FormatResponse::Failure {
            schema: 1,
            diagnostics,
        },
    };
    serde_json::to_string(&response).expect("playground format response must serialize")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn exposes_stable_toolchain_metadata() {
        let metadata: Value = serde_json::from_str(&toolchain_version_json()).unwrap();

        assert_eq!(metadata["name"], "seseragi-wasm");
        assert_eq!(metadata["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(metadata["target"], "wasm32-unknown-unknown");
        assert!(metadata.get("commit").is_none());
        assert!(metadata.get("channel").is_none());
        assert!(metadata.get("dirty").is_none());
    }

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
    fn preserves_typescript_precedence_for_grouped_arithmetic() {
        let source = include_str!(
            "../../../examples/spec/artifacts/schema-1/typescript-precedence-grouping/main.ssrg"
        );
        let response: Value = serde_json::from_str(&compile_single_file(
            "main.ssrg",
            "artifact/typescript-precedence-grouping",
            source,
        ))
        .unwrap();

        assert_eq!(response["status"], "success", "{response}");
        let typescript = response["generated"]["typescript"].as_str().unwrap();
        assert!(
            typescript.contains("(value - 1.0) / (value + 1.0)"),
            "{typescript}"
        );
        assert!(
            typescript.contains("(positive - negative) / 2.0"),
            "{typescript}"
        );
        assert!(
            typescript.contains("double((9.0 - 1.0) / 2.0)"),
            "{typescript}"
        );
    }

    #[test]
    fn returns_nested_never_evidence_for_dom_runtime_failures() {
        let source = r#"import * as dom from "std/web/dom"

pub effect fn main -> Unit
fails dom.DomRuntimeError<Never> =
  succeed ()
"#;
        let response: Value = serde_json::from_str(&compile_single_file(
            "main.ssrg",
            "playground/dom-runtime-never",
            source,
        ))
        .unwrap();

        assert_eq!(response["status"], "success");
        assert_eq!(
            response["entry"]["failureRenderer"],
            serde_json::json!({
                "kind": "show",
                "module": "@seseragi/runtime/show",
                "export": "domRuntimeErrorShow",
                "arguments": [{
                    "module": "@seseragi/runtime/show",
                    "export": "neverShow"
                }]
            })
        );
    }

    #[test]
    fn returns_explicit_success_and_environment_contract_diagnostics() {
        let source = include_str!(
            "../../../examples/spec/artifacts/semantic-diagnostics-schema-1/effect-explicit-contract-mismatch/main.ssrg"
        );
        let response: Value = serde_json::from_str(&compile_single_file(
            "main.ssrg",
            "artifact/effect-explicit-contract-mismatch",
            source,
        ))
        .unwrap();

        assert_eq!(response["status"], "failure");
        let diagnostics = response["diagnostics"]["diagnostics"].as_array().unwrap();
        assert_eq!(diagnostics.len(), 3);
        assert_eq!(
            diagnostics
                .iter()
                .map(|diagnostic| diagnostic["messageKey"].as_str().unwrap())
                .collect::<Vec<_>>(),
            [
                "effect.explicit-success-mismatch",
                "effect.explicit-environment-mismatch",
                "effect.explicit-environment-mismatch",
            ]
        );
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

    #[test]
    fn returns_frontend_queries_without_generating_or_running_code() {
        let source = "fn add left: Int -> right: Int -> Int = left + right\nlet addOne = add 1\n";
        let response: Value =
            serde_json::from_str(&analyze_single_file("main.ssrg", "playground/main", source))
                .unwrap();

        assert_eq!(response["schema"], 1);
        assert!(response["diagnostics"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty());
        assert!(response["symbolOccurrences"].as_array().unwrap().len() > 2);
        assert!(response["typeOccurrences"].as_array().unwrap().len() > 2);
        assert!(response["standardLibrary"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["name"] == "join"));
        assert!(response.get("generated").is_none());
    }

    #[test]
    fn returns_shared_standard_html_prop_diagnostics_for_the_playground() {
        let source = r#"import * as html from "std/web/html"

fn view -> html.Html<Never> =
  html.div { clas: "hero", children: "Typo" }
"#;
        let response: Value =
            serde_json::from_str(&analyze_single_file("main.ssrg", "playground/main", source))
                .unwrap();
        let diagnostics = response["diagnostics"]["diagnostics"].as_array().unwrap();

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0]["code"], "SES-L0101");
        assert_eq!(diagnostics[0]["messageKey"], "web.html.unknown-prop");
        assert_eq!(
            diagnostics[0]["fixes"][0]["edits"][0]["replacement"],
            "class"
        );
    }

    #[test]
    fn formats_with_the_shared_driver_and_preserves_invalid_source() {
        let source =
            include_str!("../../seseragi-formatter/tests/fixtures/canonical-layout.input.ssrg");
        let expected =
            include_str!("../../seseragi-formatter/tests/fixtures/canonical-layout.expected.ssrg");
        let shared = format_module("main.ssrg", source).expect("valid source");
        let formatted: Value =
            serde_json::from_str(&format_single_file("main.ssrg", source)).unwrap();

        assert_eq!(shared.text, expected);
        assert_eq!(formatted["status"], "success");
        assert_eq!(formatted["source"], shared.text);
        assert_eq!(formatted["changed"], true);

        let canonical: Value =
            serde_json::from_str(&format_single_file("main.ssrg", &shared.text)).unwrap();
        assert_eq!(canonical["status"], "success");
        assert_eq!(canonical["source"], shared.text);
        assert_eq!(canonical["changed"], false);

        let invalid: Value = serde_json::from_str(&format_single_file(
            "broken.ssrg",
            "pub let broken: Int =\n",
        ))
        .unwrap();
        assert_eq!(invalid["status"], "failure");
        assert!(invalid.get("source").is_none());
        assert!(!invalid["diagnostics"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty());
    }

    #[test]
    fn formats_single_files_with_an_explicit_line_width() {
        let source =
            "let labels = [\"formatter\", \"playground\", \"curriculum\", \"diagnostics\"]\n";
        let default: Value =
            serde_json::from_str(&format_single_file("main.ssrg", source)).unwrap();
        let narrow: Value =
            serde_json::from_str(&format_single_file_with_options("main.ssrg", source, 48))
                .unwrap();

        assert_eq!(narrow["status"], "success");
        assert_ne!(narrow["source"], default["source"]);
        assert!(narrow["source"].as_str().unwrap().contains("[\n"));
    }
}
