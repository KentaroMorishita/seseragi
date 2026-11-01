#[path = "support/native.rs"]
mod support;

use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::CoreExpr;
use std::process::Command;
use support::{emit, run, rust_id, Scratch};

#[test]
fn array_index_keeps_maybe_bounds_generic_elements_and_saved_callables() {
    run(
        include_str!("../../../examples/spec/fixtures/rust-backend/array-index.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/array-index.stdout").trim_end(),
    );
}

#[test]
fn array_index_evaluates_receiver_then_offset_once_and_stops_at_defects() {
    let source = "fn receiver -> Array<Int> = [10, 20]\nfn offset -> Int = 1\npub fn probe -> Maybe<Int> = (receiver ())[offset ()]\n";
    let artifact = emit(source);
    let main = artifact.source.rfind("fn main()").unwrap();
    let mut generated = artifact.source[..main].to_owned();
    // Instrument only this test copy of the two source functions. The real
    // generated index expression must call each exactly once, in source order.
    for name in ["receiver", "offset"] {
        let start = generated
            .find(&format!(
                "fn {}_direct(",
                rust_id(&format!("native/main::{name}"))
            ))
            .unwrap();
        let body = start + generated[start..].find("-> V {").unwrap() + "-> V {".len();
        generated.insert_str(
            body,
            &format!(" TRACE.with(|trace| trace.borrow_mut().push({name:?})); "),
        );
    }
    generated.push_str(&format!(
        r#"
thread_local! {{ static TRACE: RefCell<Vec<&'static str>> = const {{ RefCell::new(Vec::new()) }}; }}
fn main() {{
 let result = {}(vec![]).apply(V::Unit);
 assert!(result.tagged("std/prelude::Just"));
 assert_eq!(result.payload().render(), "20");
 TRACE.with(|trace| assert_eq!(&*trace.borrow(), &["receiver", "offset"]));
 drop(result); clear_dictionaries(); assert_eq!(LIVE.with(Cell::get), 0);
}}
"#,
        rust_id(&artifact.metadata.entry)
    ));
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&generated))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    for (receiver, offset, expected) in [
        (
            "{ let ignored = 9007199254740991 + 1; [1] }",
            "9007199254740991 * 9007199254740991",
            "Seseragi Int overflow",
        ),
        ("[1]", "9007199254740991 * 9007199254740991", "Int overflow"),
    ] {
        let artifact = emit(&format!(
            "pub fn probe -> Maybe<Int> = ({receiver})[{offset}]\n"
        ));
        let scratch = Scratch::new();
        let output = Command::new(scratch.compile(&artifact.source))
            .env("PATH", "")
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains(expected));
    }
}

#[test]
fn named_array_filter_evaluates_each_source_stage_once_before_the_next_argument() {
    let source = r#"
import * as arrays from "std/array"
fn predicate -> (Int -> Bool) = \value: Int -> value == 2
fn values -> Array<Int> = [1, 2, 3]
pub fn probe -> Array<Int> = arrays.filter (predicate ()) (values ())
fn partial -> Array<Int> = { let saved = arrays.filter (predicate ()); saved (values ()) }
fn referenced -> Array<Int> = {
  let filter: (Int -> Bool) -> Array<Int> -> Array<Int> = arrays.filter
  filter (predicate ()) (values ())
}
"#;
    let artifact = emit(source);
    let main = artifact.source.rfind("fn main()").unwrap();
    let mut generated = artifact.source[..main].to_owned();
    // Observe only the two source-producing functions in this test copy.
    // The generated Array application and its closure stages remain intact.
    for name in ["predicate", "values"] {
        let start = generated
            .find(&format!(
                "fn {}_direct(",
                rust_id(&format!("native/main::{name}"))
            ))
            .unwrap();
        let body = start + generated[start..].find("-> V {").unwrap() + "-> V {".len();
        generated.insert_str(body, &format!(
            " TRACE.with(|trace| trace.borrow_mut().push({name:?})); if {name:?} == \"predicate\" && FAIL_PREDICATE.with(Cell::get) {{ panic!(\"first source stage\") }} "
        ));
    }
    generated.push_str("thread_local! { static TRACE: RefCell<Vec<&'static str>> = const { RefCell::new(vec![]) }; static FAIL_PREDICATE: Cell<bool> = const { Cell::new(false) }; }\nfn main() { std::panic::set_hook(Box::new(|_| {}));\n");
    for name in ["probe", "partial", "referenced"] {
        let entry = rust_id(&format!("native/main::{name}"));
        generated.push_str(&format!(
            r#"
            TRACE.with(|trace| trace.borrow_mut().clear());
            FAIL_PREDICATE.with(|fail| fail.set(false));
            let output = {entry}(vec![]).apply(V::Unit);
            assert_eq!(output.render(), "[2]");
            assert_eq!(TRACE.with(|trace| trace.borrow().clone()), ["predicate", "values"]);
            drop(output); clear_dictionaries(); assert_eq!(LIVE.with(Cell::get), 0);
            TRACE.with(|trace| trace.borrow_mut().clear());
            FAIL_PREDICATE.with(|fail| fail.set(true));
            assert!(std::panic::catch_unwind(|| {entry}(vec![]).apply(V::Unit)).is_err());
            assert_eq!(TRACE.with(|trace| trace.borrow().clone()), ["predicate"]);
            clear_dictionaries(); assert_eq!(LIVE.with(Cell::get), 0);
        "#
        ));
    }
    generated.push_str("}\n");
    let scratch = Scratch::new();
    let result = Command::new(scratch.compile(&generated))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn array_index_rejects_forged_owners_elements_arity_and_evidence() {
    use seseragi_lowering::CoreType;
    let source = "// 雫🙂\npub fn probe -> Maybe<Int> = [41, 42][0]\n";
    let core = compile_core_module(CompileInput::new("array-λ.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    let shown = compile_core_module(CompileInput::new(
        "show.ssrg",
        "native/show",
        "pub fn probe -> String = show 1\n",
    ))
    .unwrap()
    .core_ir;
    let CoreExpr::Call {
        evidence: display_evidence,
        trait_dispatch: display_dispatch,
        ..
    } = &shown.functions[0].body
    else {
        panic!("display call")
    };
    for case in 0..15 {
        let mut changed = core.clone();
        let CoreExpr::Call {
            arguments,
            type_ref,
            evidence,
            trait_dispatch,
            evidence_argument_index,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            ..
        } = &mut changed.functions[0].body
        else {
            panic!("index call")
        };
        match case {
            0 => {
                arguments.pop();
            }
            1 => arguments.push(arguments[1].clone()),
            2 => arguments.swap(0, 1),
            3 => *type_ref = CoreType::Hole,
            4 => {
                *type_ref = CoreType::ExternalNamed {
                    name: "Maybe".into(),
                    canonical: "other::Maybe".into(),
                    arguments: vec![CoreType::Named {
                        name: "Int".into(),
                        arguments: vec![],
                    }],
                }
            }
            5 | 6 => {
                let CoreType::Named { arguments, .. } = type_ref else {
                    panic!("Maybe")
                };
                if case == 5 {
                    arguments.clear()
                } else {
                    arguments[0] = CoreType::Named {
                        name: "Bool".into(),
                        arguments: vec![],
                    };
                }
            }
            7 | 8 | 9 => {
                let CoreExpr::Array { type_ref, .. } = &mut arguments[0] else {
                    panic!("array")
                };
                *type_ref = match case {
                    7 => CoreType::ExternalNamed {
                        name: "Array".into(),
                        canonical: "other::Array".into(),
                        arguments: vec![],
                    },
                    8 => CoreType::Named {
                        name: "Array".into(),
                        arguments: vec![CoreType::Hole],
                    },
                    _ => CoreType::Named {
                        name: "Array".into(),
                        arguments: vec![],
                    },
                };
            }
            10 => *evidence_argument_index = Some(0),
            11 => deferred_evidence_parameters.push(CoreType::Hole),
            12 => deferred_evidence_type_constructor_parameters.push("F".into()),
            13 => *evidence = display_evidence.clone(),
            14 => *trait_dispatch = display_dispatch.clone(),
            _ => unreachable!(),
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001", "case {case}");
        let origin = error.origin.unwrap();
        assert_eq!(origin.source, "array-λ.ssrg");
        assert!(origin.start < origin.end && origin.end <= source.len());
        assert!(source.is_char_boundary(origin.start) && source.is_char_boundary(origin.end));
    }
}
