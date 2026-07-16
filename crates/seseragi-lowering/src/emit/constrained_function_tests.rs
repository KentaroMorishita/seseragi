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

#[test]
fn defers_constrained_function_evidence_until_after_remaining_value_arguments() {
    let source = "\
pub type Badge = | Active
pub trait Ready<A> { fn ready value: A -> String }
instance Ready<Badge> { fn ready value: Badge -> String = \"Badge is ready\" }
fn describe<T> value: T -> suffix: String -> String
where Ready<T> =
  ready value + suffix
fn applyLabel labeler: (String -> String) -> String = labeler \"!\"
pub fn label value: Badge -> String = applyLabel (describe value)
";
    let typed = type_module("artifact/partial-constrained-function/main.ssrg", source);
    let core = lower_typed_module(typed);
    let typescript = lower_core_module_to_typescript_ir(core);
    let bundle = emit_typescript_module(typescript, source);

    assert!(
        bundle.typescript.contains(
            "applyLabel((__ssrg$partial$0: string) => describe(value)(__ssrg$partial$0)(__ssrg$instance$Ready$0))"
        ),
        "{}",
        bundle.typescript
    );
}

#[test]
fn captures_scoped_evidence_in_a_polymorphic_partial_function_value() {
    let source = "\
pub trait Ready<A> { fn ready value: A -> String }
fn describe<T> value: T -> suffix: String -> String
where Ready<T> =
  ready value + suffix
fn applyLabel labeler: (String -> String) -> String = labeler \"!\"
pub fn label<T> value: T -> String
where Ready<T> =
  applyLabel (describe value)
";
    let typed = type_module(
        "artifact/polymorphic-partial-constrained-function/main.ssrg",
        source,
    );
    let core = lower_typed_module(typed);
    let typescript = lower_core_module_to_typescript_ir(core);
    let bundle = emit_typescript_module(typescript, source);

    assert!(
        bundle.typescript.contains(
            "applyLabel((__ssrg$partial$0: string) => describe(value)(__ssrg$partial$0)(__ssrg$evidence$0))"
        ),
        "{}",
        bundle.typescript
    );
}

#[test]
fn captures_scoped_hkt_evidence_in_a_partial_functor_function() {
    let source = "\
pub trait Functor<F<_>> {
  fn map<A, B> f: (A -> B) -> value: F<A> -> F<B>
}
fn transform<F<_>, A, B> f: (A -> B) -> value: F<A> -> F<B>
where Functor<F> =
  map f value
fn applyMapper<F<_>, A, B>
  mapper: (F<A> -> F<B>) -> value: F<A> -> F<B> =
  mapper value
pub fn transformWith<F<_>, A, B>
  f: (A -> B) -> value: F<A> -> F<B>
where Functor<F> =
  applyMapper (transform f) value
";
    let typed = type_module("artifact/polymorphic-partial-functor/main.ssrg", source);
    let core = lower_typed_module(typed);
    let typescript = lower_core_module_to_typescript_ir(core);
    let bundle = emit_typescript_module(typescript, source);

    assert!(
        bundle.typescript.contains(
            "applyMapper((__ssrg$partial$0: unknown) => transform(f)(__ssrg$partial$0)(__ssrg$evidence$0))(value)"
        ),
        "{}",
        bundle.typescript
    );
}

#[test]
fn dispatches_standard_reduce_through_scoped_reducible_evidence() {
    let source = "\
pub type Pair =
  | Pair (Int, Int)

instance Reducible<Pair, Int> {
  fn reduce<B>
    initial: B -> step: (B -> Int -> B) -> values: Pair -> B =
    match values {
      Pair (first, second) -> step (step initial first) second
    }
}

pub fn total<C> values: C -> Int
where Reducible<C, Int> =
  values
  |> reduce 0 (+)

pub fn answer unit: Unit -> Int =
  Pair (20, 22)
  |> total
";
    let typed = type_module("artifact/generic-reducible/main.ssrg", source);
    let core = lower_typed_module(typed);
    let typescript = lower_core_module_to_typescript_ir(core);
    let bundle = emit_typescript_module(typescript, source);

    assert!(
        bundle.typescript.contains(
            "__ssrg$evidence$0[\"reduce\"](0n)((_argument0) => (_argument1) => _ssrg_int64_add(_argument0, _argument1))(values)"
        ),
        "{}",
        bundle.typescript
    );
    assert!(
        !bundle.typescript.contains("@seseragi/runtime/array"),
        "{}",
        bundle.typescript
    );
    assert!(
        bundle
            .typescript
            .contains("total(Pair([20n, 22n] as const))(__ssrg$instance$Reducible$0)"),
        "{}",
        bundle.typescript
    );
}
