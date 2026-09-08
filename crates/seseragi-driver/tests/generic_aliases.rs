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
fn retains_generic_schemes_for_module_local_and_operator_aliases() {
    check(
        r#"
fn identity<A> value: A -> A = value
pub let forward = identity
fn local unit: Unit -> Int = {
  let keep = forward
  let text = keep "ok"
  keep 1
}
let prepend = (:)
pub effect fn main = do {
  let keep = identity
  println $ show (forward 1)
  println (forward "ok")
  println $ show (keep True)
  println $ show (local ())
  println $ show (prepend True `[])
}
"#,
    );
}
