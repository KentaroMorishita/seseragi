use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "decimal-division-雪🙂.ssrg",
        "native/decimal-division",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:?}"))
    .core_ir
}

#[test]
fn decimal_division_accepts_saved_partial_generic_and_cold_operations() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-decimal-division.ssrg"),
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/basic-decimal-division-effects.ssrg"
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
fn decimal_division_rejects_unproven_owners_kinds_arguments_evidence_and_stages() {
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
            "Either<decimal.DecimalArithmeticError,decimal.Decimal>",
            "decimal.divideExact (decimal.fromInt 2) (decimal.fromInt 5)",
        ),
        (
            "Either<decimal.DecimalArithmeticError,decimal.Decimal>",
            "decimal.divide context (decimal.fromInt 3) (decimal.fromInt 1)",
        ),
        (
            "decimal.Decimal",
            "decimal.quantize 2 numbers.HalfEven (decimal.fromInt 5)",
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
fn decimal_division_does_not_admit_unused_operations_in_program_wasm() {
    for (signature, operation) in [
        ("decimal.Decimal -> decimal.Decimal -> Either<decimal.DecimalArithmeticError,decimal.Decimal>", "divideExact"),
        ("decimal.DecimalContext -> decimal.Decimal -> decimal.Decimal -> Either<decimal.DecimalArithmeticError,decimal.Decimal>", "divide"),
        ("Int -> numbers.RoundingMode -> decimal.Decimal -> decimal.Decimal", "quantize"),
        ("decimal.DecimalArithmeticError", "DecimalDivisionByZero"),
        ("decimal.DecimalArithmeticError", "NonTerminatingDecimal"),
    ] {
        let core = checked(&format!("import * as decimal from \"std/decimal\"\nimport * as numbers from \"std/number\"\npub fn probe value: Int -> Int = {{ let saved: {signature} = decimal.{operation}; value }}\npub fn main -> Int = 0\n"));
        emit_rust_module(&core, "main").unwrap();
        let error =
            emit_program_wasm(&[&core], "native/decimal-division", &["probe".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap().source, "decimal-division-雪🙂.ssrg");
    }
}

#[test]
fn arithmetic_error_constants_keep_canonical_owner_kind_and_renamed_display_aliases() {
    for constructor in ["DecimalDivisionByZero", "NonTerminatingDecimal"] {
        let source = format!("import * as decimal from \"std/decimal\"\nfn probe -> decimal.DecimalArithmeticError = decimal.{constructor}\npub fn main -> Int = 0\n");
        let core = checked(&source);
        let index = core
            .functions
            .iter()
            .position(|function| function.symbol.ends_with("::probe"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[index].body).clone();
        for mutation in 0..6 {
            let mut changed = core.clone();
            let actual = match &mut changed.functions[index].body {
                CoreExpr::Call { type_ref, .. } | CoreExpr::Variable { type_ref, .. } => type_ref,
                _ => panic!(),
            };
            *actual = match mutation {
                0 => CoreType::Hole,
                1 => CoreType::Named {
                    name: "std/prelude::Int".into(),
                    arguments: vec![],
                },
                2 => CoreType::ExternalNamed {
                    name: "decimal.DecimalArithmeticError".into(),
                    canonical: "fake::DecimalArithmeticError".into(),
                    arguments: vec![],
                },
                3 => CoreType::Named {
                    name: "std/decimal::DecimalArithmeticError".into(),
                    arguments: vec![CoreType::Hole],
                },
                4 => CoreType::Named {
                    name: "std/decimal::DecimalContextError".into(),
                    arguments: vec![],
                },
                _ => CoreType::Named {
                    name: "DecimalArithmeticError".into(),
                    arguments: vec![],
                },
            };
            let error = emit_rust_module(&changed, "main").unwrap_err();
            assert_eq!(error.code, "SES-RUST-0001");
            assert_eq!(error.origin.unwrap(), origin);
        }
        let mut alias = core.clone();
        let actual = match &mut alias.functions[index].body {
            CoreExpr::Call { type_ref, .. } | CoreExpr::Variable { type_ref, .. } => type_ref,
            _ => panic!(),
        };
        *actual = CoreType::ExternalNamed {
            name: "renamed.Failure".into(),
            canonical: "std/decimal::DecimalArithmeticError".into(),
            arguments: vec![],
        };
        emit_rust_module(&alias, "main").unwrap();
    }
}

#[test]
fn saved_division_and_quantize_factories_reject_forged_remaining_stages() {
    for (signature,expression) in [
        ("decimal.Decimal -> Either<decimal.DecimalArithmeticError,decimal.Decimal>","decimal.divideExact (decimal.fromInt 2)"),
        ("decimal.Decimal -> decimal.Decimal -> Either<decimal.DecimalArithmeticError,decimal.Decimal>","decimal.divide context"),
        ("numbers.RoundingMode -> decimal.Decimal -> decimal.Decimal","decimal.quantize 2"),
    ] {
        let core=checked(&format!("import * as decimal from \"std/decimal\"\nimport * as numbers from \"std/number\"\nfn operation context:decimal.DecimalContext -> ({signature}) = {expression}\npub fn main -> Int = 0\n"));
        let index=core.functions.iter().position(|function|function.symbol.ends_with("::operation")).unwrap();
        let origin=core_expr_origin(&core.functions[index].body).clone();
        emit_rust_module(&core,"main").unwrap();
        for mutation in 0..10 {
            let mut bad=core.clone();
            let CoreExpr::Call {type_ref,arguments,evidence_argument_index,deferred_evidence_parameters,callee,..}=&mut bad.functions[index].body else {panic!()};
            match mutation {
                0=>*type_ref=CoreType::Hole,
                1=>*type_ref=CoreType::Named {name:"std/prelude::Int".into(),arguments:vec![]},
                2=>{let CoreType::Function {parameter,..}=type_ref else {panic!()};**parameter=CoreType::Hole;},
                3=>arguments[0]=CoreExpr::Boolean {value:true,origin:origin.clone()},
                4=>arguments.clear(),
                5=>{let argument=arguments[0].clone();arguments.extend([argument.clone(),argument.clone(),argument]);},
                6=>*evidence_argument_index=Some(0),
                7=>{let CoreType::Function {result,..}=type_ref else {panic!()};**result=CoreType::Named {name:"std/prelude::Int".into(),arguments:vec![]};},
                8=>deferred_evidence_parameters.push(CoreType::Hole),
                _=>*callee="fake/decimal::divide".into(),
            }
            let error=emit_rust_module(&bad,"main").unwrap_err();assert_eq!(error.code,"SES-RUST-0001","{expression}/{mutation}");assert_eq!(error.origin.unwrap(),origin);
        }
    }
}
