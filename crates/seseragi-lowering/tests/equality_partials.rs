use seseragi_lowering::{
    emit_typescript_module, lower_core_module_to_typescript_ir, lower_typed_module,
};
use seseragi_semantics::type_module;

fn output(operator: &str) -> String {
    let source = format!("type Mark = | Mark Int\ninstance Eq<Mark> {{ fn eq left: Mark -> (Mark -> Bool) = {{ let bad = 9007199254740991 + 1; \\right -> True }} }}\npub fn probe -> Unit = {{ let op: Mark -> Mark -> Bool = ({operator}); let saved = op (Mark 1); () }}");
    let core = lower_typed_module(type_module("equality-λ.ssrg", &source));
    emit_typescript_module(lower_core_module_to_typescript_ir(core), &source).typescript
}

#[test]
fn equality_reference_uses_the_selected_methods_actual_first_stage() {
    let output = output("==");
    assert!(
        !output.contains("(_argument0) => (_argument1) =>"),
        "{output}"
    );
}

#[test]
fn negated_equality_captures_the_selected_method_and_each_applied_stage() {
    let output = output("!=");
    assert!(output.contains("const __ssrg$equality$method:"), "{output}");
    assert!(output.contains("const __ssrg$equality$stage:"), "{output}");
    assert!(
        !output.contains("(_argument0) => (_argument1) =>"),
        "{output}"
    );
}
