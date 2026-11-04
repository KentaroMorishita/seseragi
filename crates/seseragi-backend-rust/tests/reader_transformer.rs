use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{
    core_expr_origin, CoreExpr, CoreInstanceEvidence, CoreTraitDispatch, CoreType,
};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "reader-transformer-雪.ssrg",
        "native/reader",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:#?}"))
    .core_ir
}

fn signature(operation: &str) -> (&'static str, &'static str, &'static str) {
    match operation {
        "run" => (
            "Int",
            "rt.ReaderT<Int,Maybe,Int> -> Maybe<Int>",
            "environment",
        ),
        "ask" => ("Unit", "rt.ReaderT<Int,Maybe,Int>", "unit"),
        "asks" => ("Int -> Int", "rt.ReaderT<Int,Maybe,Int>", "callback"),
        "local" => (
            "Int -> Int",
            "rt.ReaderT<Int,Maybe,Int> -> rt.ReaderT<Int,Maybe,Int>",
            "callback",
        ),
        "lift" => ("Maybe<Int>", "rt.ReaderT<Int,Maybe,Int>", "value"),
        _ => unreachable!(),
    }
}

fn source(operation: &str, saved: bool) -> String {
    let (parameter, result, name) = signature(operation);
    let probe = if saved {
        format!("pub fn probe -> (({parameter}) -> {result}) = rt.{operation}")
    } else if matches!(operation, "run" | "local") {
        format!("pub fn probe {name}:({parameter}) -> value:rt.ReaderT<Int,Maybe,Int> -> {} = rt.{operation} {name} value", if operation == "run" { "Maybe<Int>" } else { "rt.ReaderT<Int,Maybe,Int>" })
    } else {
        format!("pub fn probe {name}:({parameter}) -> {result} = rt.{operation} {name}")
    };
    format!("import * as rt from \"std/transformer/reader\"\n{probe}\npub fn main -> Int = 0\n")
}

#[test]
fn reader_transformer_accepts_standard_custom_generic_saved_partial_and_nested_monads() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/reader-transformer.ssrg"),
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/reader-transformer-effects.ssrg"
        ),
    ] {
        let core = checked(source);
        let artifact = emit_rust_module(&core, "main").unwrap();
        assert!(artifact
            .metadata
            .capabilities
            .iter()
            .any(|c| c == "reader-transformer"));
    }
}

#[test]
fn reader_transformer_rejects_forged_call_types_dictionaries_and_source_stages_at_origin() {
    let seed = checked(&source("lift", false));
    let CoreExpr::Call { evidence: seed, .. } = &seed.functions[0].body else {
        panic!()
    };
    for operation in ["run", "ask", "asks", "local", "lift"] {
        let core = checked(&source(operation, false));
        emit_rust_module(&core, "main").unwrap();
        let origin = core_expr_origin(&core.functions[0].body).clone();
        for mutation in 0..20 {
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
                1 => {
                    *type_ref = CoreType::Named {
                        name: "Bool".into(),
                        arguments: vec![],
                    }
                }
                2 => {
                    *type_ref = CoreType::ExternalNamed {
                        name: "ReaderT".into(),
                        canonical: "fake::ReaderT".into(),
                        arguments: vec![],
                    }
                }
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
                10 | 11 => {
                    let CoreExpr::Variable { type_ref, .. } = &mut arguments[0] else {
                        panic!()
                    };
                    *type_ref = if mutation == 10 {
                        CoreType::Hole
                    } else {
                        CoreType::Named {
                            name: "Bool".into(),
                            arguments: vec![],
                        }
                    };
                }
                12..=15 => {
                    let transformed = if operation == "run" {
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
                    } = transformed
                    else {
                        panic!("{transformed:?}")
                    };
                    match mutation {
                        12 => *canonical = "fake::ReaderT".into(),
                        13 => arguments.clear(),
                        14 => {
                            arguments[1] = CoreType::Named {
                                name: "Int".into(),
                                arguments: vec![],
                            }
                        }
                        15 => {
                            arguments[0] = CoreType::Named {
                                name: "Maybe".into(),
                                arguments: vec![],
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                _ if operation == "run" => {
                    let CoreExpr::Variable { type_ref, .. } = &mut arguments[1] else {
                        panic!()
                    };
                    let CoreType::ExternalNamed { arguments, .. } = type_ref else {
                        panic!()
                    };
                    match mutation {
                        16 => {
                            arguments[0] = CoreType::Named {
                                name: "String".into(),
                                arguments: vec![],
                            }
                        }
                        17 => {
                            arguments[1] = CoreType::Named {
                                name: "Array".into(),
                                arguments: vec![],
                            }
                        }
                        18 => {
                            arguments[2] = CoreType::Named {
                                name: "String".into(),
                                arguments: vec![],
                            }
                        }
                        19 => {
                            arguments[2] = CoreType::Named {
                                name: "Maybe".into(),
                                arguments: vec![],
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                _ => match mutation {
                    16 => evidence.clear(),
                    17 => evidence[0].constraint.trait_identity = Some("fake::Monad".into()),
                    18 => {
                        evidence[0].constraint.arguments[0] = CoreType::Named {
                            name: "List".into(),
                            arguments: vec![],
                        }
                    }
                    19 => evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 99 },
                    _ => unreachable!(),
                },
            }
            let error = emit_rust_module(&bad, "main").expect_err("forged ReaderT call");
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
fn reader_transformer_saved_references_keep_type_and_target_checks() {
    for operation in ["run", "ask", "asks", "local", "lift"] {
        let core = checked(&source(operation, true));
        emit_rust_module(&core, "main").unwrap();
        let origin = core_expr_origin(&core.functions[0].body).clone();
        let mut bad = core.clone();
        let mut body = &mut bad.functions[0].body;
        while let CoreExpr::Lambda { body: inner, .. } = body {
            body = inner;
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
        let (parameter, result, _) = signature(operation);
        let unused = checked(&format!(
            "import * as rt from \"std/transformer/reader\"\npub fn probe value:Int -> Int = {{ let saved:({parameter}) -> {result} = rt.{operation}; value }}\n"
        ));
        let error = emit_program_wasm(&[&unused], "native/reader", &["probe".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(
            error.message.contains("target-specific proof"),
            "{operation}: {error}"
        );
    }
}

#[test]
fn reader_transformer_rejects_forged_standard_base_providers_and_factory_children() {
    for operation in ["ask", "asks", "local", "lift"] {
        let core = checked(&source(operation, false));
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
            assert!(
                emit_rust_module(&bad, "main").is_err(),
                "{operation}/{mutation}"
            );
        }
    }
}
