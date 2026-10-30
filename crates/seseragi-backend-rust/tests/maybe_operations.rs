use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreInstanceEvidence, CoreType};

#[test]
fn maybe_operations_keep_original_fallbacks_scoped_traversal_and_returned_callables() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-maybe-operations.ssrg"),
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/basic-maybe-operations-effects.ssrg"
        ),
    ] {
        let core =
            compile_core_module(CompileInput::new("maybe-雪🙂.ssrg", "native/maybe", source))
                .unwrap_or_else(|error| panic!("{error:#?}"))
                .core_ir;
        let artifact = emit_rust_module(&core, "main").unwrap();
        assert!(artifact
            .metadata
            .capabilities
            .iter()
            .any(|c| c == "maybe-operations"));
    }
}

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new("maybe-雪🙂.ssrg", "native/maybe", source))
        .unwrap_or_else(|error| panic!("{source}\n{error:#?}"))
        .core_ir
}
fn cases() -> [(&'static str, &'static str, &'static str, &'static str); 4] {
    [
        (
            "withDefault",
            "withDefault 7 (Just 8)",
            "Int",
            "Int -> Maybe<Int> -> Int",
        ),
        (
            "orElse",
            "orElse (Just 7) (Just 8)",
            "Maybe<Int>",
            "Maybe<Int> -> Maybe<Int> -> Maybe<Int>",
        ),
        (
            "sequence",
            "sequence [Just 7,Just 8]",
            "Maybe<Array<Int>>",
            "Array<Maybe<Int>> -> Maybe<Array<Int>>",
        ),
        (
            "traverse",
            "traverse (\\value:Int -> Just (value + 1)) [7,8]",
            "Maybe<Array<Int>>",
            "(Int -> Maybe<Int>) -> Array<Int> -> Maybe<Array<Int>>",
        ),
    ]
}

#[test]
fn maybe_operations_refuse_forged_owners_kinds_payloads_dictionaries_and_source_stages_at_origin() {
    let eq = checked("pub fn equal -> Bool = 1 == 1");
    let CoreExpr::Binary {
        evidence: other, ..
    } = &eq.functions[0].body
    else {
        panic!()
    };
    for (name, expression, result, _) in cases() {
        let core = checked(&format!(
            "import * as maybe from \"std/maybe\"\npub fn probe -> {result} = maybe.{expression}\n"
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
fn maybe_saved_references_keep_the_full_canonical_curried_signature_and_unused_wasm_refusal() {
    for (name, _, _, signature) in cases() {
        let source=format!("import * as maybe from \"std/maybe\"\nfn saved -> ({signature}) = maybe.{name}\npub fn probe -> Int = 0\n");
        let core = checked(&source);
        emit_rust_module(&core, "probe").unwrap();
        let error = emit_program_wasm(&[&core], "native/maybe", &["probe".into()]).unwrap_err();
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
fn canonical_traversable_parent_and_rank_two_method_contract_are_checked() {
    let core = checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-maybe-operations.ssrg"
    ));
    let index = core
        .instances
        .iter()
        .position(|i| i.identity.starts_with("std/prelude::Traversable<"))
        .unwrap();
    for mutation in 0..9 {
        let mut bad = core.clone();
        let instance = &mut bad.instances[index];
        let seseragi_lowering::CoreInstanceImplementation::UserDefined { methods } =
            &mut instance.implementation
        else {
            panic!()
        };
        match mutation {
            0 => instance.supertrait_count = 0,
            1 => instance.constraints[0].trait_identity = Some("fake::Functor".into()),
            2 => {
                instance.constraints[0].arguments[0] = CoreType::Named {
                    name: "std/prelude::List".into(),
                    arguments: vec![],
                }
            }
            3 => methods[0].type_parameters[0].arity = 0,
            4 => methods[0].constraints[0].trait_identity = Some("fake::Applicative".into()),
            5 => {
                methods[0].constraints[0].arguments[0] = CoreType::Named {
                    name: "std/prelude::Array".into(),
                    arguments: vec![],
                }
            }
            6 => methods[0].parameters[0].type_ref = CoreType::Hole,
            7 => methods[0].constraints.clear(),
            8 => methods.push(methods[0].clone()),
            _ => unreachable!(),
        }
        assert!(
            emit_rust_module(&bad, "main").is_err(),
            "mutation {mutation}"
        );
    }
}

#[test]
fn returned_standard_functions_require_the_original_dictionary_boundary_and_late_argument_type() {
    for (expression, boundary) in [
        ("maybe.withDefault (\\left:Int -> \\right:Int -> left + right) (Just (\\left:Int -> \\right:Int -> left * right)) 6 7", 2),
        ("control.reduceUntil (\\_:Unit -> 40) (\\operation:(Unit -> Int) -> \\value:Int -> control.Next (\\_:Unit -> operation () + value)) [1,2] ()", 3),
    ] {
        let source=format!("import * as maybe from \"std/maybe\"\nimport * as control from \"std/collection\"\npub fn probe -> Int = {expression}\n");
        let core=checked(&source);
        emit_rust_module(&core,"probe").unwrap();
        let index=core.functions.iter().position(|f|f.symbol.ends_with("::probe")).unwrap();
        for selected in [None,Some(0),Some(1),Some(boundary+1)] {
            let mut bad=core.clone();
            let CoreExpr::Call {evidence_argument_index,..}=&mut bad.functions[index].body else {panic!()};
            assert_eq!(*evidence_argument_index,Some(boundary));
            *evidence_argument_index=selected;
            assert!(emit_rust_module(&bad,"probe").is_err(),"{expression}/{selected:?}");
        }
        let mut bad=core.clone();
        let CoreExpr::Call {arguments,..}=&mut bad.functions[index].body else {panic!()};
        let last=arguments.last_mut().unwrap();
        *last=CoreExpr::Boolean {value:true,origin:core_expr_origin(last).clone()};
        assert!(emit_rust_module(&bad,"probe").is_err(),"{expression}/late argument");
    }
}
