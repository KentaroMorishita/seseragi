use seseragi_driver::{compile_module, CompileInput};

#[test]
fn comparison_operators_select_ord_for_infix_and_function_values() {
    let source = r#"
struct Score { value: Int }
instance Eq<Score> {
  fn eq left: Score -> right: Score -> Bool = left.value == right.value
}
instance Ord<Score> {
  fn compare left: Score -> right: Score -> Ordering = compare right.value left.value
}
pub fn lower left: Score -> right: Score -> Bool = left < right
pub fn ordered<A> left: A -> right: A -> Bool
where Ord<A> = left <= right
pub fn greater left: Score -> right: Score -> Bool = left > right
pub fn atLeast left: Score -> right: Score -> Bool = left >= right
pub let lowerRef: Score -> Score -> Bool = (<)
pub let orderedRef: Score -> Score -> Bool = (<=)
pub fn genericRef<A> left: A -> right: A -> Bool
where Ord<A> = (>=) left right
pub fn scalar left: String -> right: String -> Bool = left < right
pub fn character left: Char -> right: Char -> Bool = left >= right
pub fn partial left: Score -> Score -> Bool = (<=) left
pub fn eqHigherOrder left: Score -> right: Score -> Bool = apply (==) left right
pub fn viaHigherOrder left: Score -> right: Score -> Bool = apply (<) left right
fn apply<A> f: (A -> A -> Bool) -> left: A -> right: A -> Bool = f left right
"#;
    let compiled = compile_module(CompileInput::new("main.ssrg", "test/ord", source))
        .expect("local, scoped and standard Ord should compile");
    let ts = &compiled.generated.typescript;
    assert!(
        ts.contains("[\"compare\"](left)(right))[\"tag\"] === \"Less\""),
        "{ts}"
    );
    assert!(
        ts.contains("__ssrg$evidence$0[\"compare\"](left)(right))[\"tag\"] !== \"Greater\""),
        "{ts}"
    );
    assert!(!ts.contains("left < right"), "{ts}");
}

#[test]
fn missing_ord_and_mismatched_operands_stop_before_lowering() {
    for expression in [
        "1.0 < 2.0",
        "1.0 <= 2.0",
        "1.0 > 2.0",
        "1.0 >= 2.0",
        "1 < True",
    ] {
        let source = format!("pub let invalid = {expression}\n");
        let diagnostics =
            compile_module(CompileInput::new("main.ssrg", "test/ord-negative", &source))
                .expect_err("comparison must require same-type Ord evidence");
        assert!(!diagnostics.diagnostics.is_empty(), "{expression}");
    }
    let source = "pub let invalid: Float -> Float -> Bool = (<)\n";
    assert!(compile_module(CompileInput::new("main.ssrg", "test/ord-negative", source)).is_err());
}

#[test]
fn canonical_numeric_ord_modules_support_comparisons_without_host_coercion() {
    let source = r#"
import * as big from "std/big-int"
import * as decimal from "std/decimal"
pub fn bigLess left: big.BigInt -> right: big.BigInt -> Bool = left < right
pub fn decimalAtLeast left: decimal.Decimal -> right: decimal.Decimal -> Bool = (>=) left right
"#;
    let compiled = compile_module(CompileInput::new("main.ssrg", "test/ord-numeric", source))
        .expect("canonical imported numeric types have Ord");
    assert!(compiled.generated.typescript.contains("bigIntOrd"));
    assert!(compiled.generated.typescript.contains("decimalOrd"));
    assert!(!compiled.generated.typescript.contains("left < right"));
}

#[test]
fn a_nominal_type_without_ord_reports_missing_instance_at_the_operator() {
    let source = "struct Score { value: Int }\npub fn invalid left: Score -> right: Score -> Bool = left < right\n";
    let diagnostics = compile_module(CompileInput::new("main.ssrg", "test/no-ord", source))
        .expect_err("nominal values must not fall back to host object comparison");
    let operator = source.find(" < ").unwrap() + 1;
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .any(|d| d.primary.start == operator && d.primary.end == operator + 1),
        "{diagnostics:?}"
    );
}

#[test]
fn analysis_and_reference_expose_comparisons_with_ord_identity() {
    let source = "pub fn less left: Int -> right: Int -> Bool = left < right\n";
    let analysis = seseragi_driver::analyze_module(CompileInput::new(
        "main.ssrg",
        "test/ord-analysis",
        source,
    ));
    assert!(analysis.diagnostics.diagnostics.is_empty());
    for operator in ["<", "<=", ">", ">="] {
        let item = analysis
            .standard_library
            .iter()
            .find(|item| item.identity == format!("std/prelude::{operator}"))
            .expect("comparison must be visible in the shared Reference catalog");
        assert_eq!(item.constraints, ["Ord"]);
        assert_eq!(
            item.signature.as_deref(),
            Some(format!("{operator} via Ord.compare").as_str())
        );
    }
    let negative = "pub let invalid = 1.0 < 2.0\n";
    let analyzed = seseragi_driver::analyze_module(CompileInput::new(
        "main.ssrg",
        "test/ord-analysis",
        negative,
    ));
    let compiled = compile_module(CompileInput::new(
        "main.ssrg",
        "test/ord-analysis",
        negative,
    ))
    .unwrap_err();
    assert_eq!(analyzed.diagnostics, compiled);
}
