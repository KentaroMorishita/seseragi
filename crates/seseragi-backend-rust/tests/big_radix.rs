use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "big-radix-雪🙂.ssrg",
        "native/big-radix",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:?}"))
    .core_ir
}

#[test]
fn big_radix_accepts_saved_partial_generic_and_cold_operations() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-big-int-radix.ssrg"),
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/basic-big-int-radix-effects.ssrg"
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
fn big_radix_rejects_unproven_owners_kinds_arguments_evidence_and_stages() {
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
            "Either<big.BigIntParseError, big.BigInt>",
            "big.parseRadix 16 \"ff\"",
        ),
        (
            "Either<big.BigIntParseError, String>",
            "big.formatRadix 16 (big.fromInt 42)",
        ),
        (
            "Either<big.BigIntConversionError, Int>",
            "big.toInt (big.fromInt 42)",
        ),
    ] {
        let core = checked(&format!("// 雪🙂\nimport * as big from \"std/big-int\"\nfn probe -> {result} = {expression}\npub fn main -> Int = 0\n"));
        let index = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::probe"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[index].body).clone();
        emit_rust_module(&core, "main").unwrap();
        for mutation in 0..15 {
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
                11 => {
                    let CoreType::Named { name, .. } = type_ref else {
                        panic!()
                    };
                    *name = "fake::Result".into();
                }
                12..=14 => {
                    let CoreType::Named {
                        arguments: slots, ..
                    } = type_ref
                    else {
                        panic!()
                    };
                    if mutation == 14 {
                        slots[1] = CoreType::Named {
                            name: "std/prelude::Unit".into(),
                            arguments: vec![],
                        };
                    } else if mutation == 12 {
                        slots[0] = CoreType::ExternalNamed {
                            name: "Error".into(),
                            canonical: "fake::Error".into(),
                            arguments: vec![],
                        };
                    } else {
                        let (CoreType::Named { arguments, .. }
                        | CoreType::ExternalNamed { arguments, .. }) = &mut slots[0]
                        else {
                            panic!()
                        };
                        arguments.push(CoreType::Hole);
                    }
                }
                _ => unreachable!(),
            }
            let error = emit_rust_module(&bad, "main").expect_err("forged BigInt radix call");
            assert_eq!(error.code, "SES-RUST-0001", "{expression}/{mutation}");
            assert_eq!(error.origin.unwrap(), origin, "{expression}/{mutation}");
        }
    }
}

#[test]
fn big_radix_does_not_admit_unused_operations_in_program_wasm() {
    for (signature, operation) in [
        (
            "Int -> String -> Either<big.BigIntParseError, big.BigInt>",
            "parseRadix",
        ),
        (
            "Int -> big.BigInt -> Either<big.BigIntParseError, String>",
            "formatRadix",
        ),
        (
            "big.BigInt -> Either<big.BigIntConversionError, Int>",
            "toInt",
        ),
        ("big.BigIntParseError", "EmptyBigInt"),
        ("Int -> big.BigIntParseError", "InvalidBigIntRadix"),
        (
            "{ offset: Int, radix: Int } -> big.BigIntParseError",
            "InvalidBigIntDigit",
        ),
        ("big.BigIntConversionError", "BigIntOutsideIntRange"),
    ] {
        let core = checked(&format!("import * as big from \"std/big-int\"\npub fn probe value: Int -> Int = {{ let saved: {signature} = big.{operation}; value }}\npub fn main -> Int = 0\n"));
        emit_rust_module(&core, "main").unwrap();
        let error = emit_program_wasm(&[&core], "native/big-radix", &["probe".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap().source, "big-radix-雪🙂.ssrg");
    }
}
