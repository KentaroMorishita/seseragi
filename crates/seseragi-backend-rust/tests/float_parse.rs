use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreExpr, CoreType};

#[test]
fn float_file_aggregate_has_closed_error_and_operation_contracts() {
    let source = include_str!(
        "../../../examples/spec/fixtures/rust-backend/filesystem-float-aggregate.ssrg"
    );
    let core = compile_core_module(CompileInput::new(
        "aggregate-雫.ssrg",
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
        .any(|c| c == "float-parse"));
}

#[test]
fn float_parse_keeps_saved_functions_and_standard_error_constructors() {
    let source = r#"
import * as floats from "std/float"
fn saved -> (String -> Either<floats.FloatParseError, Float>) = floats.parse
fn error -> floats.FloatParseError = floats.InvalidFloat { offset: 1 }
fn empty -> floats.FloatParseError = floats.EmptyFloat
fn overflow -> floats.FloatParseError = floats.FloatParseOverflow
pub effect fn main = do {
  let parse: String -> Either<floats.FloatParseError, Float> = saved ()
  println (match parse "+42" { Left _ -> "failed"; Right value -> show value })
  println (match error () { floats.InvalidFloat value -> show value.offset; _ -> "other" })
  println (match empty () { floats.EmptyFloat -> "empty"; _ -> "other" })
  println (match overflow () { floats.FloatParseOverflow -> "overflow"; _ -> "other" })
}
"#;
    let core = compile_core_module(CompileInput::new(
        "parse-stages.ssrg",
        "native/main",
        source,
    ))
    .unwrap()
    .core_ir;
    emit_rust_module(&core, "main").unwrap();
}

#[test]
fn float_parse_rejects_unknown_error_owner_kind_source_arity_and_evidence() {
    let core = compile_core_module(CompileInput::new(
        "parse-雫🙂.ssrg", "native/main",
        "import * as floats from \"std/float\"\npub fn probe -> Either<floats.FloatParseError, Float> = floats.parse \"42\"",
    )).unwrap().core_ir;
    emit_rust_module(&core, "probe").unwrap();
    let eq = compile_core_module(CompileInput::new(
        "other.ssrg",
        "native/other",
        "pub fn probe -> Bool = 1 == 1",
    ))
    .unwrap()
    .core_ir;
    let CoreExpr::Binary {
        evidence: other_evidence,
        ..
    } = &eq.functions[0].body
    else {
        panic!()
    };
    for mutation in 0..9 {
        let mut changed = core.clone();
        let CoreExpr::Call {
            arguments,
            type_ref,
            evidence,
            deferred_evidence_parameters,
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
                slots[0] = if mutation == 1 {
                    CoreType::ExternalNamed {
                        name: "FloatParseError".into(),
                        canonical: "fake::FloatParseError".into(),
                        arguments: vec![],
                    }
                } else {
                    CoreType::Named {
                        name: "std/float::FloatParseError".into(),
                        arguments: vec![CoreType::Named {
                            name: "Unit".into(),
                            arguments: vec![],
                        }],
                    }
                };
            }
            3 => arguments.clear(),
            4 => arguments.push(arguments[0].clone()),
            5 => evidence.push(other_evidence[0].clone()),
            6 => deferred_evidence_parameters.push(CoreType::Hole),
            7 => {
                let CoreType::Named {
                    arguments: slots, ..
                } = type_ref
                else {
                    panic!()
                };
                slots[1] = CoreType::Named {
                    name: "String".into(),
                    arguments: vec![],
                };
            }
            _ => {
                arguments[0] = CoreExpr::Integer {
                    value: "1".into(),
                    origin: seseragi_lowering::core_expr_origin(&arguments[0]).clone(),
                }
            }
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001", "case {mutation}");
        assert_eq!(error.origin.unwrap().source, "parse-雫🙂.ssrg");
    }
}
