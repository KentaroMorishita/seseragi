use seseragi_driver::{compile_module, CompileInput};

#[test]
fn transformers_use_standard_higher_kinded_evidence() {
    let source = r#"
import * as mt from "std/transformer/maybe"
import * as et from "std/transformer/either"
import * as rt from "std/transformer/reader"
import * as st from "std/transformer/state"
import * as wt from "std/transformer/writer"
let optional: mt.MaybeT<Maybe, Int> = mt.fromMaybe (Just 3)
let mapped: mt.MaybeT<Maybe, Int> = map (\n: Int -> n + 1) optional
let result: Maybe<Maybe<Int>> = mt.run mapped
let error: et.EitherT<String, Maybe, Int> = et.fromEither (Right 5)
let reader: rt.ReaderT<Int, Maybe, Int> = rt.ask ()
let state: st.StateT<Int, Maybe, Int> = st.get ()
let writer: wt.WriterT<String, Maybe, Unit> = wt.tell "a"
pub let values = (result, et.run error, rt.run 7 reader, st.run 8 state, wt.run writer)
"#;
    let result = compile_module(CompileInput::new(
        "transformer.ssrg",
        "fixture/transformer",
        source,
    ));
    assert!(result.is_ok(), "{result:#?}");
}

#[test]
fn nested_transformers_and_partial_effect_heads_use_ordinary_hkt_types() {
    let source = r#"
import * as mt from "std/transformer/maybe"
import * as st from "std/transformer/state"
import * as eff from "std/effect"
alias Work<A> = mt.MaybeT<Effect<{}, Never, _>, A>
let lifted: Work<Int> = mt.lift (eff.succeed 4)
let mapped: Work<Int> = map (\n: Int -> n + 1) lifted
let nested: mt.MaybeT<st.StateT<Int, Maybe, _>, Int> = mt.fromMaybe (Just 2)
pub let work: Effect<{}, Never, Maybe<Int>> = mt.run mapped
pub let pure = st.run 0 (mt.run nested)
"#;
    let result = compile_module(CompileInput::new("stack.ssrg", "fixture/stack", source));
    assert!(result.is_ok(), "{result:#?}");
}

#[test]
fn user_nominal_hkt_arguments_are_inferred_and_erased_without_runtime_special_cases() {
    let source = r#"
struct Box<M<_>, A> { value: M<A> }
fn make<M<_>, A> value: M<A> -> Box<M, A> = Box { value }
fn unwrap<M<_>, A> box: Box<M, A> -> M<A> = box.value
let boxed: Box<Maybe, Int> = make (Just 3)
pub let result: Maybe<Int> = unwrap boxed
let direct = Box { value: Just 5 }
pub let inferred: Maybe<Int> = unwrap direct
"#;
    let result = compile_module(CompileInput::new("user.ssrg", "fixture/user", source));
    assert!(result.is_ok(), "{result:#?}");
    let output = result.unwrap().generated.typescript;
    assert!(!output.contains("Box<Maybe,"), "{output}");
}

#[test]
fn public_transformer_struct_fields_preserve_applied_base_types() {
    let source = r#"
import { MaybeT } from "std/transformer/maybe"
let wrapped: MaybeT<Maybe, Int> = MaybeT { run: Just (Just 2) }
pub let value: Maybe<Maybe<Int>> = wrapped.run
"#;
    let result = compile_module(CompileInput::new("field.ssrg", "fixture/field", source));
    assert!(result.is_ok(), "{result:#?}");
}

#[test]
fn writer_requires_output_monoid_and_no_implicit_transformer_lift() {
    for source in [
        r#"import * as wt from "std/transformer/writer"
pub let value: wt.WriterT<Int, Maybe, Unit> = wt.tell 1"#,
        r#"import * as mt from "std/transformer/maybe"
pub let value: mt.MaybeT<Maybe, Int> = Just 1"#,
    ] {
        assert!(compile_module(CompileInput::new(
            "negative.ssrg",
            "fixture/negative",
            source
        ))
        .is_err());
    }
}
