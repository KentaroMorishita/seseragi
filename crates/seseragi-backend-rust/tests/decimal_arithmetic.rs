use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreInstanceEvidence, CoreType};
fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "decimal-arithmetic-雪🙂.ssrg",
        "native/decimal-arithmetic",
        source,
    ))
    .unwrap_or_else(|e| panic!("{source}\n{e:?}"))
    .core_ir
}
#[test]
fn decimal_arithmetic_keeps_saved_generic_cold_and_exact_original_inputs() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-decimal-arithmetic.ssrg"),
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/basic-decimal-arithmetic-effects.ssrg"
        ),
    ] {
        let artifact = emit_rust_module(&checked(source), "main").unwrap();
        assert!(artifact
            .metadata
            .capabilities
            .iter()
            .any(|c| c == "decimal"));
    }
}
#[test]
fn decimal_arithmetic_named_methods_reject_counterfeit_method_owner_types_children_and_source_stages(
) {
    for (trait_name, method) in [("Add", "add"), ("Sub", "sub"), ("Mul", "mul")] {
        let right = if trait_name == "Pow" {
            "Int"
        } else {
            "decimal.Decimal"
        };
        let source=format!("// 雪🙂\nimport * as decimal from \"std/decimal\"\nfn operation left:decimal.Decimal -> right:{right} -> decimal.Decimal = {method} left right\npub fn main -> Int = 0\n");
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
                        name: "decimal.Decimal".into(),
                        canonical: "fake::Decimal".into(),
                        arguments: vec![],
                    }
                }
                12 => {
                    evidence[0].constraint.arguments[0] = CoreType::Named {
                        name: "std/decimal::Decimal".into(),
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
fn decimal_constant_dictionaries_rejects_counterfeit_owner_kind_method_result_arguments_and_evidence(
) {
    for (name, signature, result) in [
        ("zero", "_unit:Unit -> decimal.Decimal = zero ()", "Int"),
        ("one", "_unit:Unit -> decimal.Decimal = one ()", "Int"),
    ] {
        let source = format!("// 雪🙂\nimport * as decimal from \"std/decimal\"\nfn operation {signature}\npub fn main -> Int = 0\n");
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
                        name: "decimal.Decimal".into(),
                        canonical: "fake::Decimal".into(),
                        arguments: vec![],
                    }
                }
                11 => {
                    evidence[0].constraint.arguments[0] = CoreType::Named {
                        name: "std/decimal::Decimal".into(),
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
fn decimal_arithmetic_binary_operators_reject_forged_operands_results_and_provider_identity() {
    for operator in ["+", "-", "*"] {
        let right = if operator == "**" {
            "Int"
        } else {
            "decimal.Decimal"
        };
        let source=format!("// 雪🙂\nimport * as decimal from \"std/decimal\"\nfn operation left:decimal.Decimal -> right:{right} -> decimal.Decimal = left {operator} right\npub fn main -> Int = 0\n");
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
fn unused_decimal_arithmetic_operations_and_constants_remain_outside_program_wasm() {
    for (signature, operation) in [
        (
            "decimal.Decimal -> decimal.Decimal -> decimal.Decimal",
            "(+)",
        ),
        (
            "decimal.Decimal -> decimal.Decimal -> decimal.Decimal",
            "(-)",
        ),
        (
            "decimal.Decimal -> decimal.Decimal -> decimal.Decimal",
            "(*)",
        ),
        ("Unit -> decimal.Decimal", "zero"),
        ("Unit -> decimal.Decimal", "one"),
        ("decimal.Decimal -> String", "debug"),
    ] {
        let source=format!("import * as decimal from \"std/decimal\"\npub fn probe value:Int -> Int = {{ let saved:{signature} = {operation}; value }}\npub fn main -> Int = 0\n");
        let core = checked(&source);
        emit_rust_module(&core, "main").unwrap();
        let error = emit_program_wasm(&[&core], "native/decimal-arithmetic", &["probe".into()])
            .unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap().source, "decimal-arithmetic-雪🙂.ssrg");
    }
}

#[test]
fn decimal_debug_checks_selected_identity_children_argument_result_and_source_stages() {
    let core=checked("// 雪🙂\nimport * as decimal from \"std/decimal\"\nfn render value: decimal.Decimal -> String = debug value\npub fn main -> Int = 0\n");
    let index = core
        .functions
        .iter()
        .position(|f| f.symbol.ends_with("::render"))
        .unwrap();
    let origin = core_expr_origin(&core.functions[index].body).clone();
    emit_rust_module(&core, "main").unwrap();
    for mutation in 0..18 {
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
                *type_ref = CoreType::Named {
                    name: "std/prelude::Int".into(),
                    arguments: vec![],
                }
            }
            2 => evidence.clear(),
            3 => evidence[0].constraint.arguments[0] = CoreType::Hole,
            4 => evidence[0].constraint.name = "Eq".into(),
            5 => evidence[0].constraint.trait_identity = Some("std/prelude::Show".into()),
            6 => {
                let seseragi_lowering::CoreInstanceEvidence::Standard { identity, .. } =
                    &mut evidence[0].evidence
                else {
                    panic!()
                };
                *identity = "Debug<std/prelude::String>".into();
            }
            7 => {
                let seseragi_lowering::CoreInstanceEvidence::Standard { type_arguments, .. } =
                    &mut evidence[0].evidence
                else {
                    panic!()
                };
                type_arguments.push(CoreType::Hole);
            }
            8 => {
                let extra = evidence[0].clone();
                let seseragi_lowering::CoreInstanceEvidence::Standard {
                    evidence_arguments, ..
                } = &mut evidence[0].evidence
                else {
                    panic!()
                };
                evidence_arguments.push(extra);
            }
            9 => trait_dispatch.as_mut().unwrap().method = "show".into(),
            10 => trait_dispatch.as_mut().unwrap().trait_identity = "std/prelude::Show".into(),
            11 => deferred_evidence_parameters.push(CoreType::Hole),
            12 => deferred_evidence_type_constructor_parameters.push("F".into()),
            13 => *evidence_argument_index = Some(0),
            14 => {
                arguments[0] = CoreExpr::Unit {
                    origin: core_expr_origin(&arguments[0]).clone(),
                }
            }
            15 => arguments.push(arguments[0].clone()),
            16 => evidence.push(evidence[0].clone()),
            _ => *trait_dispatch = None,
        }
        let error = emit_rust_module(&bad, "main").expect_err("forged Decimal Debug");
        assert_eq!(error.code, "SES-RUST-0001", "mutation{mutation}");
        assert_eq!(error.origin.unwrap(), origin, "mutation{mutation}");
    }
}

#[test]
fn decimal_saved_debug_has_its_exact_function_type_and_keeps_program_wasm_closed() {
    let saved = checked("import * as decimal from \"std/decimal\"\nfn formatter -> (decimal.Decimal -> String) = debug\npub fn main -> Int = 0\n");
    emit_rust_module(&saved, "main").unwrap();
    let index = saved
        .functions
        .iter()
        .position(|f| f.symbol.ends_with("::formatter"))
        .unwrap();
    let origin = core_expr_origin(&saved.functions[index].body).clone();
    for result in [
        CoreType::Hole,
        CoreType::Named {
            name: "std/prelude::Int".into(),
            arguments: vec![],
        },
    ] {
        let mut bad = saved.clone();
        let CoreExpr::Call { type_ref, .. } = &mut bad.functions[index].body else {
            panic!()
        };
        let CoreType::Function { result: actual, .. } = type_ref else {
            panic!()
        };
        **actual = result;
        let error = emit_rust_module(&bad, "main").expect_err("forged saved Decimal Debug result");
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap(), origin);
    }
    let core=checked("import * as decimal from \"std/decimal\"\npub fn probe value: Int -> Int = { let saved: decimal.Decimal -> String = debug; value }\npub fn main -> Int = 0\n");
    emit_rust_module(&core, "main").unwrap();
    let error =
        emit_program_wasm(&[&core], "native/decimal-arithmetic", &["probe".into()]).unwrap_err();
    assert_eq!(error.code, "SES-RUST-0001");
    assert!(error.message.contains("requires a target-specific"));
    assert_eq!(error.origin.unwrap().source, "decimal-arithmetic-雪🙂.ssrg");
}

#[test]
fn saved_decimal_arithmetic_methods_reject_counterfeit_function_stages() {
    for method in ["add", "sub", "mul"] {
        let core = checked(&format!("import * as decimal from \"std/decimal\"\nfn operation -> (decimal.Decimal -> decimal.Decimal -> decimal.Decimal) = {method}\npub fn main -> Int = 0\n"));
        let index = core
            .functions
            .iter()
            .position(|function| function.symbol.ends_with("::operation"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[index].body).clone();
        emit_rust_module(&core, "main").unwrap();
        for stage in 0..3 {
            let mut bad = core.clone();
            let CoreExpr::Call { type_ref, .. } = &mut bad.functions[index].body else {
                panic!()
            };
            let CoreType::Function { parameter, result } = type_ref else {
                panic!()
            };
            if stage == 0 {
                **parameter = CoreType::Hole;
            } else if stage == 1 {
                **result = CoreType::Hole;
            } else {
                let CoreType::Function { result, .. } = &mut **result else {
                    panic!()
                };
                **result = CoreType::Named {
                    name: "std/prelude::Int".into(),
                    arguments: vec![],
                };
            }
            let error = emit_rust_module(&bad, "main").unwrap_err();
            assert_eq!(error.code, "SES-RUST-0001", "{method}/{stage}");
            assert_eq!(error.origin.unwrap(), origin);
        }
    }
}
