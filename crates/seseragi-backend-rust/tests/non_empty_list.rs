use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "non-empty-雪🙂.ssrg",
        "native/non-empty",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:#?}"))
    .core_ir
}

fn cases() -> [(&'static str, &'static str, &'static str, &'static str); 7] {
    [
        (
            "singleton",
            "singleton value",
            "n.NonEmptyList<Int>",
            "Int -> n.NonEmptyList<Int>",
        ),
        (
            "cons",
            "cons value tail",
            "n.NonEmptyList<Int>",
            "Int -> List<Int> -> n.NonEmptyList<Int>",
        ),
        (
            "fromList",
            "fromList tail",
            "Maybe<n.NonEmptyList<Int>>",
            "List<Int> -> Maybe<n.NonEmptyList<Int>>",
        ),
        (
            "toList",
            "toList values",
            "List<Int>",
            "n.NonEmptyList<Int> -> List<Int>",
        ),
        ("head", "head values", "Int", "n.NonEmptyList<Int> -> Int"),
        (
            "tail",
            "tail values",
            "List<Int>",
            "n.NonEmptyList<Int> -> List<Int>",
        ),
        (
            "reduce1",
            "reduce1 step values",
            "Int",
            "(Int -> Int -> Int) -> n.NonEmptyList<Int> -> Int",
        ),
    ]
}

#[test]
fn all_non_empty_list_apis_and_instances_accept_generic_aliases_saved_and_partial_functions() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/non-empty-list.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/non-empty-list-effects.ssrg"),
    ] {
        let artifact = emit_rust_module(&checked(source), "main").unwrap();
        assert!(artifact
            .metadata
            .capabilities
            .iter()
            .any(|c| c == "non-empty-list"));
    }
    for (name, _, _, signature) in cases() {
        let source = format!(
            "import * as n from \"std/non-empty-list\"\npub fn probe -> ({signature}) = n.{name}\n"
        );
        emit_rust_module(&checked(&source), "probe").unwrap();
    }
    let source = "import * as n from \"std/non-empty-list\"\nfn prefix<A> head:A -> (List<A> -> n.NonEmptyList<A>) = n.cons head\nfn fold<A> step:(A -> A -> A) -> (n.NonEmptyList<A> -> A) = n.reduce1 step\npub fn probe -> Int = { let head:n.NonEmptyList<Int -> Int> -> Int -> Int = n.head; let reduce:(Int -> Int -> Int) -> n.NonEmptyList<Int> -> Int = n.reduce1; let make:Int -> List<Int> -> n.NonEmptyList<Int> = n.cons; let saved=make 10; let subtract=reduce (\\a:Int -> \\b:Int -> a-b); let chosen=n.reduce1 (\\left:(Int -> Int) -> \\_right:(Int -> Int) -> left) (n.singleton (\\value:Int -> value+1)); head (n.singleton chosen) (subtract (saved `[3,2])) }\n";
    emit_rust_module(&checked(source), "probe").unwrap();
    emit_rust_module(&checked("import * as n from \"std/non-empty-list\"\npub fn probe -> Int = n.head (n.singleton (\\value:Int -> value+1)) 41\n"), "probe").unwrap();
    emit_rust_module(&checked("import * as n from \"std/non-empty-list\"\npub fn probe -> Int = n.reduce1 (\\left:(Int -> Int) -> \\_right:(Int -> Int) -> left) (n.singleton (\\value:Int -> value+1)) 41\n"), "probe").unwrap();
}

#[test]
fn non_empty_list_refuses_forged_owners_kinds_payloads_dictionaries_and_call_stages_at_origin() {
    let eq = checked("pub fn eq -> Bool = 1 == 1");
    let CoreExpr::Binary {
        evidence: other, ..
    } = &eq.functions[0].body
    else {
        panic!()
    };
    for (name, expression, result, _) in cases() {
        let source = format!("import * as n from \"std/non-empty-list\"\nfn subject value:Int -> tail:List<Int> -> values:n.NonEmptyList<Int> -> step:(Int -> Int -> Int) -> {result} = n.{expression}\npub fn probe -> Int = 0\n");
        let core = checked(&source);
        emit_rust_module(&core, "probe").unwrap();
        let origin = core_expr_origin(&core.functions[0].body).clone();
        let CoreExpr::Call { arguments, .. } = &core.functions[0].body else {
            panic!()
        };
        for mutation in 0..11 + arguments.len() * 3 {
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
                    *type_ref = CoreType::ExternalNamed {
                        name: result.into(),
                        canonical: "fake::NonEmptyList".into(),
                        arguments: vec![],
                    }
                }
                2 => {
                    *type_ref = CoreType::Named {
                        name: "std/non-empty-list::NonEmptyList".into(),
                        arguments: vec![],
                    }
                }
                3 => arguments.clear(),
                4 => arguments.extend(vec![arguments[0].clone(); 3]),
                5 => evidence.push(other[0].clone()),
                6 => deferred_evidence_parameters.push(CoreType::Hole),
                7 => deferred_evidence_type_constructor_parameters.push("F".into()),
                8 => *evidence_argument_index = Some(0),
                9 => {
                    *trait_dispatch = Some(seseragi_lowering::CoreTraitDispatch {
                        trait_identity: "std/prelude::Eq".into(),
                        method: "eq".into(),
                        method_contract: None,
                    })
                }
                10 => {
                    *type_ref = CoreType::Named {
                        name: "std/prelude::Int".into(),
                        arguments: vec![CoreType::Hole],
                    }
                }
                n => {
                    let CoreExpr::Variable { type_ref, .. } = &mut arguments[(n - 11) / 3] else {
                        panic!()
                    };
                    *type_ref = match (n - 11) % 3 {
                        0 => CoreType::Hole,
                        1 => CoreType::Named {
                            name: "std/prelude::String".into(),
                            arguments: vec![],
                        },
                        _ => CoreType::ExternalNamed {
                            name: "NonEmptyList".into(),
                            canonical: "fake::NonEmptyList".into(),
                            arguments: vec![CoreType::Named {
                                name: "std/prelude::Int".into(),
                                arguments: vec![],
                            }],
                        },
                    };
                }
            }
            let error = emit_rust_module(&bad, "probe").unwrap_err();
            assert_eq!(error.code, "SES-RUST-0001", "{name}/{mutation}");
            assert_eq!(error.origin.unwrap(), origin, "{name}/{mutation}");
        }
    }
}

#[test]
fn saved_non_empty_list_references_refuse_forged_signatures_and_evidence_at_origin() {
    let eq = checked("pub fn eq -> Bool = 1 == 1");
    let CoreExpr::Binary {
        evidence: other, ..
    } = &eq.functions[0].body
    else {
        panic!()
    };
    for (name, _, _, signature) in cases() {
        let core = checked(&format!(
            "import * as n from \"std/non-empty-list\"\npub fn probe -> ({signature}) = n.{name}\n"
        ));
        let origin = core_expr_origin(&core.functions[0].body).clone();
        for mutation in 0..4 {
            let mut bad = core.clone();
            let CoreExpr::Variable {
                type_ref, evidence, ..
            } = &mut bad.functions[0].body
            else {
                panic!()
            };
            match mutation {
                0 => *type_ref = CoreType::Hole,
                1 => {
                    *type_ref = CoreType::Named {
                        name: "std/prelude::Int".into(),
                        arguments: vec![],
                    }
                }
                2 => {
                    *type_ref = CoreType::ExternalNamed {
                        name: "NonEmptyList".into(),
                        canonical: "fake::NonEmptyList".into(),
                        arguments: vec![],
                    }
                }
                _ => evidence.push(other[0].clone()),
            }
            let error = emit_rust_module(&bad, "probe").unwrap_err();
            assert_eq!(error.code, "SES-RUST-0001", "{name}/{mutation}");
            assert_eq!(error.origin.unwrap(), origin, "{name}/{mutation}");
        }
    }
}

#[test]
fn returned_non_empty_list_functions_require_the_exact_source_boundary_and_argument_types() {
    for (expression, arity) in [
        ("n.head (n.singleton (\\value:Int -> value+1)) 41", 1),
        ("n.reduce1 (\\left:(Int -> Int) -> \\_right:(Int -> Int) -> left) (n.singleton (\\value:Int -> value+1)) 41", 2),
    ] {
        let core = checked(&format!("import * as n from \"std/non-empty-list\"\npub fn probe -> Int = {expression}\n"));
        emit_rust_module(&core, "probe").unwrap();
        let origin = core_expr_origin(&core.functions[0].body).clone();
        for mutation in 0..5 {
            let mut bad = core.clone();
            let CoreExpr::Call { arguments, evidence_argument_index, type_ref, .. } = &mut bad.functions[0].body else { panic!() };
            match mutation {
                0 => *evidence_argument_index = None,
                1 => *evidence_argument_index = Some(0),
                2 => *evidence_argument_index = Some(arity+1),
                3 => {
                    let last = arguments.last_mut().unwrap();
                    *last = CoreExpr::Unit { origin: core_expr_origin(last).clone() };
                }
                _ => *type_ref = CoreType::Hole,
            }
            let error = emit_rust_module(&bad, "probe").unwrap_err();
            assert_eq!(error.code, "SES-RUST-0001", "{expression}/{mutation}");
            assert_eq!(error.origin.unwrap(), origin, "{expression}/{mutation}");
        }
    }
}

#[test]
fn native_non_empty_list_calls_and_saved_references_do_not_widen_program_wasm() {
    for (name, expression, result, signature) in cases() {
        for declaration in [
            format!("pub fn unused value:Int -> tail:List<Int> -> values:n.NonEmptyList<Int> -> step:(Int -> Int -> Int) -> {result} = n.{expression}"),
            format!("pub fn unused -> ({signature}) = n.{name}"),
        ] {
            let core = checked(&format!("import * as n from \"std/non-empty-list\"\n{declaration}\npub fn probe -> Int = 0\n"));
            emit_rust_module(&core, "probe").unwrap();
            emit_program_wasm(&[&core], "native/non-empty", &["probe".into()])
                .unwrap_or_else(|error| panic!("program WASM refused a proven operation: {error:?}"));
        }
    }
}
