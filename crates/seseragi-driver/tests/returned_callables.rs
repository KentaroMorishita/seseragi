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
fn applies_returned_functions_with_generic_and_captured_evidence() {
    check(
        r#"
fn returned x: Int -> Int -> Int = \y -> x + y
fn same<A> x: A -> A -> Bool where Eq<A> = \y -> x == y
fn twice<A> x: A -> A -> A -> A = \y -> \z -> z
pub effect fn main = do {
  println $ show (returned 1 2)
  println $ show (same 1 1)
  println $ show (twice 1 2 3)
  let next = returned 4
  println $ show (next 5)
}
"#,
    );
}

#[test]
fn rejects_extra_arguments_and_does_not_generalize_captured_values() {
    for source in [
        "fn f x: Int -> Int = x\npub let bad = f 1 2",
        "fn f<A> x: A -> A = { let keep = x; let wrong: Int = keep; x }",
        "fn f x: Int -> Int = x\nlet keep = f\npub let bad = keep True",
    ] {
        assert!(
            compile_module(CompileInput::new("invalid.ssrg", "fixture/invalid", source)).is_err()
        );
    }
}
