use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreInstanceEvidence, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "codec-雪🙂.ssrg",
        "native/json-codecs",
        source,
    ))
    .unwrap_or_else(|e| panic!("{source}\n{e:?}"))
    .core_ir
}
#[test]
fn primitive_json_codecs_keep_selected_generic_saved_and_custom_dictionaries() {
    let core = checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-json-codecs.ssrg"
    ));
    let emitted = emit_rust_module(&core, "main").unwrap_or_else(|e| panic!("{e:?}"));
    assert!(emitted.metadata.capabilities.iter().any(|c| c == "json"));
}

#[test]
fn codec_calls_reject_forged_owner_kind_dispatch_evidence_and_source_stages_at_original_span() {
    for (parameter, result, expression) in [
        ("value: Int", "String", "json.encodeString value"),
        (
            "value: String",
            "Either<json.JsonReadError, Int>",
            "json.decodeString value",
        ),
        ("value: Int", "json.Json", "encodeJson value"),
        (
            "value: json.Json",
            "Either<json.DecodeError, Int>",
            "decodeJson value",
        ),
    ] {
        let source = format!("// 雪🙂\nimport * as json from \"std/json\"\npub fn probe {parameter} -> {result} = {expression}\npub fn main -> Int = 0\n");
        let core = checked(&source);
        emit_rust_module(&core, "main").unwrap();
        let index = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::probe"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[index].body).clone();
        for mutation in 0..20 {
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
                1 => evidence.clear(),
                2 => evidence.push(evidence[0].clone()),
                3 => evidence[0].constraint.name = "Show".into(),
                4 => evidence[0].constraint.trait_identity = Some("fake::JsonEncode".into()),
                5 => evidence[0].constraint.arguments.clear(),
                6 => evidence[0].constraint.arguments.push(CoreType::Hole),
                7 => evidence[0].constraint.arguments[0] = CoreType::Hole,
                8 => {
                    evidence[0].constraint.arguments[0] = CoreType::Named {
                        name: "Array".into(),
                        arguments: vec![],
                    }
                }
                9 => {
                    evidence[0].constraint.arguments[0] = CoreType::ExternalNamed {
                        name: "Int".into(),
                        canonical: "fake::Int".into(),
                        arguments: vec![],
                    }
                }
                10 => {
                    let CoreInstanceEvidence::Standard { type_arguments, .. } =
                        &mut evidence[0].evidence
                    else {
                        panic!()
                    };
                    type_arguments.push(CoreType::Hole);
                }
                11 => {
                    let child = evidence[0].clone();
                    let CoreInstanceEvidence::Standard {
                        evidence_arguments, ..
                    } = &mut evidence[0].evidence
                    else {
                        panic!()
                    };
                    evidence_arguments.push(child);
                }
                12 => {
                    let CoreInstanceEvidence::Standard { identity, .. } = &mut evidence[0].evidence
                    else {
                        panic!()
                    };
                    *identity = "std/bool::JsonEncode".into();
                }
                13 => arguments.push(arguments[0].clone()),
                14 => {
                    *type_ref = CoreType::Named {
                        name: "Bool".into(),
                        arguments: vec![],
                    };
                }
                15 => {
                    arguments[0] = CoreExpr::Unit {
                        origin: core_expr_origin(&arguments[0]).clone(),
                    }
                }
                16 => deferred_evidence_parameters.push(CoreType::Hole),
                17 => deferred_evidence_type_constructor_parameters.push("F".into()),
                18 => *evidence_argument_index = Some(0),
                _ => {
                    *trait_dispatch = Some(seseragi_lowering::CoreTraitDispatch {
                        trait_identity: "std/prelude::Show".into(),
                        method: "show".into(),
                        method_contract: None,
                    })
                }
            }
            let error = emit_rust_module(&bad, "main").unwrap_err();
            assert_eq!(error.code, "SES-RUST-0001");
            assert_eq!(
                error.origin.as_ref(),
                Some(&origin),
                "{expression} mutation{mutation}: {error:?}"
            );
        }
    }
}

#[test]
fn primitive_json_dictionaries_stay_out_of_program_wasm_even_if_ignored() {
    for source in [
        "import * as json from \"std/json\"\npub fn probe value: Int -> Int = {let saved: Int -> String = json.encodeString; value}\npub fn main -> Int = 0\n",
        "fn ignore<A> _value: A -> Int where JsonEncode<A> = 1\npub fn probe value: Int -> Int = ignore value\npub fn main -> Int = 0\n",
        "fn ignore<A> _value: A -> Int where JsonDecode<A> = 1\npub fn probe value: Int -> Int = ignore value\npub fn main -> Int = 0\n",
    ] {
        let core=checked(source);
        emit_rust_module(&core,"main").unwrap();
        let error=emit_program_wasm(&[&core],"native/json-codecs",&["probe".into()]).unwrap_err();
        assert_eq!(error.code,"SES-RUST-0001");
        assert!(error.message.contains("require a target-specific"),"{error:?}");
    }
}

#[test]
fn all_twelve_primitive_dictionaries_reject_same_kind_nominals_and_forged_children() {
    for value_type in [
        "Bool",
        "String",
        "Int",
        "decimal.Decimal",
        "Unit",
        "json.Json",
    ] {
        for encode in [true, false] {
            let (parameter, result, expression) = if encode {
                (
                    format!("value: {value_type}"),
                    "String".to_owned(),
                    "json.encodeString value",
                )
            } else {
                (
                    "value: String".into(),
                    format!("Either<json.JsonReadError, {value_type}>"),
                    "json.decodeString value",
                )
            };
            let source=format!("// 雪🙂\nimport * as json from \"std/json\"\nimport * as decimal from \"std/decimal\"\ntype Pretender = | Pretender\npub fn probe {parameter} -> {result} = {expression}\npub fn main -> Int = 0\n");
            let core = checked(&source);
            emit_rust_module(&core, "main").unwrap();
            let index = core
                .functions
                .iter()
                .position(|f| f.symbol.ends_with("::probe"))
                .unwrap();
            let origin = core_expr_origin(&core.functions[index].body).clone();
            for mutation in 0..3 {
                let mut bad = core.clone();
                let CoreExpr::Call {
                    evidence,
                    arguments,
                    type_ref,
                    ..
                } = &mut bad.functions[index].body
                else {
                    panic!()
                };
                match mutation {
                    0 => {
                        let pretend = CoreType::Named {
                            name: "native/json-codecs::Pretender".into(),
                            arguments: vec![],
                        };
                        evidence[0].constraint.arguments[0] = pretend.clone();
                        if encode {
                            let CoreExpr::Variable { type_ref, .. } = &mut arguments[0] else {
                                panic!()
                            };
                            *type_ref = pretend;
                        } else {
                            let CoreType::Named { arguments, .. } = type_ref else {
                                panic!()
                            };
                            arguments[1] = pretend;
                        }
                    }
                    1 => {
                        let CoreInstanceEvidence::Standard { type_arguments, .. } =
                            &mut evidence[0].evidence
                        else {
                            panic!()
                        };
                        type_arguments.push(CoreType::Hole);
                    }
                    _ => {
                        let child = evidence[0].clone();
                        let CoreInstanceEvidence::Standard {
                            evidence_arguments, ..
                        } = &mut evidence[0].evidence
                        else {
                            panic!()
                        };
                        evidence_arguments.push(child);
                    }
                }
                let error = emit_rust_module(&bad, "main").unwrap_err();
                assert_eq!(error.origin.as_ref(), Some(&origin));
                assert!(
                    error.message.contains("unproven standard JSON codec"),
                    "{value_type}:{encode}:{mutation}:{error:?}"
                );
            }
        }
    }
}

#[test]
fn public_decode_and_read_error_constructors_check_owner_type_and_source_stages() {
    for (parameter, result, expression) in [
        (
            "String",
            "json.DecodeErrorKind",
            "json.ExpectedJsonType value",
        ),
        (
            "String",
            "json.DecodeErrorKind",
            "json.MissingJsonField value",
        ),
        (
            "String",
            "json.DecodeErrorKind",
            "json.UnknownJsonField value",
        ),
        (
            "String",
            "json.DecodeErrorKind",
            "json.UnknownJsonTag value",
        ),
        (
            "String",
            "json.DecodeErrorKind",
            "json.InvalidJsonValue value",
        ),
        (
            "json.JsonParseError",
            "json.JsonReadError",
            "json.JsonSyntaxFailure value",
        ),
        (
            "json.DecodeError",
            "json.JsonReadError",
            "json.JsonDecodeFailure value",
        ),
    ] {
        let core=checked(&format!("// 雪🙂\nimport * as json from \"std/json\"\npub fn probe value: {parameter} -> {result} = {expression}\npub fn main -> Int = 0\n"));
        emit_rust_module(&core, "main").unwrap();
        let index = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::probe"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[index].body).clone();
        for mutation in 0..8 {
            let mut bad = core.clone();
            let CoreExpr::Call {
                arguments,
                type_ref,
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
                        name: "JsonReadError".into(),
                        canonical: "fake::JsonReadError".into(),
                        arguments: vec![],
                    }
                }
                2 => arguments.push(arguments[0].clone()),
                3 => {
                    arguments[0] = CoreExpr::Unit {
                        origin: core_expr_origin(&arguments[0]).clone(),
                    }
                }
                4 => deferred_evidence_parameters.push(CoreType::Hole),
                5 => deferred_evidence_type_constructor_parameters.push("F".into()),
                6 => *evidence_argument_index = Some(0),
                _ => {
                    *trait_dispatch = Some(seseragi_lowering::CoreTraitDispatch {
                        trait_identity: "std/prelude::Show".into(),
                        method: "show".into(),
                        method_contract: None,
                    })
                }
            }
            let error = emit_rust_module(&bad, "main").unwrap_err();
            assert_eq!(error.origin.as_ref(), Some(&origin));
        }
        let saved=checked(&format!("import * as json from \"std/json\"\npub fn probe value: Int -> Int = {{let saved: {parameter} -> {result} = {};value}}\npub fn main -> Int = 0\n",expression.strip_suffix(" value").unwrap()));
        let error =
            emit_program_wasm(&[&saved], "native/json-codecs", &["probe".into()]).unwrap_err();
        assert!(
            error.message.contains("requires a target-specific"),
            "{error:?}"
        );
    }
}
