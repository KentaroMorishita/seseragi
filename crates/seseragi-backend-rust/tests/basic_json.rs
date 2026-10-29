use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreType};
fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new("json-雪🙂.ssrg", "native/json", source))
        .unwrap_or_else(|e| panic!("{source}\n{e:?}"))
        .core_ir
}
#[test]
fn json_checked_source_keeps_data_constructors_paths_exact_decimal_and_saved_functions() {
    let core = checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-json.ssrg"
    ));
    let artifact = emit_rust_module(&core, "main").unwrap_or_else(|e| panic!("{e:?}"));
    assert!(artifact.metadata.capabilities.iter().any(|c| c == "json"));
}
#[test]
fn json_refuses_forged_owner_kind_argument_result_evidence_and_source_stages() {
    let selected = checked("pub fn probe -> Bool = 1 == 1");
    let CoreExpr::Binary {
        evidence: extra, ..
    } = &selected.functions[0].body
    else {
        panic!()
    };
    for (parameter, result, expression) in [
        (
            "value: String",
            "Either<json.JsonParseError, json.Json>",
            "json.parse value",
        ),
        ("value: json.Json", "String", "json.stringify value"),
        ("value: String", "json.Json", "json.JsonString value"),
    ] {
        let core=checked(&format!("// 雪🙂\nimport * as json from \"std/json\"\nfn probe {parameter} -> {result} = {expression}\npub fn main -> Int = 0\n"));
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
                panic!("{expression}:{:#?}", bad.functions[index].body)
            };
            match mutation {
                0 => *type_ref = CoreType::Hole,
                1 => {
                    *type_ref = CoreType::Named {
                        name: "std/json::Json".into(),
                        arguments: vec![CoreType::Hole],
                    }
                }
                2 => {
                    *type_ref = CoreType::ExternalNamed {
                        name: "Json".into(),
                        canonical: "fake::Json".into(),
                        arguments: vec![],
                    }
                }
                3 => arguments.clear(),
                4 => arguments.push(arguments[0].clone()),
                5 => evidence.push(extra[0].clone()),
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
            let error = emit_rust_module(&bad, "main").expect_err("forgedJSONABI");
            assert_eq!(error.code, "SES-RUST-0001", "{expression}:{mutation}");
            assert_eq!(error.origin.unwrap(), origin, "{expression}:{mutation}");
        }
    }
}
#[test]
fn json_saved_references_do_not_expand_program_wasm() {
    for (name, ty) in [
        ("parse", "String -> Either<json.JsonParseError, json.Json>"),
        ("stringify", "json.Json -> String"),
        ("JsonNull", "json.Json"),
        ("JsonBool", "Bool -> json.Json"),
        ("JsonNumber", "decimal.Decimal -> json.Json"),
        ("JsonString", "String -> json.Json"),
        ("JsonArray", "Array<json.Json> -> json.Json"),
        ("JsonObject", "maps.Map<String, json.Json> -> json.Json"),
        ("JsonField", "String -> json.JsonPathSegment"),
        ("JsonIndex", "Int -> json.JsonPathSegment"),
        (
            "InvalidJsonSyntax",
            "{ message: String, offset: Int } -> json.JsonParseError",
        ),
        (
            "DuplicateJsonField",
            "{ field: String, path: Array<json.JsonPathSegment> } -> json.JsonParseError",
        ),
    ] {
        let core=checked(&format!("import * as json from \"std/json\"\nimport * as decimal from \"std/decimal\"\nimport * as maps from \"std/map\"\npub fn probe value: Int -> Int = {{let saved: {ty} = json.{name}; value}}\npub fn main -> Int = 0\n"));
        emit_rust_module(&core, "main").unwrap_or_else(|e| panic!("{name}:{e:?}"));
        let error = emit_program_wasm(&[&core], "native/json", &["probe".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(
            error.message.contains("requires a target-specific"),
            "{name}:{error:?}"
        );
        assert_eq!(error.origin.unwrap().source, "json-雪🙂.ssrg");
    }
}

#[test]
fn json_error_constructor_inputs_keep_record_width_and_refuse_missing_optional_or_wrong_fields() {
    for (name, fields) in [
        (
            "InvalidJsonSyntax",
            "message: String, offset: Int, extra: Bool",
        ),
        (
            "DuplicateJsonField",
            "field: String, path: Array<json.JsonPathSegment>, extra: Bool",
        ),
    ] {
        let core=checked(&format!("import * as json from \"std/json\"\nfn make value: {{ {fields} }} -> json.JsonParseError = json.{name} value\npub fn main -> Int = 0\n"));
        emit_rust_module(&core, "main").unwrap();
        let index = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::make"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[index].body).clone();
        for mutation in 0..5 {
            let mut bad = core.clone();
            let CoreExpr::Call { arguments, .. } = &mut bad.functions[index].body else {
                panic!()
            };
            let CoreExpr::Variable { type_ref, .. } = &mut arguments[0] else {
                panic!()
            };
            let CoreType::Record { fields, .. } = type_ref else {
                panic!()
            };
            let required = fields
                .iter()
                .position(|f| {
                    f.name
                        == if name == "InvalidJsonSyntax" {
                            "offset"
                        } else {
                            "path"
                        }
                })
                .unwrap();
            match mutation {
                0 => {
                    fields.remove(required);
                }
                1 => {
                    fields[required].type_ref = CoreType::Named {
                        name: "std/prelude::Bool".into(),
                        arguments: vec![],
                    }
                }
                2 => fields[required].optional = true,
                3 => fields[required].name = "wrong".into(),
                _ => fields.push(seseragi_lowering::CoreRecordField {
                    name: "unproven".into(),
                    optional: false,
                    type_ref: CoreType::Hole,
                }),
            }
            let error = emit_rust_module(&bad, "main").expect_err("unprovenJSONerrorrecordinput");
            assert_eq!(error.code, "SES-RUST-0001", "{name}:{mutation}");
            assert_eq!(error.origin.unwrap(), origin, "{name}:{mutation}");
        }
    }
}
