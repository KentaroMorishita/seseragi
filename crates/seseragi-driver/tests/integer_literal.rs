use seseragi_driver::{compile_core_module, compile_module, format_module, CompileInput};
use seseragi_lowering::{scalar_self_tail_loop, CoreExpr};

#[test]
fn all_spellings_reach_both_backends_as_the_same_integer_with_source_origin() {
    for raw in [
        "65280",
        "65_280",
        "0xFF_00",
        "0Xff00",
        "0b1111111100000000",
        "0o177400",
    ] {
        let source = format!("pub let value: Int = {raw}\n");
        let input = CompileInput::new("number.ssrg", "fixture/number", &source);
        let compiled = compile_module(input).unwrap();
        let shared = compile_core_module(input).unwrap();
        assert_eq!(compiled.core_ir, shared.core_ir);
        let CoreExpr::Integer { value, origin } = &shared.core_ir.bindings[0].value else {
            panic!("expected integer")
        };
        assert_eq!(value, "65280");
        assert_eq!(&source[origin.start..origin.end], raw);
        assert!(compiled.generated.typescript.contains("= 65280;"));
        let formatted = format_module("number.ssrg", &source).unwrap();
        assert!(formatted.text.contains(raw));
        assert_eq!(
            format_module("number.ssrg", &formatted.text).unwrap().text,
            formatted.text
        );
    }
}

#[test]
fn prefixed_and_separated_scalars_share_tail_loop_recognition() {
    let source = "pub fn sum n: Int -> total: Int -> Int = if n <= 0b0 then total + 1_000 else sum (n - 0x1) (total + 0o1)\n";
    let compiled =
        compile_core_module(CompileInput::new("loop.ssrg", "fixture/loop", source)).unwrap();
    assert!(scalar_self_tail_loop(&compiled.core_ir.functions[0]).is_some());
}

#[test]
fn formatter_keeps_prefix_case_separators_signed_patterns_and_templates() {
    let source = "pub fn probe value:Int ->String=match value{-0X2_A->`${0b1_0}`;0o755->show 1_000;_->show 0xFF_00}\n";
    let formatted = format_module("format.ssrg", source).unwrap().text;
    for raw in ["0X2_A", "0b1_0", "0o755", "1_000", "0xFF_00"] {
        assert!(formatted.contains(raw), "{formatted}");
    }
    assert_eq!(
        format_module("format.ssrg", &formatted).unwrap().text,
        formatted
    );
    assert!(compile_module(CompileInput::new(
        "format.ssrg",
        "fixture/format",
        &formatted
    ))
    .is_ok());
    for raw in ["0b102", "0x_FF", "0x20000000000000", "42suffix"] {
        let bad = format!("pub let value = {raw}\n");
        let diagnostics = format_module("format.ssrg", &bad).unwrap_err();
        assert!(diagnostics
            .diagnostics
            .iter()
            .any(|d| d.code == "SES-P0203"));
    }
}
