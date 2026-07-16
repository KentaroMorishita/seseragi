use seseragi_lowering::{
    emit_typescript_module, lower_core_module_to_typescript_ir, lower_typed_module, CoreExpr,
    TypeScriptExpr,
};
use seseragi_semantics::type_module;

#[test]
fn lowers_user_add_to_the_selected_dictionary_method() {
    let source = "\
pub type Score = | Score Int
instance Add<Score, Score, Score> {
  fn add left: Score -> right: Score -> Score = left
}
pub fn combine left: Score -> right: Score -> Score = left + right
";
    let typed = type_module("artifact/user-add-operator/main.ssrg", source);
    let core = lower_typed_module(typed);
    assert!(matches!(
        core.functions[0].body,
        CoreExpr::Binary { ref evidence, .. }
            if matches!(evidence.as_slice(), [selected]
                if matches!(selected.evidence, seseragi_lowering::CoreInstanceEvidence::Local { .. }))
    ));

    let typescript = lower_core_module_to_typescript_ir(core);
    assert!(matches!(
        &typescript.functions[0],
        seseragi_lowering::TypeScriptFunction::ConstFunction {
            body: TypeScriptExpr::DictionaryCall { method, arguments, .. },
            ..
        } if method == "add" && arguments.len() == 2
    ));
    let bundle = emit_typescript_module(typescript, source);
    assert!(bundle
        .typescript
        .contains("__ssrg$instance$Add$0[\"add\"](left)(right)"));
}
