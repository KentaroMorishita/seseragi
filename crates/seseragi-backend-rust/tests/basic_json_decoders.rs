use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreType};
fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "decoder-雪🙂.ssrg",
        "native/json-decoders",
        source,
    ))
    .unwrap_or_else(|e| panic!("{source}\n{e:?}"))
    .core_ir
}
#[test]
fn decoders_keep_generic_saved_partial_and_returned_source_applications() {
    let core = checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-json-decoders.ssrg"
    ));
    emit_rust_module(&core, "main").unwrap_or_else(|e| panic!("{e:?}"));
}
#[test]
fn decoder_operations_reject_unproved_source_stages_and_types_at_original_span() {
    for (result, expression) in [
        ("json.Decoder<Int>", "json.field \"n\" decodeJson"),
        (
            "json.Decoder<Maybe<Int>>",
            "json.optionalField \"n\" decodeJson",
        ),
        ("json.Decoder<Int>", "json.index 0 decodeJson"),
        ("json.Decoder<Array<Int>>", "json.array decodeJson"),
        (
            "json.Decoder<Array<(String,Int)>>",
            "json.record [(\"n\",decodeJson)]",
        ),
        ("json.Decoder<Int>", "json.oneOf [decodeJson]"),
        (
            "json.Decoder<String>",
            "json.map (\\value: Int -> show value) decodeJson",
        ),
        (
            "json.Decoder<Int>",
            "json.flatMap (\\_: Int -> decodeJson<Int>) decodeJson",
        ),
    ] {
        let source=format!("// 雪🙂\nimport * as json from \"std/json\"\npub fn probe -> {result} = {expression}\npub fn main -> Int = 0\n");
        let core = checked(&source);
        emit_rust_module(&core, "main").unwrap();
        let index = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::probe"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[index].body).clone();
        for mutation in 0..11 {
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
                        name: "Decoder".into(),
                        canonical: "fake::Decoder".into(),
                        arguments: vec![],
                    }
                }
                2 => {
                    arguments[0] = CoreExpr::Unit {
                        origin: core_expr_origin(&arguments[0]).clone(),
                    }
                }
                3 => arguments.extend([
                    CoreExpr::Unit {
                        origin: origin.clone(),
                    },
                    CoreExpr::Unit {
                        origin: origin.clone(),
                    },
                ]),
                4 => deferred_evidence_parameters.push(CoreType::Hole),
                5 => deferred_evidence_type_constructor_parameters.push("F".into()),
                6 => *evidence_argument_index = Some(0),
                7 => {
                    *trait_dispatch = Some(seseragi_lowering::CoreTraitDispatch {
                        trait_identity: "std/prelude::Show".into(),
                        method: "show".into(),
                        method_contract: None,
                    })
                }
                8 => {
                    let selected = checked("pub fn probe -> Bool = 1 == 1");
                    let CoreExpr::Binary {
                        evidence: extra, ..
                    } = &selected.functions[0].body
                    else {
                        panic!()
                    };
                    evidence.push(extra[0].clone());
                }
                9 => {
                    let CoreType::Function { parameter, .. } = type_ref else {
                        panic!()
                    };
                    **parameter = CoreType::Named {
                        name: "String".into(),
                        arguments: vec![],
                    };
                }
                _ => {
                    let CoreType::Function { result, .. } = type_ref else {
                        panic!()
                    };
                    let CoreType::Named { arguments, .. } = &mut **result else {
                        panic!()
                    };
                    arguments[0] = CoreType::ExternalNamed {
                        name: "DecodeError".into(),
                        canonical: "fake::DecodeError".into(),
                        arguments: vec![],
                    };
                }
            }
            let error = emit_rust_module(&bad, "main").unwrap_err();
            assert_eq!(
                error.origin.as_ref(),
                Some(&origin),
                "{expression}:{mutation}:{error:?}"
            );
        }
        let saved=checked(&format!("import * as json from \"std/json\"\npub fn probe value:Int -> Int = {{let saved: {result} = {expression};value}}\npub fn main -> Int = 0\n"));
        let error =
            emit_program_wasm(&[&saved], "native/json-decoders", &["probe".into()]).unwrap_err();
        assert!(
            error.message.contains("require a target-specific"),
            "{error:?}"
        );
    }
}
