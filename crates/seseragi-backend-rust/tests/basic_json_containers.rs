use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreInstanceEvidence, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "container-雪🙂.ssrg",
        "native/json-containers",
        source,
    ))
    .unwrap_or_else(|e| panic!("{source}\n{e:?}"))
    .core_ir
}
#[test]
fn standard_container_codecs_keep_nested_generic_saved_methods_and_key_dictionaries() {
    let module = checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-json-containers.ssrg"
    ));
    emit_rust_module(&module, "main").unwrap_or_else(|e| panic!("{e:?}"));
}

const CONTAINERS: [&str; 8] = [
    "Array<Int>",
    "List<Int>",
    "Maybe<Int>",
    "Either<Int,String>",
    "(Int,String)",
    "maps.Map<Int,String>",
    "sets.Set<Int>",
    "{ a: Int, b?: String }",
];

fn ignored_codec(value_type: &str, method: &str) -> seseragi_lowering::CoreModule {
    checked(&format!(
        "// 雪🙂\nimport * as maps from \"std/map\"\nimport * as sets from \"std/set\"\nfn ignore<A> _value: A -> Int where {method}<A> = 1\npub fn probe value: {value_type} -> Int = ignore value\npub fn main -> Int = 0\n"
    ))
}

#[test]
fn all_sixteen_factories_refuse_forged_nested_dictionaries_at_the_original_utf8_span() {
    for value_type in CONTAINERS {
        for method in ["JsonEncode", "JsonDecode"] {
            let core = ignored_codec(value_type, method);
            emit_rust_module(&core, "main").unwrap();
            let index = core
                .functions
                .iter()
                .position(|f| f.symbol.ends_with("::probe"))
                .unwrap();
            let origin = core_expr_origin(&core.functions[index].body).clone();
            for mutation in 0..15 {
                let mut bad = core.clone();
                let CoreExpr::Call { evidence, .. } = &mut bad.functions[index].body else {
                    panic!()
                };
                let selected = &mut evidence[0];
                if mutation == 0 {
                    selected.constraint.trait_identity = Some(format!("fake::{method}"));
                } else if mutation == 1 {
                    selected.constraint.arguments[0] = CoreType::Hole;
                } else {
                    let CoreInstanceEvidence::Standard {
                        identity,
                        type_arguments,
                        evidence_arguments,
                    } = &mut selected.evidence
                    else {
                        panic!("{value_type}:{method}")
                    };
                    match mutation {
                        2 => *identity = format!("fake::{method}"),
                        3 => type_arguments.clear(),
                        4 => type_arguments.push(CoreType::Hole),
                        5 => type_arguments[0] = CoreType::Hole,
                        6 => evidence_arguments.clear(),
                        7 => evidence_arguments.push(evidence_arguments[0].clone()),
                        8 => {
                            evidence_arguments[0].constraint.trait_identity =
                                Some("fake::JsonEncode".into());
                        }
                        9 => {
                            evidence_arguments[0].constraint.arguments[0] = CoreType::Named {
                                name: "Bool".into(),
                                arguments: vec![],
                            };
                        }
                        10 => evidence_arguments[0].constraint.name = "Show".into(),
                        11 => {
                            evidence_arguments[0].evidence =
                                CoreInstanceEvidence::Parameter { index: 99 };
                        }
                        12 | 13 => {
                            let CoreInstanceEvidence::Standard {
                                identity,
                                type_arguments,
                                ..
                            } = &mut evidence_arguments[0].evidence
                            else {
                                panic!()
                            };
                            if mutation == 12 {
                                *identity = "fake::JsonEncode".into();
                            } else {
                                type_arguments.push(CoreType::Hole);
                            }
                        }
                        _ => {
                            type_arguments[0] = CoreType::Named {
                                name: "Array".into(),
                                arguments: vec![],
                            };
                        }
                    }
                }
                let error = emit_rust_module(&bad, "main")
                    .expect_err(&format!("{value_type}:{method}:{mutation}"));
                assert_eq!(error.code, "SES-RUST-0001");
                assert_eq!(
                    error.origin.as_ref(),
                    Some(&origin),
                    "{value_type}:{method}:{mutation}:{error:?}"
                );
            }
        }
    }
}

#[test]
fn map_and_set_decoders_require_eq_hash_then_the_exact_child_codecs() {
    for value_type in ["maps.Map<Int,String>", "sets.Set<Int>"] {
        let core = ignored_codec(value_type, "JsonDecode");
        let index = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::probe"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[index].body).clone();
        for mutation in 0..3 {
            let mut bad = core.clone();
            let CoreExpr::Call { evidence, .. } = &mut bad.functions[index].body else {
                panic!()
            };
            let CoreInstanceEvidence::Standard {
                evidence_arguments, ..
            } = &mut evidence[0].evidence
            else {
                panic!()
            };
            match mutation {
                0 => evidence_arguments.swap(0, 1),
                1 => evidence_arguments.swap(1, 2),
                _ => {
                    let CoreInstanceEvidence::Standard { identity, .. } =
                        &mut evidence_arguments[1].evidence
                    else {
                        panic!()
                    };
                    *identity = "std/bool::Hash".into();
                }
            }
            let error = emit_rust_module(&bad, "main").unwrap_err();
            assert_eq!(error.origin.as_ref(), Some(&origin));
        }
    }
}

#[test]
fn container_dictionaries_stay_out_of_program_wasm_even_when_ignored() {
    for value_type in CONTAINERS {
        for method in ["JsonEncode", "JsonDecode"] {
            let core = checked(&format!(
                "import * as maps from \"std/map\"\nimport * as sets from \"std/set\"\nfn ignore<A> value: Int -> Int where {method}<A> = value\npub fn probe value: Int -> Int = ignore<{value_type}> value\npub fn main -> Int = 0\n"
            ));
            emit_rust_module(&core, "main").unwrap();
            let error = emit_program_wasm(&[&core], "native/json-containers", &["probe".into()])
                .unwrap_err();
            assert_eq!(error.code, "SES-RUST-0001");
            assert!(
                error.message.contains("require a target-specific"),
                "{error:?}"
            );
        }
    }
}
