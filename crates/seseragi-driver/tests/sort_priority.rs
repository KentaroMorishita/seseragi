use seseragi_driver::{analyze_module, compile_module, CompileInput};

fn check(source: &str) {
    let input = CompileInput::new("callable.ssrg", "fixture/callable", source);
    let analysis = analyze_module(input);
    assert!(
        analysis.diagnostics.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    compile_module(CompileInput::new(
        "callable.ssrg",
        "fixture/callable",
        source,
    ))
    .unwrap();
}

#[test]
fn supports_temporary_sort_priority_for_arrays_and_lists() {
    check(
        r#"
import * as arrays from "std/array"
import * as lists from "std/list"
fn rank value: String -> Int = if value == "urgent" then 0 else 1
pub effect fn main = do {
  println $ show (arrays.sortBy rank ["normal", "urgent", "later"])
  println $ show (lists.sortBy rank `["normal", "urgent", "later"])
}
"#,
    );
}
