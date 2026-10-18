use seseragi_lowering::{
    emit_typescript_module, lower_core_module_to_typescript_ir, lower_typed_module,
};
use seseragi_semantics::type_module;
#[test]
fn comparison_partial_evaluates_its_prefix_before_the_remaining_lambda() {
    let source =
        "pub fn probe -> Unit = { let less: Int -> Bool = (<) (9007199254740991 + 1); () }";
    let core = lower_typed_module(type_module("strict-comparison.ssrg", source));
    let output =
        emit_typescript_module(lower_core_module_to_typescript_ir(core), source).typescript;
    assert!(output.contains("const __ssrg$comparison$stage"), "{output}");
}
