use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{
    core_expr_origin, CoreExpr, CoreInstanceEvidence, CoreTraitDispatch, CoreType,
};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "maybe-transformer-雪.ssrg",
        "native/transformer",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:#?}"))
    .core_ir
}

fn source(operation: &str, saved: bool) -> String {
    let (parameter, result) = if operation == "run" {
        ("mt.MaybeT<Maybe,Int>", "Maybe<Maybe<Int>>")
    } else {
        ("Maybe<Int>", "mt.MaybeT<Maybe,Int>")
    };
    let probe = if saved {
        format!("pub fn probe -> ({parameter} -> {result}) = mt.{operation}")
    } else {
        format!("pub fn probe value:{parameter} -> {result} = mt.{operation} value")
    };
    format!("import * as mt from \"std/transformer/maybe\"\n{probe}\npub fn main -> Int = 0\n")
}

#[test]
fn maybe_transformer_accepts_standard_custom_generic_saved_and_nested_monads() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/maybe-transformer.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/maybe-transformer-effects.ssrg"),
    ] {
        let core = checked(source);
        let artifact = emit_rust_module(&core, "main").unwrap();
        assert!(artifact
            .metadata
            .capabilities
            .iter()
            .any(|c| c == "maybe-transformer"));
    }
}

#[test]
fn maybe_transformer_rejects_forged_call_types_dictionaries_and_source_stages_at_origin() {
    let seed = checked(&source("fromMaybe", false));
    let CoreExpr::Call { evidence: seed, .. } = &seed.functions[0].body else {
        panic!()
    };
    for operation in ["run", "fromMaybe", "lift"] {
        let core = checked(&source(operation, false));
        emit_rust_module(&core, "main").unwrap();
        let origin = core_expr_origin(&core.functions[0].body).clone();
        for mutation in 0..16 {
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
                        name: "MaybeT".into(),
                        canonical: "fake::MaybeT".into(),
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
                10 => {
                    arguments[0] = CoreExpr::Unit {
                        origin: core_expr_origin(&arguments[0]).clone(),
                    }
                }
                11 => {
                    if let CoreExpr::Variable { type_ref, .. } = &mut arguments[0] {
                        *type_ref = CoreType::Hole
                    } else {
                        panic!()
                    }
                }
                _ if operation == "run" => {
                    let CoreExpr::Variable { type_ref, .. } = &mut arguments[0] else {
                        panic!()
                    };
                    let CoreType::ExternalNamed {
                        canonical,
                        arguments,
                        ..
                    } = type_ref
                    else {
                        panic!("{type_ref:?}")
                    };
                    match mutation {
                        12 => *canonical = "fake::MaybeT".into(),
                        13 => arguments.clear(),
                        14 => {
                            arguments[0] = CoreType::Named {
                                name: "Int".into(),
                                arguments: vec![],
                            }
                        }
                        15 => {
                            arguments[1] = CoreType::Named {
                                name: "Maybe".into(),
                                arguments: vec![],
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                _ => match mutation {
                    12 => evidence.clear(),
                    13 => evidence[0].constraint.trait_identity = Some("fake::Monad".into()),
                    14 => {
                        evidence[0].constraint.arguments[0] = CoreType::Named {
                            name: "List".into(),
                            arguments: vec![],
                        }
                    }
                    15 => evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 99 },
                    _ => unreachable!(),
                },
            }
            let error = emit_rust_module(&bad, "main").expect_err("forged MaybeT call");
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
fn maybe_transformer_saved_references_keep_type_and_target_checks() {
    for operation in ["run", "fromMaybe", "lift"] {
        let core = checked(&source(operation, true));
        emit_rust_module(&core, "main").unwrap();
        let origin = core_expr_origin(&core.functions[0].body).clone();
        let mut bad = core.clone();
        let body = match &mut bad.functions[0].body {
            CoreExpr::Lambda { body, .. } => body.as_mut(),
            expr => expr,
        };
        let type_ref = match body {
            CoreExpr::Call { type_ref, .. } => {
                *type_ref = CoreType::Hole;
                None
            }
            CoreExpr::Variable { type_ref, .. } => Some(type_ref),
            other => panic!("{other:?}"),
        };
        if let Some(type_ref) = type_ref {
            let CoreType::Function { result, .. } = type_ref else {
                panic!()
            };
            *result = Box::new(CoreType::Hole);
        }
        let error = emit_rust_module(&bad, "main").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.as_ref(), Some(&origin));
        let unused = checked(&format!(
            "import * as mt from \"std/transformer/maybe\"\npub fn probe value:Int -> Int = {{ let saved:{} = mt.{operation}; value }}\n",
            if operation == "run" { "mt.MaybeT<Maybe,Int> -> Maybe<Maybe<Int>>" } else { "Maybe<Int> -> mt.MaybeT<Maybe,Int>" }
        ));
        let error =
            emit_program_wasm(&[&unused], "native/transformer", &["probe".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(
            error.message.contains("target-specific proof"),
            "{operation}: {error}"
        );
    }
}

#[test]
fn maybe_transformer_rejects_forged_standard_base_providers_and_factory_children() {
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
        assert!(emit_rust_module(&bad, "main").is_err(), "{mutation}");
    }
}
