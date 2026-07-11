use seseragi_lowering::{
    emit_typescript_module, lower_core_module_to_typescript_ir, lower_typed_module,
    TypeScriptBinding, TypeScriptExpr, TypeScriptFunction,
};
use seseragi_semantics::type_module;

#[test]
fn lowers_standard_sum_constructors_to_runtime_bindings() {
    let source = "type Hand = | Rock\n\
                  type HandInputError = | InvalidHand\n\
                  pub fn accepted hand: Hand -> Either<HandInputError, Hand> = Right hand\n\
                  pub fn rejected error: HandInputError -> Either<HandInputError, Hand> = Left error\n\
                  pub fn present hand: Hand -> Maybe<Hand> = Just hand\n\
                  pub fn absent unit: Unit -> Maybe<Hand> = Nothing\n";
    let typed = type_module("artifact/standard-sum/main.ssrg", source);
    let core = lower_typed_module(typed);
    let typescript = lower_core_module_to_typescript_ir(core);

    for feature in [
        "core.either.right",
        "core.either.left",
        "core.maybe.just",
        "core.maybe.nothing",
    ] {
        assert!(typescript
            .runtime_requirements
            .iter()
            .any(|requirement| requirement == feature));
        assert!(typescript
            .imports
            .iter()
            .any(|runtime_import| runtime_import.feature == feature));
    }
    assert!(matches!(
        &typescript.functions[0],
        TypeScriptFunction::ConstFunction {
            body: TypeScriptExpr::RuntimeCall { callee, .. },
            ..
        } if callee == "_ssrg_either_Right"
    ));
    assert!(matches!(
        &typescript.functions[3],
        TypeScriptFunction::ConstFunction {
            body: TypeScriptExpr::RuntimeReference { name },
            is_async: false,
            ..
        } if name == "_ssrg_maybe_Nothing"
    ));

    let bundle = emit_typescript_module(typescript, source);
    assert!(bundle.typescript.contains(
        "import { Right as _ssrg_either_Right, Left as _ssrg_either_Left, Just as _ssrg_maybe_Just, Nothing as _ssrg_maybe_Nothing } from \"@seseragi/runtime/sum\""
    ));
    assert!(bundle.typescript.contains("=> _ssrg_either_Right(hand)"));
    assert!(bundle.typescript.contains("=> _ssrg_maybe_Nothing"));
    assert!(!bundle.typescript.contains("_ssrg_maybe_Nothing()"));
    for name in ["Right", "Left", "Just", "Nothing"] {
        assert!(bundle.source_map.names.iter().any(|entry| entry == name));
    }
}

#[test]
fn imports_a_standard_sum_value_used_by_a_top_level_binding() {
    let source = "type Hand = | Rock\npub let absent: Maybe<Hand> = Nothing\n";
    let typed = type_module("artifact/standard-sum-binding/main.ssrg", source);
    let core = lower_typed_module(typed);
    let typescript = lower_core_module_to_typescript_ir(core);

    assert!(typescript
        .imports
        .iter()
        .any(|runtime_import| runtime_import.feature == "core.maybe.nothing"));
    assert!(matches!(
        &typescript.bindings[0],
        TypeScriptBinding::Const {
            initializer: TypeScriptExpr::RuntimeReference { name },
            ..
        } if name == "_ssrg_maybe_Nothing"
    ));
}

#[test]
fn keeps_local_sum_constructors_out_of_runtime_imports() {
    let source = "type Hand = | Rock\n\
                  type Maybe<A> = | Nothing | Just A\n\
                  fn present hand: Hand -> Maybe<Hand> = Just hand\n\
                  fn absent unit: Unit -> Maybe<Hand> = Nothing\n";
    let typed = type_module("artifact/local-sum/main.ssrg", source);
    let core = lower_typed_module(typed);
    let typescript = lower_core_module_to_typescript_ir(core);

    assert!(typescript.imports.iter().all(|runtime_import| {
        !runtime_import.feature.starts_with("core.maybe.")
            && !runtime_import.feature.starts_with("core.either.")
    }));
}

#[test]
fn freshens_only_the_colliding_runtime_reference() {
    let source = "type Hand = | Rock\n\
                  fn _ssrg_maybe_Nothing unit: Unit -> Unit = unit\n\
                  pub fn absent unit: Unit -> Maybe<Hand> = Nothing\n";
    let typed = type_module("artifact/sum-import-collision/main.ssrg", source);
    let core = lower_typed_module(typed);
    let typescript = lower_core_module_to_typescript_ir(core);

    assert!(matches!(
        &typescript.functions[0],
        TypeScriptFunction::ConstFunction { name, .. } if name == "_ssrg_maybe_Nothing"
    ));
    assert!(typescript
        .imports
        .iter()
        .any(|runtime_import| runtime_import.local == "_ssrg_maybe_Nothing_1"));
    assert!(matches!(
        &typescript.functions[1],
        TypeScriptFunction::ConstFunction {
            body: TypeScriptExpr::RuntimeReference { name },
            ..
        } if name == "_ssrg_maybe_Nothing_1"
    ));
}
