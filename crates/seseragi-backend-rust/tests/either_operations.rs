use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreInstanceEvidence, CoreType};

#[test]
fn either_operations_keep_original_selected_branches_and_scoped_traversal() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-either-operations.ssrg"),
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/basic-either-operations-effects.ssrg"
        ),
    ] {
        let core = compile_core_module(CompileInput::new(
            "either-雪🙂.ssrg",
            "native/either",
            source,
        ))
        .unwrap_or_else(|error| panic!("{error:#?}"))
        .core_ir;
        let artifact = emit_rust_module(&core, "main").unwrap();
        assert!(artifact
            .metadata
            .capabilities
            .iter()
            .any(|c| c == "either-operations"));
    }
}

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "either-雪🙂.ssrg",
        "native/either",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:#?}"))
    .core_ir
}
fn cases() -> [(&'static str, &'static str, &'static str, &'static str); 7] {
    [
        (
            "mapLeft",
            "mapLeft (\\value:Int -> show value) (Left 7)",
            "Either<String,Bool>",
            "(Int -> String) -> Either<Int,Bool> -> Either<String,Bool>",
        ),
        (
            "mapRight",
            "mapRight (\\value:Int -> show value) (Right 7)",
            "Either<Bool,String>",
            "(Int -> String) -> Either<Bool,Int> -> Either<Bool,String>",
        ),
        (
            "bimap",
            "bimap (\\value:Int -> show value) (\\value:Int -> value + 1) (Left 7)",
            "Either<String,Int>",
            "(Int -> String) -> (Int -> Int) -> Either<Int,Int> -> Either<String,Int>",
        ),
        (
            "fold",
            "fold (\\value:Int -> value + 1) (\\value:Bool -> if value then 7 else 8) (Left 7)",
            "Int",
            "(Int -> Int) -> (Bool -> Int) -> Either<Int,Bool> -> Int",
        ),
        (
            "swap",
            "swap (Left 7)",
            "Either<Bool,Int>",
            "Either<Int,Bool> -> Either<Bool,Int>",
        ),
        (
            "sequence",
            "sequence [Right 7,Right 8]",
            "Either<String,Array<Int>>",
            "Array<Either<String,Int>> -> Either<String,Array<Int>>",
        ),
        (
            "traverse",
            "traverse (\\value:Int -> Right (value + 1)) [7,8]",
            "Either<String,Array<Int>>",
            "(Int -> Either<String,Int>) -> Array<Int> -> Either<String,Array<Int>>",
        ),
    ]
}

#[test]
fn either_operations_refuse_forged_owners_kinds_payloads_dictionaries_and_source_stages_at_origin()
{
    let eq = checked("pub fn equal -> Bool = 1 == 1");
    let CoreExpr::Binary {
        evidence: other, ..
    } = &eq.functions[0].body
    else {
        panic!()
    };
    for (name, expression, result, _) in cases() {
        let core = checked(&format!(
            "import * as either from \"std/either\"\npub fn probe -> {result} = either.{expression}\n"
        ));
        emit_rust_module(&core, "probe").unwrap();
        let index = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::probe"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[index].body).clone();
        let CoreExpr::Call {
            arguments,
            evidence,
            ..
        } = &core.functions[index].body
        else {
            panic!()
        };
        let count = arguments.len();
        let extras = if evidence.is_empty() { 0 } else { 9 };
        for mutation in 0..10 + count + extras {
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
                panic!()
            };
            match mutation {
                0 => *type_ref = CoreType::Hole,
                1 => {
                    *type_ref = CoreType::ExternalNamed {
                        name: result.into(),
                        canonical: "fake::Payload".into(),
                        arguments: vec![],
                    }
                }
                2 => {
                    *type_ref = CoreType::Named {
                        name: "std/prelude::Int".into(),
                        arguments: vec![CoreType::Hole],
                    }
                }
                3 => arguments.clear(),
                4 => arguments.extend(vec![arguments[0].clone(); 4]),
                5 => {
                    if evidence.is_empty() {
                        evidence.push(other[0].clone())
                    } else {
                        evidence.clear()
                    }
                }
                6 => deferred_evidence_parameters.push(CoreType::Hole),
                7 => deferred_evidence_type_constructor_parameters.push("F".into()),
                8 => *evidence_argument_index = Some(0),
                9 => {
                    *trait_dispatch = Some(seseragi_lowering::CoreTraitDispatch {
                        trait_identity: "std/prelude::Traversable".into(),
                        method: "traverse".into(),
                        method_contract: None,
                    })
                }
                n if n < 10 + count => {
                    let i = n - 10;
                    arguments[i] = CoreExpr::Boolean {
                        value: true,
                        origin: core_expr_origin(&arguments[i]).clone(),
                    };
                }
                n => {
                    let selected = &mut evidence[0];
                    match n - 10 - count {
                        0 => selected.constraint.trait_identity = Some("fake::Traversable".into()),
                        1 => selected.constraint.name = "Functor".into(),
                        2 => selected.constraint.arguments.clear(),
                        3 => {
                            selected.constraint.arguments[0] = CoreType::Named {
                                name: "std/prelude::Int".into(),
                                arguments: vec![],
                            }
                        }
                        4 => selected.evidence = CoreInstanceEvidence::Parameter { index: 0 },
                        which => {
                            let CoreInstanceEvidence::Standard {
                                identity,
                                type_arguments,
                                evidence_arguments,
                            } = &mut selected.evidence
                            else {
                                panic!()
                            };
                            match which {
                                5 => *identity = "std/list::Traversable".into(),
                                6 => type_arguments.push(CoreType::Hole),
                                7 => evidence_arguments.push(other[0].clone()),
                                8 => *identity = "std/non-empty-list::Traversable".into(),
                                _ => unreachable!(),
                            }
                        }
                    }
                }
            }
            let Err(error) = emit_rust_module(&bad, "probe") else {
                panic!("{name}/{mutation} was accepted");
            };
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
fn either_saved_references_keep_the_full_canonical_curried_signature_and_unused_wasm_refusal() {
    for (name, _, _, signature) in cases() {
        let source=format!("import * as either from \"std/either\"\nfn saved -> ({signature}) = either.{name}\npub fn probe -> Int = 0\n");
        let core = checked(&source);
        emit_rust_module(&core, "probe").unwrap();
        let error = emit_program_wasm(&[&core], "native/either", &["probe".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        let index = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::saved"))
            .unwrap();
        for invalid in [
            CoreType::Hole,
            CoreType::Named {
                name: "std/prelude::Bool".into(),
                arguments: vec![],
            },
            CoreType::Function {
                parameter: Box::new(CoreType::Named {
                    name: "std/prelude::Bool".into(),
                    arguments: vec![],
                }),
                result: Box::new(CoreType::Hole),
            },
        ] {
            let mut bad = core.clone();
            let mut body = &mut bad.functions[index].body;
            while let CoreExpr::Lambda { body: inner, .. } = body {
                body = inner;
            }
            match body {
                CoreExpr::Variable { type_ref, .. } | CoreExpr::Call { type_ref, .. } => {
                    *type_ref = invalid
                }
                expr => panic!("{expr:?}"),
            }
            assert!(emit_rust_module(&bad, "probe").is_err(), "{name}");
        }
    }
}

#[test]
fn either_fold_returned_functions_keep_the_original_source_boundary_and_late_argument_type() {
    let core=checked("import * as either from \"std/either\"\npub fn probe -> Int = either.fold (\\error:String -> \\left:Int -> \\right:Int -> left + right) (\\n:Int -> \\left:Int -> \\right:Int -> n + left + right) (Right 7) 10 25\n");
    emit_rust_module(&core, "probe").unwrap();
    let index = core
        .functions
        .iter()
        .position(|f| f.symbol.ends_with("::probe"))
        .unwrap();
    for selected in [None, Some(0), Some(2), Some(4)] {
        let mut bad = core.clone();
        let CoreExpr::Call {
            evidence_argument_index,
            ..
        } = &mut bad.functions[index].body
        else {
            panic!()
        };
        assert_eq!(*evidence_argument_index, Some(3));
        *evidence_argument_index = selected;
        assert!(emit_rust_module(&bad, "probe").is_err(), "{selected:?}");
    }
    let mut bad = core.clone();
    let CoreExpr::Call { arguments, .. } = &mut bad.functions[index].body else {
        panic!()
    };
    let last = arguments.last_mut().unwrap();
    *last = CoreExpr::Boolean {
        value: true,
        origin: core_expr_origin(last).clone(),
    };
    assert!(emit_rust_module(&bad, "probe").is_err());
}
