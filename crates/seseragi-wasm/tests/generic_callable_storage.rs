use serde_json::Value;
use seseragi_wasm::{analyze_single_file, compile_single_file};

#[test]
fn reports_unresolved_callable_storage_through_both_browser_entrypoints() {
    let source = "let identity = \\value -> value\nlet stored = {apply: identity}\n";
    let compiled: Value = serde_json::from_str(&compile_single_file(
        "main.ssrg",
        "playground/storage",
        source,
    ))
    .unwrap();
    let analyzed: Value = serde_json::from_str(&analyze_single_file(
        "main.ssrg",
        "playground/storage",
        source,
    ))
    .unwrap();
    assert_eq!(compiled["status"], "failure");
    assert_eq!(compiled["diagnostics"], analyzed["diagnostics"]);
    assert!(compiled["diagnostics"]["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|diagnostic| diagnostic["messageKey"] == "call.value-type-unresolved"));
    assert!(compiled.get("generated").is_none());
}
