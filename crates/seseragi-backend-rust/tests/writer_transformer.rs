use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{
    core_expr_origin, CoreExpr, CoreInstanceEvidence, CoreTraitDispatch, CoreType,
};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "writer-transformer-雪.ssrg",
        "native/writer",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:#?}"))
    .core_ir
}

fn signature(operation: &str) -> (&str, &str) {
    match operation {
        "run" => ("wt.WriterT<String,Maybe,Int>", "Maybe<(Int,String)>"),
        "tell" => ("String", "wt.WriterT<String,Maybe,Unit>"),
        "listen" => (
            "wt.WriterT<String,Maybe,Int>",
            "wt.WriterT<String,Maybe,(Int,String)>",
        ),
        "lift" => ("Maybe<Int>", "wt.WriterT<String,Maybe,Int>"),
        _ => unreachable!(),
    }
}

fn source(operation: &str, saved: bool) -> String {
    let (parameter, result) = signature(operation);
    let probe = if saved {
        format!("pub fn probe -> ({parameter} -> {result}) = wt.{operation}")
    } else {
        format!("pub fn probe value:{parameter} -> {result} = wt.{operation} value")
    };
    format!("import * as wt from \"std/transformer/writer\"\n{probe}\npub fn main -> Int = 0\n")
}

fn named(name: &str) -> CoreType {
    CoreType::Named {
        name: name.into(),
        arguments: vec![],
    }
}

#[test]
fn writer_transformer_accepts_standard_custom_generic_saved_and_nested_dictionaries() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/writer-transformer.ssrg"),
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/writer-transformer-effects.ssrg"
        ),
    ] {
        let artifact = emit_rust_module(&checked(source), "main").unwrap();
        assert!(artifact
            .metadata
            .capabilities
            .iter()
            .any(|c| c == "writer-transformer"));
    }
}

#[test]
fn writer_transformer_rejects_forged_call_types_dictionaries_and_stages_at_origin() {
    let seed = checked(&source("tell", false));
    let CoreExpr::Call { evidence: seed, .. } = &seed.functions[0].body else {
        panic!()
    };
    for operation in ["run", "tell", "listen", "lift"] {
        let core = checked(&source(operation, false));
        emit_rust_module(&core, "main").unwrap();
        let origin = core_expr_origin(&core.functions[0].body).clone();
        for mutation in 0..30 {
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
                2 => *type_ref = named("fake::WriterT"),
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
                    })
                }
                10 => {
                    let argument = &mut arguments[0];
                    *argument = CoreExpr::Boolean {
                        value: false,
                        origin: core_expr_origin(argument).clone(),
                    };
                }
                11 => {
                    let CoreExpr::Variable { type_ref, .. } = &mut arguments[0] else {
                        panic!()
                    };
                    *type_ref = CoreType::Hole;
                }
                12..=17 => {
                    let owner = if operation == "run" || operation == "listen" {
                        let CoreExpr::Variable { type_ref, .. } = &mut arguments[0] else {
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
                        12 => *canonical = "fake::WriterT".into(),
                        13 => arguments.clear(),
                        14 => arguments[0] = named("Maybe"),
                        15 => arguments[1] = named("Int"),
                        16 => arguments[2] = named("Maybe"),
                        17 => arguments[2] = named("String"),
                        _ => unreachable!(),
                    }
                }
                _ if operation == "run" => evidence.push(seed[0].clone()),
                18 => evidence.clear(),
                19 => {
                    evidence.pop();
                }
                20 => evidence.swap(0, 1),
                21 => evidence[0].constraint.trait_identity = Some("fake::Monad".into()),
                22 => evidence[0].constraint.arguments[0] = named("List"),
                23 => evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 99 },
                24 => evidence[1].constraint.trait_identity = Some("fake::Monoid".into()),
                25 => evidence[1].constraint.arguments[0] = named("Int"),
                26 => evidence[1].evidence = CoreInstanceEvidence::Parameter { index: 99 },
                27 => {
                    let CoreInstanceEvidence::Standard { identity, .. } = &mut evidence[0].evidence
                    else {
                        panic!()
                    };
                    *identity = "std/list::Monad".into();
                }
                28 => {
                    let CoreInstanceEvidence::Standard { identity, .. } = &mut evidence[1].evidence
                    else {
                        panic!()
                    };
                    *identity = "fake::Monoid".into();
                }
                29 => evidence[1].constraint.name = "Semigroup".into(),
                _ => unreachable!(),
            }
            let error = emit_rust_module(&bad, "main").expect_err("forged WriterT call");
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
fn writer_transformer_saved_references_keep_type_and_target_checks() {
    for operation in ["run", "tell", "listen", "lift"] {
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
                *type_ref = CoreType::Hole
            }
            other => panic!("{other:?}"),
        }
        let error = emit_rust_module(&bad, "main").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.as_ref(), Some(&origin));
        let (parameter, result) = signature(operation);
        let unused = checked(&format!(
            "import * as wt from \"std/transformer/writer\"\npub fn probe value:Int -> Int = {{ let saved:{parameter} -> {result} = wt.{operation}; value }}\npub fn main -> Int = 0\n"
        ));
        emit_rust_module(&unused, "main").unwrap();
        let error = emit_program_wasm(&[&unused], "native/writer", &["probe".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(error.message.contains("target-specific proof"), "{error}");
        assert_eq!(error.origin.unwrap().source, "writer-transformer-雪.ssrg");
    }
}

#[test]
fn writer_transformer_returned_function_requires_proven_types_and_stages() {
    let core = checked("import * as wt from \"std/transformer/writer\"\nalias Base<A> = Int -> A\npub fn probe value:wt.WriterT<String,Base<_>,Int> -> (Int,String) = wt.run value 4\npub fn main -> Int = 0");
    emit_rust_module(&core, "main").unwrap();
    let origin = core_expr_origin(&core.functions[0].body).clone();
    for mutation in 0..6 {
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
        assert_eq!(*evidence_argument_index, Some(1));
        match mutation {
            0 => *evidence_argument_index = None,
            1 => *evidence_argument_index = Some(2),
            2 => {
                arguments[1] = CoreExpr::Boolean {
                    value: false,
                    origin: core_expr_origin(&arguments[1]).clone(),
                }
            }
            3 => {
                *type_ref = CoreType::Tuple {
                    elements: vec![named("Bool"), named("String")],
                }
            }
            4 => {
                arguments.pop();
            }
            5 => arguments.push(arguments[0].clone()),
            _ => unreachable!(),
        }
        let error = emit_rust_module(&bad, "main").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.as_ref(), Some(&origin));
    }
}

#[test]
fn writer_transformer_effect_run_validates_the_complete_base_result() {
    let original = checked("import * as wt from \"std/transformer/writer\"\npub effect fn probe value:wt.WriterT<String,Effect<{},String>,Int> -> (Int,String) fails String = wt.run value\npub fn main -> Int = 0");
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
