use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreTraitDispatch, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new("utf8-雪🙂.ssrg", "native/utf8", source))
        .unwrap_or_else(|error| panic!("{source}\n{error:?}"))
        .core_ir
}

#[test]
fn utf8_accepts_direct_saved_and_generic_operations() {
    let artifact = emit_rust_module(
        &checked(include_str!(
            "../../../examples/spec/fixtures/rust-backend/text-utf8.ssrg"
        )),
        "main",
    )
    .unwrap();
    assert!(artifact.metadata.capabilities.iter().any(|c| c == "text"));
}

#[test]
fn utf8_rejects_unproven_argument_result_evidence_and_source_stages() {
    let eq = checked("pub fn probe -> Bool = 1 == 1");
    let CoreExpr::Binary {
        evidence: other, ..
    } = &eq.functions[0].body
    else {
        panic!()
    };
    for (parameter, result, expression) in [
        ("String", "bytes.Bytes", "text.encodeUtf8 value"),
        (
            "bytes.Bytes",
            "Either<text.Utf8DecodeError, String>",
            "text.decodeUtf8 value",
        ),
        ("bytes.Bytes", "String", "text.decodeUtf8Lossy value"),
    ] {
        let core = checked(&format!(
            "import * as bytes from \"std/bytes\"\nimport * as text from \"std/text\"\npub fn probe value: {parameter} -> {result} = {expression}\npub fn main -> Int = 0\n"
        ));
        emit_rust_module(&core, "main").unwrap();
        let probe = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::probe"))
            .unwrap();
        let expected_origin = core_expr_origin(&core.functions[probe].body).clone();
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
                _ => {
                    arguments[0] = CoreExpr::Unit {
                        origin: core_expr_origin(&arguments[0]).clone(),
                    }
                }
            }
            let error = emit_rust_module(&changed, "main").expect_err("unproven UTF-8 call");
            assert_eq!(error.code, "SES-RUST-0001", "{expression}/{mutation}");
            assert_eq!(
                error.origin.unwrap(),
                expected_origin,
                "{expression}/{mutation}"
            );
        }
    }
}

#[test]
fn utf8_keeps_unused_program_wasm_references_rejected() {
    for (operation, signature) in [
        ("encodeUtf8", "String -> bytes.Bytes"),
        (
            "decodeUtf8",
            "bytes.Bytes -> Either<text.Utf8DecodeError, String>",
        ),
        ("decodeUtf8Lossy", "bytes.Bytes -> String"),
    ] {
        let core = checked(&format!(
            "import * as bytes from \"std/bytes\"\nimport * as text from \"std/text\"\npub fn probe value: Int -> Int = {{ let saved: {signature} = text.{operation}; value }}\npub fn main -> Int = probe 42\n"
        ));
        emit_rust_module(&core, "main").unwrap();
        let error = emit_program_wasm(&[&core], "native/utf8", &["probe".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(
            error.message.contains("requires a target-specific proof"),
            "{operation}: {error}"
        );
        assert_eq!(error.origin.unwrap().source, "utf8-雪🙂.ssrg");
    }
}
