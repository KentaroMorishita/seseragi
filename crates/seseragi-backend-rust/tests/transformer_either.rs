use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreInstanceEvidence, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "either-transformer-雪🙂.ssrg",
        "native/either-transformer",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:#?}"))
    .core_ir
}

#[test]
fn either_transformer_apis_accept_saved_generic_and_nested_base_dictionaries() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-transformer-either.ssrg"),
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/basic-transformer-either-effects.ssrg"
        ),
    ] {
        let core = checked(source);
        let artifact = emit_rust_module(&core, "main").unwrap();
        assert!(artifact
            .metadata
            .capabilities
            .iter()
            .any(|capability| capability == "transformer-either"));
    }
}

#[test]
fn either_transformer_run_accepts_type_function_bases_without_a_monad_dictionary() {
    let mut core = checked(
        "import * as et from \"std/transformer/either\"\n\
        fn run value: et.EitherT<String, Maybe, Int> -> Maybe<Either<String, Int>> = et.run value\n\
        pub fn probe -> Int = 0",
    );
    let function = core
        .functions
        .iter_mut()
        .find(|function| function.symbol.ends_with("::run"))
        .unwrap();
    let CoreExpr::Call { arguments, .. } = &mut function.body else {
        panic!()
    };
    let CoreExpr::Variable { type_ref, .. } = &mut arguments[0] else {
        panic!()
    };
    for ty in [&mut function.parameters[0].type_ref, type_ref] {
        let CoreType::ExternalNamed { arguments, .. } = ty else {
            panic!()
        };
        arguments[1] = CoreType::TypeFunction {
            parameter_kinds: vec![0],
            body: Box::new(CoreType::ExternalNamed {
                name: "Maybe".into(),
                canonical: "std/prelude::Maybe".into(),
                arguments: vec![CoreType::TypeFunctionParameter { depth: 0, index: 0 }],
            }),
        };
    }
    let artifact = emit_rust_module(&core, "probe").unwrap();
    assert!(artifact
        .source
        .contains("native_either_transformer_operation"));
    assert!(!artifact
        .metadata
        .capabilities
        .iter()
        .any(|capability| capability == "monad"));
}

fn cases() -> [(&'static str, &'static str, &'static str); 3] {
    [
        ("run", "et.run (pure 7)", "Maybe<Either<String, Int>>"),
        (
            "fromEither",
            "et.fromEither (Right 7)",
            "et.EitherT<String, Maybe, Int>",
        ),
        ("lift", "et.lift (Just 7)", "et.EitherT<String, Maybe, Int>"),
    ]
}

#[test]
fn either_transformer_apis_refuse_forged_types_dictionaries_and_stages_at_origin() {
    for (name, expression, result) in cases() {
        let core = checked(&format!(
            "import * as et from \"std/transformer/either\"\npub fn probe -> {result} = {expression}"
        ));
        emit_rust_module(&core, "probe").unwrap();
        let index = core
            .functions
            .iter()
            .position(|function| function.symbol.ends_with("::probe"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[index].body).clone();
        for mutation in 0..if name == "run" { 10 } else { 15 } {
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
            } = &mut bad.functions[index].body
            else {
                panic!("expected direct {name} call")
            };
            match mutation {
                0 => *type_ref = CoreType::Hole,
                1 => {
                    *type_ref = CoreType::ExternalNamed {
                        name: "EitherT".into(),
                        canonical: "fake::EitherT".into(),
                        arguments: vec![],
                    }
                }
                2 => {
                    arguments[0] = CoreExpr::Boolean {
                        value: false,
                        origin: core_expr_origin(&arguments[0]).clone(),
                    }
                }
                3 => arguments.clear(),
                4 => arguments.push(arguments[0].clone()),
                5 => evidence.clear(),
                6 => deferred_evidence_parameters.push(CoreType::Hole),
                7 => deferred_evidence_type_constructor_parameters.push("M".into()),
                8 => *evidence_argument_index = Some(0),
                9 => {
                    *trait_dispatch = Some(seseragi_lowering::CoreTraitDispatch {
                        trait_identity: "std/prelude::Monad".into(),
                        method: "pure".into(),
                        method_contract: None,
                    })
                }
                10 => evidence[0].constraint.name = "Functor".into(),
                11 => evidence[0].constraint.trait_identity = Some("fake::Monad".into()),
                12 => {
                    evidence[0].constraint.arguments[0] = CoreType::Named {
                        name: "std/prelude::Array".into(),
                        arguments: vec![],
                    }
                }
                13 => evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 0 },
                14 => {
                    let CoreInstanceEvidence::Standard { identity, .. } = &mut evidence[0].evidence
                    else {
                        panic!()
                    };
                    *identity = "std/array::Monad".into();
                }
                _ => unreachable!(),
            }
            if mutation == 5 && name == "run" {
                let CoreExpr::Call { evidence, .. } = &mut bad.functions[index].body else {
                    unreachable!()
                };
                evidence.push(seseragi_lowering::CoreCallEvidence {
                    constraint: seseragi_lowering::CoreInstanceConstraint {
                        name: "Monad".into(),
                        trait_identity: Some("std/prelude::Monad".into()),
                        arguments: vec![CoreType::Named {
                            name: "std/prelude::Maybe".into(),
                            arguments: vec![],
                        }],
                    },
                    evidence: CoreInstanceEvidence::Parameter { index: 0 },
                });
            }
            let error = emit_rust_module(&bad, "probe").unwrap_err();
            assert_eq!(error.code, "SES-RUST-0001", "{name}/{mutation}: {error:?}");
            assert_eq!(
                error.origin.as_ref(),
                Some(&origin),
                "{name}/{mutation}: {error:?}"
            );
        }
    }
}

#[test]
fn either_transformer_saved_apis_check_signatures_and_keep_unused_program_wasm_refusal() {
    for (name, parameter, result) in [
        (
            "run",
            "et.EitherT<String, Maybe, Int>",
            "Maybe<Either<String, Int>>",
        ),
        (
            "fromEither",
            "Either<String, Int>",
            "et.EitherT<String, Maybe, Int>",
        ),
        ("lift", "Maybe<Int>", "et.EitherT<String, Maybe, Int>"),
    ] {
        let core = checked(&format!(
            "import * as et from \"std/transformer/either\"\nfn saved -> ({parameter} -> {result}) = et.{name}\npub fn probe -> Int = 0"
        ));
        emit_rust_module(&core, "probe").unwrap();
        let error = emit_program_wasm(&[&core], "native/either-transformer", &["probe".into()])
            .unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        let mut bad = core.clone();
        let saved = bad
            .functions
            .iter_mut()
            .find(|function| function.symbol.ends_with("::saved"))
            .unwrap();
        let mut expression = &mut saved.body;
        while let CoreExpr::Lambda { body, .. } = expression {
            expression = body;
        }
        let origin = core_expr_origin(expression).clone();
        match expression {
            CoreExpr::Variable { type_ref, .. } | CoreExpr::Call { type_ref, .. } => {
                *type_ref = CoreType::Hole
            }
            _ => panic!("expected saved {name} reference"),
        }
        let error = emit_rust_module(&bad, "probe").unwrap_err();
        assert_eq!(error.origin.as_ref(), Some(&origin));
    }
}
