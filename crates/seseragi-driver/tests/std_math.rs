use seseragi_driver::{compile_module, CompileInput};

#[test]
fn lowers_math_values_and_curried_functions_through_runtime_imports() {
    let source = include_str!("../../../examples/spec/artifacts/schema-1/std-math/main.ssrg");
    let compiled = compile_module(CompileInput::new("math.ssrg", "fixture/math", source)).unwrap();
    let ts = &compiled.generated.typescript;
    assert!(ts.contains("@seseragi/runtime/math"), "{ts}");
    assert!(!ts.contains("Math."), "{ts}");
    assert!(
        !ts.contains("_ssrg_math_pi("),
        "constant must remain a value: {ts}"
    );
}
