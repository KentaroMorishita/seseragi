use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreInstanceEvidence, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "big-key-雪🙂.ssrg",
        "native/big-int-keys",
        source,
    ))
    .unwrap_or_else(|e| panic!("{source}\n{e:?}"))
    .core_ir
}

#[test]
fn big_int_keys_keep_generic_saved_methods_shadowing_and_collection_key_dictionaries() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-big-int-keys.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-big-int-key-effects.ssrg"),
    ] {
        let artifact = emit_rust_module(&checked(source), "main").unwrap();
        assert!(artifact
            .metadata
            .capabilities
            .iter()
            .any(|c| c == "big-int"));
    }
}

#[test]
fn big_int_keys_reject_counterfeit_owner_kind_method_result_arguments_and_evidence() {
    for (name, signature, result) in [
        (
            "eq",
            "left:big.BigInt -> right:big.BigInt -> Bool = eq left right",
            "Int",
        ),
        ("hash", "value:big.BigInt -> Int = hash value", "Bool"),
    ] {
        let source = format!("// 雪🙂\nimport * as big from \"std/big-int\"\nfn operation {signature}\npub fn main -> Int = 0\n");
        let core = checked(&source);
        let index = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::operation"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[index].body).clone();
        emit_rust_module(&core, "main").unwrap();
        for mutation in 0..21 {
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
            let canonical = |name: &str| CoreType::Named {
                name: format!("std/prelude::{name}"),
                arguments: vec![],
            };
            match mutation {
                0 => *type_ref = CoreType::Hole,
                1 => *type_ref = canonical(result),
                2 => arguments.clear(),
                3 => arguments.push(arguments[0].clone()),
                4 => {
                    arguments[0] = CoreExpr::Unit {
                        origin: origin.clone(),
                    }
                }
                5 => evidence.clear(),
                6 => evidence[0].constraint.name = "Counterfeit".into(),
                7 => evidence[0].constraint.trait_identity = Some("fake::Eq".into()),
                8 => evidence[0].constraint.arguments.clear(),
                9 => evidence[0].constraint.arguments[0] = canonical("Int"),
                10 => {
                    evidence[0].constraint.arguments[0] = CoreType::ExternalNamed {
                        name: "big.BigInt".into(),
                        canonical: "fake::BigInt".into(),
                        arguments: vec![],
                    }
                }
                11 => {
                    evidence[0].constraint.arguments[0] = CoreType::Named {
                        name: "std/big-int::BigInt".into(),
                        arguments: vec![canonical("Int")],
                    }
                }
                12 => {
                    let CoreInstanceEvidence::Standard { type_arguments, .. } =
                        &mut evidence[0].evidence
                    else {
                        panic!()
                    };
                    type_arguments.push(canonical("Int"));
                }
                13 => {
                    let extra = evidence[0].clone();
                    let CoreInstanceEvidence::Standard {
                        evidence_arguments, ..
                    } = &mut evidence[0].evidence
                    else {
                        panic!()
                    };
                    evidence_arguments.push(extra);
                }
                14 => evidence[0].evidence = CoreInstanceEvidence::Parameter { index: usize::MAX },
                15 => trait_dispatch.as_mut().unwrap().method = "counterfeit".into(),
                16 => trait_dispatch.as_mut().unwrap().trait_identity = "fake::Eq".into(),
                17 => deferred_evidence_parameters.push(CoreType::Hole),
                18 => deferred_evidence_type_constructor_parameters.push("F".into()),
                19 => *evidence_argument_index = Some(0),
                20 => *trait_dispatch = None,
                _ => unreachable!(),
            }
            let error = emit_rust_module(&bad, "main")
                .err()
                .unwrap_or_else(|| panic!("accepted {name}/{mutation}"));
            assert_eq!(error.code, "SES-RUST-0001");
            assert_eq!(error.origin.unwrap(), origin, "{name}/{mutation}");
        }
    }
}

#[test]
fn unused_saved_big_int_key_dictionaries_do_not_expand_program_wasm() {
    for (method, signature) in [
        ("eq", "big.BigInt -> big.BigInt -> Bool"),
        ("hash", "big.BigInt -> Int"),
    ] {
        let source = format!("import * as big from \"std/big-int\"\npub fn probe value:Int -> Int = {{ let saved:{signature} = {method}; value }}\npub fn main -> Int = 0\n");
        let core = checked(&source);
        emit_rust_module(&core, "main").unwrap();
        let error =
            emit_program_wasm(&[&core], "native/big-int-keys", &["probe".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap().source, "big-key-雪🙂.ssrg");
    }
}

#[test]
fn both_big_int_equality_operators_reject_forged_operands_results_and_provider_identity() {
    for operator in ["==", "!="] {
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
                    *identity = "std/int::Eq".into();
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
