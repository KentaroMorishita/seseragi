use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreTraitDispatch, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "bytes-codecs-雪🙂.ssrg",
        "native/codecs",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:?}"))
    .core_ir
}

const OPERATIONS: &[(&str, &str, &str)] = &[
    ("base64.encode", "bytes.Bytes", "String"),
    ("base64.encodeUrl", "bytes.Bytes", "String"),
    (
        "base64.decode",
        "String",
        "Either<base64.Base64DecodeError, bytes.Bytes>",
    ),
    (
        "base64.decodeUrl",
        "String",
        "Either<base64.Base64DecodeError, bytes.Bytes>",
    ),
    ("hex.encode", "bytes.Bytes", "String"),
    (
        "hex.decode",
        "String",
        "Either<hex.HexDecodeError, bytes.Bytes>",
    ),
];

const IMPORTS: &str = "import * as bytes from \"std/bytes\"\nimport * as base64 from \"std/bytes/base64\"\nimport * as hex from \"std/bytes/hex\"\n";

#[test]
fn bytes_codecs_accept_the_existing_spec_example() {
    let artifact = emit_rust_module(
        &checked(include_str!(
            "../../../examples/spec/artifacts/schema-1/bytes-codecs/main.ssrg"
        )),
        "main",
    )
    .unwrap();
    assert!(artifact.metadata.capabilities.iter().any(|c| c == "bytes"));
}

#[test]
fn bytes_codecs_accept_direct_saved_and_generic_calls() {
    let artifact = emit_rust_module(
        &checked(include_str!(
            "../../../examples/spec/fixtures/rust-backend/bytes-codecs.ssrg"
        )),
        "main",
    )
    .unwrap();
    assert!(artifact.metadata.capabilities.iter().any(|c| c == "bytes"));
}

#[test]
fn bytes_codecs_validate_argument_result_evidence_and_source_stages() {
    let eq = checked("pub fn probe -> Bool = 1 == 1");
    let CoreExpr::Binary {
        evidence: other, ..
    } = &eq.functions[0].body
    else {
        panic!()
    };
    for &(operation, parameter, result) in OPERATIONS {
        let core = checked(&format!(
            "{IMPORTS}pub fn probe value: {parameter} -> {result} = {operation} value\npub fn main -> Int = 0\n"
        ));
        emit_rust_module(&core, "main").unwrap();
        let probe = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::probe"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[probe].body).clone();
        for mutation in 0..15 {
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
                        name: "std/bytes::Bytes".into(),
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
                    *trait_dispatch = Some(CoreTraitDispatch {
                        trait_identity: "std/prelude::Eq".into(),
                        method: "eq".into(),
                        method_contract: None,
                    })
                }
                10 => {
                    arguments[0] = CoreExpr::Unit {
                        origin: core_expr_origin(&arguments[0]).clone(),
                    }
                }
                _ => {
                    let CoreExpr::Variable { type_ref, .. } = &mut arguments[0] else {
                        panic!()
                    };
                    *type_ref = match mutation {
                        11 => CoreType::Hole,
                        12 => CoreType::ExternalNamed {
                            name: parameter.into(),
                            canonical: "fake::Input".into(),
                            arguments: vec![],
                        },
                        13 => CoreType::Named {
                            name: if parameter == "String" {
                                "std/prelude::String"
                            } else {
                                "std/bytes::Bytes"
                            }
                            .into(),
                            arguments: vec![CoreType::Hole],
                        },
                        _ => CoreType::Named {
                            name: "std/prelude::Int".into(),
                            arguments: vec![],
                        },
                    };
                }
            }
            let error = emit_rust_module(&changed, "main").expect_err("unproven bytes codec call");
            assert_eq!(error.code, "SES-RUST-0001", "{operation}/{mutation}");
            assert_eq!(error.origin.unwrap(), origin, "{operation}/{mutation}");
        }
    }
}

#[test]
fn bytes_codecs_build_as_program_wasm() {
    for &(operation, parameter, result) in OPERATIONS {
        let core = checked(&format!(
            "{IMPORTS}pub fn probe value: Int -> Int = {{ let saved: {parameter} -> {result} = {operation}; value }}\npub fn main -> Int = probe 42\n"
        ));
        emit_rust_module(&core, "main").unwrap();
        emit_program_wasm(&[&core], "native/codecs", &["probe".into()])
            .unwrap_or_else(|error| panic!("{error:?}"));
    }
}
