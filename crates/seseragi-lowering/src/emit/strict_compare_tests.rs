use seseragi_semantics::type_module;

use crate::{
    emit_typescript_module, lower_core_module_to_typescript_ir_with_options, lower_typed_module,
    TypeScriptLoweringOptions, TypeScriptOutputPlan,
};

fn emit(source: &str) -> String {
    let typed = type_module("artifact/strict-compare/main.ssrg", source);
    let core = lower_typed_module(typed).expect("valid test lowering input");
    let typescript = lower_core_module_to_typescript_ir_with_options(
        core,
        &TypeScriptOutputPlan::default(),
        &TypeScriptLoweringOptions {
            runtime_sources: true,
            ..Default::default()
        },
    )
    .expect("valid test lowering input");
    emit_typescript_module(typescript, source)
        .expect("valid test lowering input")
        .typescript
}

#[test]
fn standard_scalar_equality_and_int_order_use_javascript_operators() {
    let typescript = emit(
        r#"pub fn same left: Int -> right: Int -> Bool = left == right
pub fn differs left: String -> right: String -> Bool = left != right
pub fn flags left: Bool -> right: Bool -> Bool = left == right
pub fn letters left: Char -> right: Char -> Bool = left != right
pub fn units left: Unit -> right: Unit -> Bool = left == right
pub fn below left: Int -> right: Int -> Bool = left < right
pub fn atMost left: Int -> right: Int -> Bool = left <= right
pub fn above left: Int -> right: Int -> Bool = left > right
pub fn atLeast left: Int -> right: Int -> Bool = left >= right
"#,
    );
    for expected in [
        "same = (left: number) => (right: number) => ((left) as number) === right",
        "differs = (left: string) => (right: string) => ((left) as string) !== right",
        "flags = (left: boolean) => (right: boolean) => ((left) as boolean) === right",
        "letters = (left: string) => (right: string) => ((left) as string) !== right",
        "units = (left: undefined) => (right: undefined) => left === right",
        "left < right",
        "left <= right",
        "left > right",
        "left >= right",
    ] {
        assert!(typescript.contains(expected), "{expected}\n{typescript}");
    }
    assert!(
        !typescript.contains("@seseragi/runtime/equality"),
        "{typescript}"
    );
    for dictionary in ["_eq_dictionary", "_ord_dictionary", "$ssrg$atSource"] {
        assert!(
            !typescript.contains(dictionary),
            "{dictionary}\n{typescript}"
        );
    }
}

#[test]
fn other_orders_and_derived_instances_keep_dictionary_dispatch() {
    let typescript = body(&emit(
        r#"pub newtype Level deriving Eq, Ord = Int
pub fn textBelow left: String -> right: String -> Bool = left < right
pub fn charBelow left: Char -> right: Char -> Bool = left < right
pub fn sameList left: List<Int> -> right: List<Int> -> Bool = left == right
pub fn sameLevel left: Level -> right: Level -> Bool = left == right
pub fn levelBelow left: Level -> right: Level -> Bool = left < right
"#,
    ));
    for dispatch in [
        "_ssrg_string_ord_dictionary[\"compare\"](left)(right)",
        "_ssrg_char_ord_dictionary[\"compare\"](left)(right)",
        "[\"eq\"](left)(right)",
        "[\"compare\"](left)(right)",
    ] {
        assert!(typescript.contains(dispatch), "{dispatch}\n{typescript}");
    }
    for operator in ["=== right", "left < right"] {
        assert!(!typescript.contains(operator), "{operator}\n{typescript}");
    }
}

fn body(typescript: &str) -> String {
    typescript
        .lines()
        .filter(|line| !line.starts_with("import "))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn equality_operands_widen_so_strict_typescript_accepts_them() {
    let typescript = emit(
        r#"pub fn texts unit: Unit -> Bool = "a" == "b"
pub fn numbers unit: Unit -> Bool = 40 != 41
pub fn flags unit: Unit -> Bool = True == False
pub fn narrowed x: Bool -> Bool = if x then x == False else x == True
"#,
    );
    for expected in [
        r#"(("a") as string) === "b""#,
        "((40) as number) !== 41",
        "((true) as boolean) === false",
        "x ? ((x) as boolean) === false : ((x) as boolean) === true",
    ] {
        assert!(typescript.contains(expected), "{expected}\n{typescript}");
    }
}
