use seseragi_lowering::{
    emit_typescript_module, lower_core_module_to_typescript_ir, lower_typed_module,
};
use seseragi_semantics::type_module;

#[test]
fn primitive_method_partial_captures_the_supplied_value_before_the_lambda() {
    let source =
        "pub fn probe -> Unit = { let saved: Int -> Int = add (9007199254740991 + 1); () }";
    let core = lower_typed_module(type_module("strict-operator.ssrg", source));
    let output =
        emit_typescript_module(lower_core_module_to_typescript_ir(core), source).typescript;
    assert!(
        output
            .contains("const __ssrg$operator$saved0: number = _ssrg_int_add(9007199254740991, 1)"),
        "{output}"
    );
}

#[test]
fn selected_arithmetic_reference_keeps_the_methods_own_first_source_stage() {
    let source = "type Mark = | Mark Int\ninstance Add<Mark, Mark, Mark> { fn add left: Mark -> (Mark -> Mark) = { let bad = 9007199254740991 + 1; \\right -> left } }\npub fn probe -> Unit = { let op: Mark -> Mark -> Mark = (+); let saved = op (Mark 1); () }";
    let core = lower_typed_module(type_module("strict-operator.ssrg", source));
    let output =
        emit_typescript_module(lower_core_module_to_typescript_ir(core), source).typescript;
    assert!(
        !output.contains("(_argument0) => (_argument1) =>"),
        "{output}"
    );
}
