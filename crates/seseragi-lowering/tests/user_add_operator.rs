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

#[test]
fn lowers_impl_operator_sugar_through_the_same_dictionary_path() {
    let source = "\
struct Score { value: Int }
impl Score {
  operator + self -> bonus: Int -> Score =
    Score { value: self.value + bonus }
}
pub fn addBonus score: Score -> bonus: Int -> Score = score + bonus
";
    let typed = type_module("artifact/impl-add-operator/main.ssrg", source);
    assert_eq!(typed.instances.len(), 1);
    assert_eq!(typed.instances[0].trait_name, "Add");

    let core = lower_typed_module(typed);
    assert!(matches!(
        core.functions[0].body,
        CoreExpr::Binary { ref evidence, .. }
            if matches!(evidence.as_slice(), [selected]
                if matches!(selected.evidence, seseragi_lowering::CoreInstanceEvidence::Local { .. }))
    ));

    let typescript = lower_core_module_to_typescript_ir(core);
    let bundle = emit_typescript_module(typescript, source);
    assert!(bundle
        .typescript
        .contains("__ssrg$instance$Add$0[\"add\"](score)(bonus)"));
}

#[test]
fn carries_generic_impl_constraints_into_the_operator_dictionary() {
    let source = "\
struct Pair<A> { value: A }
impl<A> Pair<A>
where Add<A, A, A> {
  operator + self -> other: Pair<A> -> Pair<A> =
    Pair { value: self.value + other.value }
}
pub fn combine<A> left: Pair<A> -> right: Pair<A> -> Pair<A>
where Add<A, A, A> =
  left + right
";
    let typed = type_module("artifact/generic-impl-operator/main.ssrg", source);
    assert_eq!(typed.instances.len(), 1);
    assert_eq!(typed.instances[0].constraints.len(), 1);

    let core = lower_typed_module(typed);
    assert!(matches!(
        core.functions[0].body,
        CoreExpr::Binary { ref evidence, .. }
            if matches!(evidence.as_slice(), [selected]
                if matches!(
                    selected.evidence,
                    seseragi_lowering::CoreInstanceEvidence::Local {
                        ref evidence_arguments,
                        ..
                    } if matches!(
                        evidence_arguments.as_slice(),
                        [required] if matches!(
                            required.evidence,
                            seseragi_lowering::CoreInstanceEvidence::Parameter { index: 0 }
                        )
                    )
                ))
    ));
}

#[test]
fn lowers_a_scoped_add_operator_section_to_a_curried_dictionary_method() {
    let source = "\
pub fn apply<T> step: (T -> T -> T) -> left: T -> right: T -> T =
  step left right
pub fn combine<T> left: T -> right: T -> T
where Add<T, T, T> =
  apply (+) left right
";
    let typed = type_module("artifact/user-add-section/main.ssrg", source);
    let core = lower_typed_module(typed);
    let typescript = lower_core_module_to_typescript_ir(core);
    let seseragi_lowering::TypeScriptFunction::ConstFunction { body, .. } =
        &typescript.functions[1];
    let TypeScriptExpr::Call { arguments, .. } = body else {
        panic!("expected apply call");
    };
    assert!(matches!(
        &arguments[0],
        TypeScriptExpr::Lambda { body, .. }
            if matches!(body.as_ref(), TypeScriptExpr::Lambda { body, .. }
                if matches!(body.as_ref(), TypeScriptExpr::DictionaryCall { method, .. }
                    if method == "add"))
    ));
    let bundle = emit_typescript_module(typescript, source);
    assert!(bundle.typescript.contains(
        "(_argument0) => (_argument1) => __ssrg$evidence$0[\"add\"](_argument0)(_argument1)"
    ));
}

#[test]
fn lowers_the_standard_string_add_section_without_int_runtime_imports() {
    let source = "\
pub fn apply step: (String -> String -> String) -> String =
  step \"sese\" \"ragi\"
pub fn name unit: Unit -> String =
  apply (+)
";
    let typed = type_module("artifact/string-add-section/main.ssrg", source);
    let core = lower_typed_module(typed);
    let typescript = lower_core_module_to_typescript_ir(core);
    let bundle = emit_typescript_module(typescript, source);
    assert!(bundle
        .typescript
        .contains("(_argument0) => (_argument1) => _argument0 + _argument1"));
    assert!(!bundle.typescript.contains("@seseragi/runtime/int64"));
}
