use crate::{emit_typescript_module, lower_core_module_to_typescript_ir, lower_typed_module};
use seseragi_semantics::type_module;

#[test]
fn emits_and_invokes_a_local_constrained_function_with_dictionary_evidence() {
    let source = "\
pub type Badge = | Active
pub trait Ready<A> { fn ready value: A -> String }
instance Ready<Badge> { fn ready value: Badge -> String = \"Badge is ready\" }
pub fn describe<T> value: T -> String
where Ready<T> =
  ready value
pub fn label value: Badge -> String = describe value
";
    let typed = type_module("artifact/constrained-function/main.ssrg", source);
    let core = lower_typed_module(typed);
    let typescript = lower_core_module_to_typescript_ir(core);
    let bundle = emit_typescript_module(typescript, source);

    assert!(
        bundle.typescript.contains(
            "export const describe = <T,>(value: T) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => __ssrg$evidence$0[\"ready\"](value)"
        ),
        "{}",
        bundle.typescript
    );
    assert!(
        bundle
            .typescript
            .contains("describe(value)(__ssrg$instance$Ready$0)"),
        "{}",
        bundle.typescript
    );
}

#[test]
fn captures_trait_evidence_in_a_partially_applied_method_value() {
    let source = "\
pub trait Functor<F<_>> {
  fn map<A, B> f: (A -> B) -> value: F<A> -> F<B>
}
instance Functor<Maybe> {
  fn map<A, B> f: (A -> B) -> value: Maybe<A> -> Maybe<B> =
    match value {
      Nothing -> Nothing
      Just item -> Just $ f item
    }
}
fn increment value: Int -> Int = value + 1
fn applyMapper mapper: (Maybe<Int> -> Maybe<Int>) -> value: Maybe<Int> -> Maybe<Int> =
  mapper value
pub fn answer value: Maybe<Int> -> Maybe<Int> =
  applyMapper (map increment) value
";
    let typed = type_module("artifact/partial-functor/main.ssrg", source);
    let core = lower_typed_module(typed);
    let typescript = lower_core_module_to_typescript_ir(core);
    let bundle = emit_typescript_module(typescript, source);

    assert!(
        bundle
            .typescript
            .contains("applyMapper(__ssrg$instance$Functor$0[\"map\"](increment))(value)"),
        "{}",
        bundle.typescript
    );
}
