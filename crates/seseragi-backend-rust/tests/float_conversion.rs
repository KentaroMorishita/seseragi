use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreExpr, CoreType};

#[test]
fn saved_rounding_and_conversions_keep_canonical_closed_signatures() {
    let source = r#"
import * as floats from "std/float"
import * as numbers from "std/number"
fn saved -> (numbers.RoundingMode -> Float -> Either<floats.FloatConversionError, Int>) = floats.toInt
fn partial mode: numbers.RoundingMode -> (Float -> Either<floats.FloatConversionError, Int>) = floats.toInt mode
fn error -> floats.FloatConversionError = floats.FloatNotFinite
pub effect fn main = do {
  let convert = partial numbers.HalfEven
  println (match convert 2.5 { Left _ -> "failed"; Right value -> show value })
  println (show (floats.roundIntegral numbers.HalfUp (-2.5)))
  println (match error () { floats.FloatNotFinite -> "finite"; _ -> "other" })
}
"#;
    let core = compile_core_module(CompileInput::new(
        "conversion-雫.ssrg",
        "native/main",
        source,
    ))
    .unwrap()
    .core_ir;
    let artifact = emit_rust_module(&core, "main").unwrap();
    assert!(artifact
        .metadata
        .capabilities
        .iter()
        .any(|c| c == "float-conversion"));
}

#[test]
fn conversion_rejects_forged_mode_error_kind_evidence_and_source_stages() {
    let source="import * as floats from \"std/float\"\nimport * as numbers from \"std/number\"\npub fn probe -> Either<floats.FloatConversionError, Int> = floats.toInt numbers.HalfEven 2.5";
    let core = compile_core_module(CompileInput::new(
        "conversion-雫🙂.ssrg",
        "native/main",
        source,
    ))
    .unwrap()
    .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    let eq = compile_core_module(CompileInput::new(
        "eq.ssrg",
        "native/eq",
        "pub fn probe -> Bool = 1 == 1",
    ))
    .unwrap()
    .core_ir;
    let CoreExpr::Binary {
        evidence: other, ..
    } = &eq.functions[0].body
    else {
        panic!()
    };
    for mutation in 0..12 {
        let mut changed = core.clone();
        let CoreExpr::Call {
            arguments,
            type_ref,
            evidence,
            deferred_evidence_parameters,
            evidence_argument_index,
            ..
        } = &mut changed.functions[0].body
        else {
            panic!()
        };
        match mutation {
            0 => *type_ref = CoreType::Hole,
            1 | 2 => {
                let CoreType::Named {
                    arguments: slots, ..
                } = type_ref
                else {
                    panic!()
                };
                slots[0] = CoreType::Named {
                    name: if mutation == 1 {
                        "fake::FloatConversionError"
                    } else {
                        "std/float::FloatConversionError"
                    }
                    .into(),
                    arguments: if mutation == 1 {
                        vec![]
                    } else {
                        vec![CoreType::Hole]
                    },
                };
            }
            3 => {
                arguments.remove(0);
            }
            4 => arguments.push(arguments[0].clone()),
            5..=7 => {
                let CoreExpr::Variable { type_ref, .. } = &mut arguments[0] else {
                    panic!()
                };
                *type_ref = if mutation == 5 {
                    CoreType::Hole
                } else {
                    CoreType::Named {
                        name: if mutation == 6 {
                            "fake::RoundingMode"
                        } else {
                            "std/number::RoundingMode"
                        }
                        .into(),
                        arguments: if mutation == 6 {
                            vec![]
                        } else {
                            vec![CoreType::Hole]
                        },
                    }
                };
            }
            8 => {
                arguments[1] = CoreExpr::String {
                    value: "2.5".into(),
                    origin: seseragi_lowering::core_expr_origin(&arguments[1]).clone(),
                }
            }
            9 => evidence.push(other[0].clone()),
            10 => deferred_evidence_parameters.push(CoreType::Hole),
            _ => *evidence_argument_index = Some(0),
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001", "case{mutation}");
        assert_eq!(error.origin.unwrap().source, "conversion-雫🙂.ssrg");
    }
}

#[test]
fn program_wasm_conversion_remains_an_explicit_positioned_refusal() {
    let source = r#"
import * as floats from "std/float"
import * as numbers from "std/number"
pub fn probe value: Int -> Int = match floats.toInt numbers.HalfEven (floats.fromInt value) {
  Left _ -> 0
  Right integer -> integer
}
"#;
    let core = compile_core_module(CompileInput::new(
        "conversion-wasm.ssrg",
        "program/main",
        source,
    ))
    .unwrap()
    .core_ir;
    let error = emit_program_wasm(&[&core], "program/main", &["probe".into()]).unwrap_err();
    assert_eq!(error.code, "SES-RUST-0001");
    assert!(
        error
            .message
            .contains("Float conversion is not admitted in program WASM"),
        "{error}"
    );
    assert_eq!(error.origin.unwrap().source, "conversion-wasm.ssrg");
}
