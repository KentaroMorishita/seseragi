use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "decimal-context-雪🙂.ssrg",
        "native/decimal-context",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:?}"))
    .core_ir
}

#[test]
fn decimal_context_accepts_saved_partial_generic_and_cold_operations() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-decimal-context.ssrg"),
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/basic-decimal-context-effects.ssrg"
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
fn decimal_context_rejects_unproven_owners_kinds_arguments_evidence_and_stages() {
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
            "Either<decimal.DecimalContextError,decimal.DecimalContext>",
            "decimal.context 34 numbers.HalfEven",
        ),
        ("Int", "decimal.precision context"),
        ("numbers.RoundingMode", "decimal.rounding context"),
        (
            "decimal.DecimalContextError",
            "decimal.NonPositiveDecimalPrecision (-1)",
        ),
    ] {
        let core = checked(&format!("// 雪🙂\nimport * as decimal from \"std/decimal\"\nimport * as numbers from \"std/number\"\nfn probe context: decimal.DecimalContext -> {result} = {expression}\npub fn main -> Int = 0\n"));
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
            let error = emit_rust_module(&bad, "main").expect_err("forged Decimal context call");
            assert_eq!(error.code, "SES-RUST-0001", "{expression}/{mutation}");
            assert_eq!(error.origin.unwrap(), origin, "{expression}/{mutation}");
        }
    }
}

#[test]
fn decimal_context_does_not_admit_unused_operations_in_program_wasm() {
    for (signature, operation) in [
        ("Int -> numbers.RoundingMode -> Either<decimal.DecimalContextError,decimal.DecimalContext>", "context"),
        ("decimal.DecimalContext -> Int", "precision"),
        ("decimal.DecimalContext -> numbers.RoundingMode", "rounding"),
        ("Int -> decimal.DecimalContextError", "NonPositiveDecimalPrecision"),
    ] {
        let core = checked(&format!("import * as decimal from \"std/decimal\"\nimport * as numbers from \"std/number\"\npub fn probe value: Int -> Int = {{ let saved: {signature} = decimal.{operation}; value }}\npub fn main -> Int = 0\n"));
        emit_rust_module(&core, "main").unwrap();
        let error =
            emit_program_wasm(&[&core], "native/decimal-context", &["probe".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap().source, "decimal-context-雪🙂.ssrg");
    }
}
