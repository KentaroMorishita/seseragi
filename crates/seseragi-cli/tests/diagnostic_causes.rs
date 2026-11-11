//! A diagnostic names the error to fix, not the errors that follow from it:
//! an unresolved type name or an argument that fails to check is reported
//! alone. Each fixture's sidecar lists every diagnostic `seseragi run` reports.
use serde_json::Value;
use std::{path::Path, process::Command};

fn reports_only_the_expected_diagnostics(name: &str) {
    let fixtures =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/spec/fixtures/diagnostics");
    let expected: Value = serde_json::from_str(
        &std::fs::read_to_string(fixtures.join(format!("{name}.expect.json"))).unwrap(),
    )
    .unwrap();
    let source = fixtures.join(format!("{name}.ssrg"));
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(["run", "--diagnostic-format", "json"])
        .arg(&source)
        .output()
        .unwrap();
    assert!(!output.status.success(), "{name} must be refused");
    let report: Value = serde_json::from_slice(&output.stderr)
        .unwrap_or_else(|_| panic!("{}", String::from_utf8_lossy(&output.stderr)));
    let actual = report["diagnostics"][0]["diagnostics"]["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|diagnostic| {
            let mut entry = serde_json::json!({
                "code": diagnostic["code"],
                "severity": diagnostic["severity"],
                "start": diagnostic["primary"]["start"],
                "end": diagnostic["primary"]["end"],
            });
            let labels = diagnostic["labels"]
                .as_array()
                .unwrap()
                .iter()
                .map(|label| label["message"].clone())
                .collect::<Vec<_>>();
            if !labels.is_empty() {
                entry["labels"] = Value::Array(labels);
            }
            entry
        })
        .collect::<Vec<_>>();
    let expected = expected["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|diagnostic| {
            let mut entry = serde_json::json!({
                "code": diagnostic["code"],
                "severity": diagnostic["severity"],
                "start": diagnostic["primary"]["start"],
                "end": diagnostic["primary"]["end"],
            });
            if let Some(labels) = diagnostic.get("labels") {
                entry["labels"] = labels.clone();
            }
            entry
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, expected, "{name}");
}

#[test]
fn an_unresolved_type_name_is_reported_alone_with_how_to_reach_it() {
    reports_only_the_expected_diagnostics("unresolved-namespace-type");
    reports_only_the_expected_diagnostics("unresolved-parameter-type");
}

#[test]
fn an_argument_error_is_reported_instead_of_a_missing_show_instance() {
    reports_only_the_expected_diagnostics("show-argument-error");
    reports_only_the_expected_diagnostics("println-argument-error");
    reports_only_the_expected_diagnostics("template-argument-error");
}
