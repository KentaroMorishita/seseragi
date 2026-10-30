use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreInstanceEvidence, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "set-operations-雪🙂.ssrg",
        "native/set",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:?}"))
    .core_ir
}
fn operations() -> [(&'static str, &'static str, &'static str, bool); 6] {
    [
        ("filter", "(String -> Bool)", "sets.Set<String>", false),
        ("union", "sets.Set<String>", "sets.Set<String>", true),
        ("intersection", "sets.Set<String>", "sets.Set<String>", true),
        ("difference", "sets.Set<String>", "sets.Set<String>", true),
        ("isSubsetOf", "sets.Set<String>", "Bool", true),
        ("toList", "", "List<String>", false),
    ]
}

#[test]
fn set_operations_keep_generic_saved_partial_and_original_checked_source_contracts() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-set-operations.ssrg"),
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/basic-set-operation-effects.ssrg"
        ),
    ] {
        let artifact = emit_rust_module(&checked(source), "main").unwrap();
        assert!(artifact
            .metadata
            .capabilities
            .iter()
            .any(|capability| capability == "set"));
    }
}

#[test]
fn all_six_set_operations_refuse_forged_contracts_at_the_original_utf8_span() {
    let selected = checked("import * as sets from \"std/set\"\nfn has values: sets.Set<String> -> Bool = sets.contains \"snow\" values\npub fn main -> Int = 0\n");
    let CoreExpr::Call {
        evidence: selected, ..
    } = &selected
        .functions
        .iter()
        .find(|function| function.symbol.ends_with("::has"))
        .unwrap()
        .body
    else {
        panic!()
    };
    for (name, first, output, needs_key) in operations() {
        let (parameter, argument) = if first.is_empty() {
            (String::new(), "")
        } else {
            (format!("first: {first} -> "), "first ")
        };
        let source = format!("// 雪🙂\nimport * as sets from \"std/set\"\nfn operation {parameter}values: sets.Set<String> -> {output} = sets.{name} {argument}values\npub fn main -> Int = 0\n");
        let core = checked(&source);
        emit_rust_module(&core, "main").unwrap();
        let index = core
            .functions
            .iter()
            .position(|function| function.symbol.ends_with("::operation"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[index].body).clone();
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
            } = &mut bad.functions[index].body
            else {
                panic!()
            };
            match mutation {
                0 => *type_ref = CoreType::Hole,
                1 => {
                    *type_ref = CoreType::ExternalNamed {
                        name: output.into(),
                        canonical: "fake::Result".into(),
                        arguments: vec![],
                    }
                }
                2 => arguments.clear(),
                3 => arguments.extend(vec![arguments[0].clone(); 3]),
                4 => {
                    arguments[0] = CoreExpr::Unit {
                        origin: core_expr_origin(&arguments[0]).clone(),
                    }
                }
                5 => {
                    let last = arguments.last_mut().unwrap();
                    *last = CoreExpr::Unit {
                        origin: core_expr_origin(last).clone(),
                    };
                }
                6 => {
                    if needs_key {
                        evidence.clear()
                    } else {
                        *evidence = selected.clone()
                    }
                }
                7 => {
                    if needs_key {
                        evidence.swap(0, 1)
                    } else {
                        evidence.push(selected[1].clone())
                    }
                }
                8 => {
                    *trait_dispatch = Some(seseragi_lowering::CoreTraitDispatch {
                        trait_identity: "std/prelude::Eq".into(),
                        method: "eq".into(),
                        method_contract: None,
                    })
                }
                9 => deferred_evidence_parameters.push(CoreType::Hole),
                10 => deferred_evidence_type_constructor_parameters.push("F".into()),
                11 => *evidence_argument_index = Some(0),
                _ => {
                    let CoreExpr::Variable { type_ref, .. } = arguments.last_mut().unwrap() else {
                        panic!()
                    };
                    let slots = match mutation {
                        12 => vec![],
                        13 => vec![CoreType::Hole, CoreType::Hole],
                        _ => vec![CoreType::Named {
                            name: "std/prelude::Bool".into(),
                            arguments: vec![],
                        }],
                    };
                    *type_ref = CoreType::Named {
                        name: if mutation == 15 {
                            "fake::Set"
                        } else {
                            "std/set::Set"
                        }
                        .into(),
                        arguments: slots,
                    };
                }
            }
            let error = emit_rust_module(&bad, "main").expect_err("unproven Set operation");
            assert_eq!(error.code, "SES-RUST-0001", "{name}/{mutation}");
            assert_eq!(error.origin.unwrap(), origin, "{name}/{mutation}");
        }
        if needs_key {
            for mutation in 0..5 {
                let mut bad = core.clone();
                let CoreExpr::Call { evidence, .. } = &mut bad.functions[index].body else {
                    panic!()
                };
                match mutation {
                    0 => evidence[0].constraint.arguments[0] = CoreType::Hole,
                    1 => {
                        evidence[1].constraint.arguments[0] = CoreType::Named {
                            name: "std/prelude::Bool".into(),
                            arguments: vec![],
                        }
                    }
                    2 => evidence[1].evidence = CoreInstanceEvidence::Parameter { index: 99 },
                    3 => {
                        let CoreInstanceEvidence::Standard { identity, .. } =
                            &mut evidence[1].evidence
                        else {
                            panic!()
                        };
                        *identity = "std/int::Hash".into();
                    }
                    _ => {
                        let extra = evidence[0].clone();
                        let CoreInstanceEvidence::Standard {
                            evidence_arguments, ..
                        } = &mut evidence[1].evidence
                        else {
                            panic!()
                        };
                        evidence_arguments.push(extra);
                    }
                }
                let error =
                    emit_rust_module(&bad, "main").expect_err("unproven selected Set dictionaries");
                assert_eq!(
                    error.origin.unwrap(),
                    origin,
                    "{name}/dictionary/{mutation}"
                );
            }
        }
    }
}

#[test]
fn all_six_unused_set_operation_references_stay_out_of_program_wasm() {
    for (name, first, output, _) in operations() {
        let parameter = if first.is_empty() {
            String::new()
        } else {
            format!("{first} -> ")
        };
        let source = format!("import * as sets from \"std/set\"\npub fn probe value: Int -> Int = {{ let saved: {parameter}sets.Set<String> -> {output} = sets.{name}; value }}\n");
        let error =
            emit_program_wasm(&[&checked(&source)], "native/set", &["probe".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(
            error
                .message
                .contains("native collection requires a target-specific ABI proof"),
            "{name}: {error}"
        );
        assert_eq!(error.origin.unwrap().source, "set-operations-雪🙂.ssrg");
    }
}
