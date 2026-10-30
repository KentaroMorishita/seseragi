use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreInstanceEvidence, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "big-int-ord-雪🙂.ssrg",
        "native/big-int-ord",
        source,
    ))
    .unwrap_or_else(|e| panic!("{source}\n{e:?}"))
    .core_ir
}

#[test]
fn big_int_ord_keeps_saved_generic_stages_original_zero_input_and_inherited_eq() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-big-int-keys.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-big-int-key-effects.ssrg"),
    ] {
        emit_rust_module(&checked(source), "main").unwrap();
    }
}

#[test]
fn big_int_ord_rejects_counterfeit_method_owner_types_children_and_source_stages() {
    let source="// 雪🙂\nimport * as big from \"std/big-int\"\nfn operation left:big.BigInt -> right:big.BigInt -> Ordering = compare left right\npub fn main -> Int = 0\n";
    let core = checked(source);
    let index = core
        .functions
        .iter()
        .position(|f| f.symbol.ends_with("::operation"))
        .unwrap();
    let origin = core_expr_origin(&core.functions[index].body).clone();
    emit_rust_module(&core, "main").unwrap();
    for mutation in 0..22 {
        let mut bad = core.clone();
        let CoreExpr::Call {
            arguments,
            evidence,
            trait_dispatch,
            type_ref,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            ..
        } = &mut bad.functions[index].body
        else {
            panic!()
        };
        let named = |name: &str| CoreType::Named {
            name: format!("std/prelude::{name}"),
            arguments: vec![],
        };
        match mutation {
            0 => *type_ref = CoreType::Hole,
            1 => *type_ref = named("Bool"),
            2 => {
                *type_ref = CoreType::ExternalNamed {
                    name: "Ordering".into(),
                    canonical: "fake::Ordering".into(),
                    arguments: vec![],
                }
            }
            3 => arguments.clear(),
            4 => arguments.push(arguments[0].clone()),
            5 => {
                arguments[0] = CoreExpr::Unit {
                    origin: origin.clone(),
                }
            }
            6 => evidence.clear(),
            7 => evidence[0].constraint.name = "Fake".into(),
            8 => evidence[0].constraint.trait_identity = Some("fake::Ord".into()),
            9 => evidence[0].constraint.arguments.clear(),
            10 => evidence[0].constraint.arguments[0] = named("Int"),
            11 => {
                evidence[0].constraint.arguments[0] = CoreType::ExternalNamed {
                    name: "big.BigInt".into(),
                    canonical: "fake::BigInt".into(),
                    arguments: vec![],
                }
            }
            12 => {
                evidence[0].constraint.arguments[0] = CoreType::Named {
                    name: "std/big-int::BigInt".into(),
                    arguments: vec![named("Int")],
                }
            }
            13 => {
                let CoreInstanceEvidence::Standard { type_arguments, .. } =
                    &mut evidence[0].evidence
                else {
                    panic!()
                };
                type_arguments.push(named("Int"));
            }
            14 => {
                let extra = evidence[0].clone();
                let CoreInstanceEvidence::Standard {
                    evidence_arguments, ..
                } = &mut evidence[0].evidence
                else {
                    panic!()
                };
                evidence_arguments.push(extra);
            }
            15 => trait_dispatch.as_mut().unwrap().trait_identity = "fake::Ord".into(),
            16 => trait_dispatch.as_mut().unwrap().method = "eq".into(),
            17 => *trait_dispatch = None,
            18 => deferred_evidence_parameters.push(CoreType::Hole),
            19 => deferred_evidence_type_constructor_parameters.push("F".into()),
            20 => *evidence_argument_index = Some(0),
            21 => evidence[0].evidence = CoreInstanceEvidence::Parameter { index: usize::MAX },
            _ => unreachable!(),
        }
        let error = emit_rust_module(&bad, "main")
            .err()
            .unwrap_or_else(|| panic!("accepted Ord mutation{mutation}"));
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap(), origin, "mutation{mutation}");
    }
}

#[test]
fn unused_big_int_compare_and_operator_references_stay_out_of_program_wasm() {
    for (method, result) in [
        ("compare", "Ordering"),
        ("(<)", "Bool"),
        ("(<=)", "Bool"),
        ("(>)", "Bool"),
        ("(>=)", "Bool"),
    ] {
        let source=format!("import * as big from \"std/big-int\"\npub fn probe value:Int -> Int = {{ let saved:big.BigInt -> big.BigInt -> {result} = {method}; value }}\npub fn main -> Int = 0\n");
        let core = checked(&source);
        emit_rust_module(&core, "main").unwrap();
        let error = emit_program_wasm(&[&core], "native/big-int-ord", &["probe".into()])
            .err()
            .unwrap_or_else(|| panic!("admitted unused BigInt {method}"));
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap().source, "big-int-ord-雪🙂.ssrg");
    }
}

#[test]
fn all_big_int_comparison_operators_reject_forged_operands_results_and_provider_identity() {
    for operator in ["<", "<=", ">", ">="] {
        let source=format!("// 雪🙂\nimport * as big from \"std/big-int\"\nfn operation left:big.BigInt -> right:big.BigInt -> Bool = left {operator} right\npub fn main -> Int = 0\n");
        let core = checked(&source);
        let index = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::operation"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[index].body).clone();
        emit_rust_module(&core, "main").unwrap();
        for mutation in 0..8 {
            let mut bad = core.clone();
            let CoreExpr::Binary {
                left,
                right,
                evidence,
                type_ref,
                ..
            } = &mut bad.functions[index].body
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
                    **left = CoreExpr::Unit {
                        origin: origin.clone(),
                    }
                }
                3 => {
                    **right = CoreExpr::Unit {
                        origin: origin.clone(),
                    }
                }
                4 => evidence.clear(),
                5 => {
                    evidence[0].constraint.arguments[0] = CoreType::Named {
                        name: "std/prelude::Int".into(),
                        arguments: vec![],
                    }
                }
                6 => {
                    let CoreInstanceEvidence::Standard { type_arguments, .. } =
                        &mut evidence[0].evidence
                    else {
                        panic!()
                    };
                    type_arguments.push(CoreType::Hole);
                }
                7 => {
                    let CoreInstanceEvidence::Standard { identity, .. } = &mut evidence[0].evidence
                    else {
                        panic!()
                    };
                    *identity = "std/int::Ord".into();
                }
                _ => unreachable!(),
            }
            let error = emit_rust_module(&bad, "main")
                .err()
                .unwrap_or_else(|| panic!("accepted {operator}/{mutation}"));
            assert_eq!(error.code, "SES-RUST-0001");
            assert_eq!(error.origin.unwrap(), origin, "{operator}/{mutation}");
        }
    }
}
