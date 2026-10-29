use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "decimal-雪🙂.ssrg",
        "native/decimal",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:?}"))
    .core_ir
}

#[test]
fn basic_decimal_accepts_saved_functions_exact_values_and_captured_opaque_storage() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-decimal.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-decimal-effects.ssrg"),
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
fn basic_decimal_rejects_forged_owner_kind_evidence_and_stages_at_the_original_byte_span() {
    let eq = checked("pub fn probe -> Bool = 1 == 1");
    let CoreExpr::Binary {
        evidence: other, ..
    } = &eq.functions[0].body
    else {
        panic!()
    };
    for (parameter, result, expression) in [
        (
            "",
            "Either<big.DecimalParseError, big.Decimal>",
            "big.parse \"900719925474099200000001\"",
        ),
        ("", "big.Decimal", "big.fromInt 42"),
    ] {
        let source = format!("// 雪🙂\nimport * as big from \"std/decimal\"\nfn probe{parameter} -> {result} = {expression}\npub fn main -> Int = 0\n");
        let core = checked(&source);
        let probe = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::probe"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[probe].body).clone();
        emit_rust_module(&core, "main").unwrap();
        for mutation in 0..11 {
            let mut changed = core.clone();
            let CoreExpr::Call {
                arguments,
                type_ref,
                evidence,
                trait_dispatch,
                deferred_evidence_parameters,
                deferred_evidence_type_constructor_parameters,
                evidence_argument_index,
                ..
            } = &mut changed.functions[probe].body
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
                        name: "std/decimal::Decimal".into(),
                        arguments: vec![CoreType::Hole],
                    }
                }
                3 => arguments.clear(),
                4 => arguments.push(arguments[0].clone()),
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
                _ => {
                    arguments[0] = CoreExpr::Unit {
                        origin: core_expr_origin(&arguments[0]).clone(),
                    }
                }
            }
            let error = emit_rust_module(&changed, "main")
                .err()
                .unwrap_or_else(|| panic!("{expression}/{mutation} accepted"));
            assert_eq!(error.code, "SES-RUST-0001");
            assert_eq!(error.origin.unwrap(), origin, "{expression}/{mutation}");
        }
    }
}

#[test]
fn basic_decimal_does_not_replace_scoped_opaque_or_int_binders() {
    for (binder, result, expression) in [
        ("Decimal", "String", "show (big.fromInt 42)"),
        ("Int", "big.Decimal", "big.fromInt 42"),
    ] {
        let mut core = checked(&format!(
            "fn identity<{binder}> value: {binder} -> {binder} = value\npub fn main -> Int = 0\n"
        ));
        let other = checked(&format!(
            "import * as big from \"std/decimal\"\npub fn probe -> {result} = {expression}\n"
        ));
        let mut call = other.functions[0].body.clone();
        let identity = core
            .functions
            .iter_mut()
            .find(|f| f.symbol.ends_with("::identity"))
            .unwrap();
        let CoreExpr::Call { arguments, .. } = &mut call else {
            panic!()
        };
        arguments[0] = identity.body.clone();
        let origin = core_expr_origin(&call).clone();
        identity.body = call;
        let error = emit_rust_module(&core, "main").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap(), origin);
    }
}

#[test]
fn basic_decimal_keeps_program_wasm_admission_explicit() {
    for (name, signature) in [
        (
            "parse",
            "String -> Either<big.DecimalParseError, big.Decimal>",
        ),
        ("fromInt", "Int -> big.Decimal"),
    ] {
        let source=format!("import * as big from \"std/decimal\"\npub fn probe value: Int -> Int = {{ let saved: {signature} = big.{name}; value }}\npub fn main -> Int = probe 1\n");
        let core = checked(&source);
        emit_rust_module(&core, "main").unwrap();
        let error = emit_program_wasm(&[&core], "native/decimal", &["probe".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(error.message.contains("requires a target-specific"));
        assert_eq!(error.origin.unwrap().source, "decimal-雪🙂.ssrg");
    }
}

#[test]
fn decimal_show_checks_selected_identity_children_argument_result_and_source_stages() {
    let core=checked("// 雪🙂\nimport * as decimal from \"std/decimal\"\nfn render value: decimal.Decimal -> String = show value\npub fn main -> Int = 0\n");
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
            5 => evidence[0].constraint.trait_identity = Some("std/prelude::Debug".into()),
            6 => {
                let seseragi_lowering::CoreInstanceEvidence::Standard { identity, .. } =
                    &mut evidence[0].evidence
                else {
                    panic!()
                };
                *identity = "Show<std/prelude::String>".into();
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
            9 => trait_dispatch.as_mut().unwrap().method = "debug".into(),
            10 => trait_dispatch.as_mut().unwrap().trait_identity = "std/prelude::Debug".into(),
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
        let error = emit_rust_module(&bad, "main").expect_err("forged Decimal Show");
        assert_eq!(error.code, "SES-RUST-0001", "mutation{mutation}");
        assert_eq!(error.origin.unwrap(), origin, "mutation{mutation}");
    }
}

#[test]
fn decimal_saved_show_has_its_exact_function_type_and_keeps_program_wasm_closed() {
    let saved = checked("import * as decimal from \"std/decimal\"\nfn formatter -> (decimal.Decimal -> String) = show\npub fn main -> Int = 0\n");
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
        let error = emit_rust_module(&bad, "main").expect_err("forged saved Decimal Show result");
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap(), origin);
    }
    let core=checked("import * as decimal from \"std/decimal\"\npub fn probe value: Int -> Int = { let saved: decimal.Decimal -> String = show; value }\npub fn main -> Int = 0\n");
    emit_rust_module(&core, "main").unwrap();
    let error = emit_program_wasm(&[&core], "native/decimal", &["probe".into()]).unwrap_err();
    assert_eq!(error.code, "SES-RUST-0001");
    assert!(error.message.contains("requires a target-specific"));
    assert_eq!(error.origin.unwrap().source, "decimal-雪🙂.ssrg");
}
