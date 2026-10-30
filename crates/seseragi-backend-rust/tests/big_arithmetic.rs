use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreInstanceEvidence, CoreType};
fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "big-arithmetic-雪🙂.ssrg",
        "native/big-int-arithmetic",
        source,
    ))
    .unwrap_or_else(|e| panic!("{source}\n{e:?}"))
    .core_ir
}
#[test]
fn big_arithmetic_keeps_saved_generic_cold_and_exact_original_inputs() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-big-int-arithmetic.ssrg"),
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/basic-big-int-arithmetic-effects.ssrg"
        ),
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
fn big_arithmetic_api_rejects_unproven_owners_kinds_arguments_evidence_and_stages() {
    let other = checked("pub fn probe -> Bool = 1 == 1");
    let CoreExpr::Binary {
        evidence: other_evidence,
        ..
    } = &other.functions[0].body
    else {
        panic!()
    };
    for (result, expression) in [
        (
            "Either<big.BigIntDivisionError,big.BigInt>",
            "big.checkedDivide (big.fromInt 3) (big.fromInt (-10))",
        ),
        (
            "Either<big.BigIntDivisionError,big.BigInt>",
            "big.checkedRemainder (big.fromInt 3) (big.fromInt (-10))",
        ),
        (
            "Either<big.BigIntPowerError,big.BigInt>",
            "big.checkedPower 3 (big.fromInt (-2))",
        ),
        ("big.BigInt", "big.abs (big.fromInt (-42))"),
        ("Int", "big.sign (big.fromInt (-42))"),
        ("big.BigIntPowerError", "big.NegativeBigIntExponent (-2)"),
    ] {
        let core = checked(&format!("// 雪🙂\nimport * as big from \"std/big-int\"\nfn probe -> {result} = {expression}\npub fn main -> Int = 0\n"));
        let index = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::probe"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[index].body).clone();
        emit_rust_module(&core, "main").unwrap();
        for mutation in 0..12 {
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
                        canonical: "fake::Result".into(),
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
                4 => arguments.push(arguments[0].clone()),
                5 => evidence.push(other_evidence[0].clone()),
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
                    arguments[0] = CoreExpr::Boolean {
                        value: true,
                        origin: origin.clone(),
                    }
                }
                _ => match type_ref {
                    CoreType::Named { name, .. } => *name = "fake::Result".into(),
                    CoreType::ExternalNamed { canonical, .. } => *canonical = "fake::Result".into(),
                    _ => panic!(),
                },
            }
            let error = emit_rust_module(&bad, "main").expect_err("forged Int text call");
            assert_eq!(error.code, "SES-RUST-0001", "{expression}/{mutation}");
            assert_eq!(error.origin.unwrap(), origin, "{expression}/{mutation}");
        }
    }
}

#[test]
fn big_arithmetic_named_methods_reject_counterfeit_method_owner_types_children_and_source_stages() {
    for (trait_name, method) in [
        ("Add", "add"),
        ("Sub", "sub"),
        ("Mul", "mul"),
        ("Div", "div"),
        ("Rem", "rem"),
        ("Pow", "pow"),
    ] {
        let right = if trait_name == "Pow" {
            "Int"
        } else {
            "big.BigInt"
        };
        let source=format!("// 雪🙂\nimport * as big from \"std/big-int\"\nfn operation left:big.BigInt -> right:{right} -> big.BigInt = {method} left right\npub fn main -> Int = 0\n");
        let core = checked(&source);
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
}
#[test]
fn big_constant_dictionaries_rejects_counterfeit_owner_kind_method_result_arguments_and_evidence() {
    for (name, signature, result) in [
        ("zero", "_unit:Unit -> big.BigInt = zero ()", "Int"),
        ("one", "_unit:Unit -> big.BigInt = one ()", "Int"),
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
                    arguments[0] = CoreExpr::Boolean {
                        value: true,
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
fn big_arithmetic_binary_operators_reject_forged_operands_results_and_provider_identity() {
    for operator in ["+", "-", "*", "/", "%", "**"] {
        let right = if operator == "**" {
            "Int"
        } else {
            "big.BigInt"
        };
        let source=format!("// 雪🙂\nimport * as big from \"std/big-int\"\nfn operation left:big.BigInt -> right:{right} -> big.BigInt = left {operator} right\npub fn main -> Int = 0\n");
        let core = checked(&source);
        let index = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::operation"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[index].body).clone();
        emit_rust_module(&core, "main").unwrap();
        for mutation in 0..16 {
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
                    *identity = "std/int::Add".into();
                }
                8 => evidence[0].constraint.name = "Counterfeit".into(),
                9 => evidence[0].constraint.trait_identity = Some("fake::Add".into()),
                10 => evidence[0].constraint.arguments.clear(),
                11 => evidence[0].constraint.arguments[1] = CoreType::Hole,
                12 => evidence[0].constraint.arguments[2] = CoreType::Hole,
                13 => evidence.push(evidence[0].clone()),
                14 => evidence[0].evidence = CoreInstanceEvidence::Parameter { index: usize::MAX },
                15 => {
                    let extra = evidence[0].clone();
                    let CoreInstanceEvidence::Standard {
                        evidence_arguments, ..
                    } = &mut evidence[0].evidence
                    else {
                        panic!()
                    };
                    evidence_arguments.push(extra);
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

#[test]
fn unused_big_arithmetic_operations_and_constants_remain_outside_program_wasm() {
    for (signature, operation) in [
        (
            "big.BigInt -> big.BigInt -> Either<big.BigIntDivisionError,big.BigInt>",
            "big.checkedDivide",
        ),
        (
            "big.BigInt -> big.BigInt -> Either<big.BigIntDivisionError,big.BigInt>",
            "big.checkedRemainder",
        ),
        (
            "Int -> big.BigInt -> Either<big.BigIntPowerError,big.BigInt>",
            "big.checkedPower",
        ),
        ("big.BigInt -> big.BigInt", "big.abs"),
        ("big.BigInt -> Int", "big.sign"),
        ("big.BigInt -> big.BigInt -> big.BigInt", "(+)"),
        ("big.BigInt -> big.BigInt -> big.BigInt", "(-)"),
        ("big.BigInt -> big.BigInt -> big.BigInt", "(*)"),
        ("big.BigInt -> big.BigInt -> big.BigInt", "(/)"),
        ("big.BigInt -> big.BigInt -> big.BigInt", "(%)"),
        ("big.BigInt -> Int -> big.BigInt", "(**)"),
        ("Unit -> big.BigInt", "zero"),
        ("Unit -> big.BigInt", "one"),
        ("big.BigIntDivisionError", "big.BigIntDivisionByZero"),
        ("Int -> big.BigIntPowerError", "big.NegativeBigIntExponent"),
    ] {
        let source=format!("import * as big from \"std/big-int\"\npub fn probe value:Int -> Int = {{ let saved:{signature} = {operation}; value }}\npub fn main -> Int = 0\n");
        let core = checked(&source);
        emit_rust_module(&core, "main").unwrap();
        let error = emit_program_wasm(&[&core], "native/big-int-arithmetic", &["probe".into()])
            .unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap().source, "big-arithmetic-雪🙂.ssrg");
    }
}
