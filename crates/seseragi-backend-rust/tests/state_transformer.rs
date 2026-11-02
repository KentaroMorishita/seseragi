use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{
    core_expr_origin, CoreExpr, CoreInstanceEvidence, CoreTraitDispatch, CoreType,
};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "state-transformer-雪.ssrg",
        "native/state",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:#?}"))
    .core_ir
}

#[test]
fn state_transformer_accepts_standard_custom_generic_saved_and_nested_monads() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/state-transformer.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/state-transformer-effects.ssrg"),
    ] {
        let core = checked(source);
        let artifact = emit_rust_module(&core, "main").unwrap();
        assert!(artifact
            .metadata
            .capabilities
            .iter()
            .any(|c| c == "state-transformer"));
    }
}

fn signature(operation: &str) -> (&str, &str) {
    match operation {
        "run" => ("Int -> st.StateT<Int,Maybe,Int>", "Maybe<(Int,Int)>"),
        "get" => ("Unit", "st.StateT<Int,Maybe,Int>"),
        "put" => ("Int", "st.StateT<Int,Maybe,Unit>"),
        "modify" => ("(Int -> Int)", "st.StateT<Int,Maybe,Unit>"),
        "lift" => ("Maybe<Int>", "st.StateT<Int,Maybe,Int>"),
        _ => unreachable!(),
    }
}

fn source(operation: &str, saved: bool) -> String {
    let (parameter, result) = signature(operation);
    let probe = if saved {
        format!("pub fn probe -> ({parameter} -> {result}) = st.{operation}")
    } else if operation == "run" {
        format!("pub fn probe initial:Int -> value:st.StateT<Int,Maybe,Int> -> {result} = st.run initial value")
    } else {
        format!("pub fn probe value:{parameter} -> {result} = st.{operation} value")
    };
    format!("import * as st from \"std/transformer/state\"\n{probe}\npub fn main -> Int = 0\n")
}

fn named(name: &str) -> CoreType {
    CoreType::Named {
        name: name.into(),
        arguments: vec![],
    }
}

#[test]
fn state_transformer_rejects_forged_call_types_dictionaries_and_stages_at_origin() {
    let seed = checked(&source("get", false));
    let CoreExpr::Call { evidence: seed, .. } = &seed.functions[0].body else {
        panic!()
    };
    for operation in ["run", "get", "put", "modify", "lift"] {
        let core = checked(&source(operation, false));
        emit_rust_module(&core, "main").unwrap();
        let origin = core_expr_origin(&core.functions[0].body).clone();
        for mutation in 0..22 {
            let mut bad = core.clone();
            let CoreExpr::Call {
                arguments,
                type_ref,
                evidence,
                trait_dispatch,
                deferred_evidence_parameters,
                deferred_evidence_type_constructor_parameters,
                evidence_argument_index,
                ..
            } = &mut bad.functions[0].body
            else {
                panic!()
            };
            match mutation {
                0 => *type_ref = CoreType::Hole,
                1 => *type_ref = named("Bool"),
                2 => *type_ref = named("fake::StateT"),
                3 => arguments.clear(),
                4 => arguments.push(arguments[0].clone()),
                5 => evidence.push(seed[0].clone()),
                6 => *evidence_argument_index = Some(0),
                7 => deferred_evidence_parameters.push(CoreType::Hole),
                8 => deferred_evidence_type_constructor_parameters.push("M".into()),
                9 => {
                    *trait_dispatch = Some(CoreTraitDispatch {
                        trait_identity: "std/prelude::Monad".into(),
                        method: "flatMap".into(),
                        method_contract: None,
                    });
                }
                10 => {
                    let argument = arguments.last_mut().unwrap();
                    *argument = CoreExpr::Boolean {
                        value: false,
                        origin: core_expr_origin(argument).clone(),
                    };
                }
                11 => {
                    let CoreExpr::Variable { type_ref, .. } = arguments.last_mut().unwrap() else {
                        panic!()
                    };
                    *type_ref = CoreType::Hole;
                }
                12..=17 => {
                    let owner = if operation == "run" {
                        let CoreExpr::Variable { type_ref, .. } = &mut arguments[1] else {
                            panic!()
                        };
                        type_ref
                    } else {
                        type_ref
                    };
                    let CoreType::ExternalNamed {
                        canonical,
                        arguments,
                        ..
                    } = owner
                    else {
                        panic!("{owner:?}")
                    };
                    match mutation {
                        12 => *canonical = "fake::StateT".into(),
                        13 => arguments.clear(),
                        14 => arguments[0] = named("Maybe"),
                        15 => arguments[1] = named("Int"),
                        16 => arguments[2] = named("Maybe"),
                        17 => arguments[2] = named("String"),
                        _ => unreachable!(),
                    }
                }
                _ if operation == "run" => {
                    evidence.push(seed[0].clone());
                }
                18 => evidence.clear(),
                19 => evidence[0].constraint.trait_identity = Some("fake::Monad".into()),
                20 => evidence[0].constraint.arguments[0] = named("List"),
                21 => evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 99 },
                _ => unreachable!(),
            }
            let error = emit_rust_module(&bad, "main").expect_err("forged StateT call");
            assert_eq!(error.code, "SES-RUST-0001", "{operation}/{mutation}");
            assert_eq!(
                error.origin.as_ref(),
                Some(&origin),
                "{operation}/{mutation}"
            );
        }
    }
}

#[test]
fn state_transformer_saved_references_and_partial_run_keep_type_and_target_checks() {
    for operation in ["run", "get", "put", "modify", "lift"] {
        let core = checked(&source(operation, true));
        emit_rust_module(&core, "main").unwrap();
        let origin = core_expr_origin(&core.functions[0].body).clone();
        let mut bad = core.clone();
        let mut body = &mut bad.functions[0].body;
        while let CoreExpr::Lambda { body: next, .. } = body {
            body = next;
        }
        match body {
            CoreExpr::Call { type_ref, .. } | CoreExpr::Variable { type_ref, .. } => {
                *type_ref = CoreType::Hole;
            }
            other => panic!("{other:?}"),
        }
        let error = emit_rust_module(&bad, "main").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.as_ref(), Some(&origin));
        let (parameter, result) = signature(operation);
        let unused = checked(&format!(
            "import * as st from \"std/transformer/state\"\npub fn probe value:Int -> Int = {{ let saved:{parameter} -> {result} = st.{operation}; value }}\npub fn main -> Int = 0\n"
        ));
        emit_rust_module(&unused, "main").unwrap();
        let error = emit_program_wasm(&[&unused], "native/state", &["probe".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(error.message.contains("target-specific proof"), "{error}");
        assert_eq!(error.origin.unwrap().source, "state-transformer-雪.ssrg");
    }
    let core = checked("import * as st from \"std/transformer/state\"\npub fn probe initial:Int -> (st.StateT<Int,Maybe,Int> -> Maybe<(Int,Int)>) = st.run initial\npub fn main -> Int = 0");
    emit_rust_module(&core, "main").unwrap();
}

#[test]
fn state_transformer_rejects_forged_standard_base_providers_and_factory_children() {
    let core = checked(&source("lift", false));
    for mutation in 0..5 {
        let mut bad = core.clone();
        let CoreExpr::Call { evidence, .. } = &mut bad.functions[0].body else {
            panic!()
        };
        let child = evidence[0].clone();
        let CoreInstanceEvidence::Standard {
            identity,
            type_arguments,
            evidence_arguments,
        } = &mut evidence[0].evidence
        else {
            panic!()
        };
        match mutation {
            0 => *identity = "std/list::Monad".into(),
            1 => *identity = "std/maybe::Functor".into(),
            2 => *identity = "fake::Monad".into(),
            3 => type_arguments.push(CoreType::Hole),
            4 => evidence_arguments.push(child),
            _ => unreachable!(),
        }
        let error = emit_rust_module(&bad, "main").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(
            error.origin.as_ref(),
            Some(core_expr_origin(&core.functions[0].body))
        );
    }
}

#[test]
fn state_transformer_returned_function_requires_proven_argument_types_and_stages() {
    let core = checked("import * as st from \"std/transformer/state\"\nalias Base<A> = Int -> A\npub fn probe value:st.StateT<Int,Base<_>,Int> -> (Int,Int) = st.run 3 value 4\npub fn main -> Int = 0");
    emit_rust_module(&core, "main").unwrap();
    let origin = core_expr_origin(&core.functions[0].body).clone();
    for mutation in 0..7 {
        let mut bad = core.clone();
        let CoreExpr::Call {
            arguments,
            type_ref,
            evidence_argument_index,
            ..
        } = &mut bad.functions[0].body
        else {
            panic!()
        };
        assert_eq!(*evidence_argument_index, Some(2));
        match mutation {
            0 => *evidence_argument_index = None,
            1 => *evidence_argument_index = Some(1),
            2 => *evidence_argument_index = Some(3),
            3 => {
                arguments[2] = CoreExpr::Boolean {
                    value: false,
                    origin: core_expr_origin(&arguments[2]).clone(),
                }
            }
            4 => {
                *type_ref = CoreType::Tuple {
                    elements: vec![named("Bool"), named("Int")],
                }
            }
            5 => {
                arguments.pop();
            }
            6 => arguments.push(arguments[0].clone()),
            _ => unreachable!(),
        }
        let error = emit_rust_module(&bad, "main").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.as_ref(), Some(&origin));
    }
}

#[test]
fn state_transformer_effect_run_validates_the_complete_base_result() {
    let original = checked("import * as st from \"std/transformer/state\"\npub effect fn probe value:st.StateT<Int,Effect<{},String>,Int> -> (Int,Int) fails String = st.run 0 value\npub fn main -> Int = 0");
    let origin = core_expr_origin(&original.functions[0].body).clone();
    for invoke in [false, true] {
        let mut core = original.clone();
        if invoke {
            let CoreExpr::Call {
                callee,
                callee_signature,
                type_ref:
                    CoreType::Named {
                        arguments: types, ..
                    },
                arguments,
                evidence,
                origin,
                ..
            } = core.functions[0].body.clone()
            else {
                panic!()
            };
            core.functions[0].body = CoreExpr::EffectInvoke {
                callee,
                callee_signature,
                arguments,
                evidence,
                origin,
                requirements: types[0].clone(),
                failure: types[1].clone(),
                success: types[2].clone(),
            };
        }
        emit_rust_module(&core, "main").unwrap();
        for mutation in 0..5 {
            let mut bad = core.clone();
            let (requirements, failure, success, arguments) = match &mut bad.functions[0].body {
                CoreExpr::EffectInvoke {
                    requirements,
                    failure,
                    success,
                    arguments,
                    ..
                } => (requirements, failure, success, arguments),
                CoreExpr::Call {
                    type_ref:
                        CoreType::Named {
                            arguments: types, ..
                        },
                    arguments,
                    ..
                } => {
                    let [requirements, failure, success] = types.as_mut_slice() else {
                        panic!()
                    };
                    (requirements, failure, success, arguments)
                }
                _ => panic!(),
            };
            match mutation {
                0 => *success = CoreType::Hole,
                1 => *success = named("Int"),
                2 => *failure = named("Int"),
                3 => *requirements = CoreType::Hole,
                4 => arguments.clear(),
                _ => unreachable!(),
            }
            let error = emit_rust_module(&bad, "main").unwrap_err();
            assert_eq!(error.code, "SES-RUST-0001");
            assert_eq!(error.origin.as_ref(), Some(&origin));
        }
    }
}
