use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreInstanceEvidence, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "map-upsert-雪🙂.ssrg",
        "native/update",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:?}"))
    .core_ir
}
fn source(name: &str) -> String {
    let signature = match name {
        "upsert" => "key: String -> callback: (Maybe<Int> -> Int) -> values: maps.Map<String,Int> -> maps.Map<String,Int> = maps.upsert key callback values",
        "mergeWith" => "callback: (Int -> Int -> Int) -> right: maps.Map<String,Int> -> values: maps.Map<String,Int> -> maps.Map<String,Int> = maps.mergeWith callback right values",
        _ => "callback: (Int -> Int -> Int) -> key: (String -> Int) -> values: maps.Map<String,Int> -> maps.Map<Int,Int> = maps.mapKeysWith callback key values",
    };
    format!("// 雪🙂\nimport * as maps from \"std/map\"\nfn operation {signature}\npub fn main -> Int = 0\n")
}

#[test]
fn map_update_functions_keep_generic_partials_and_only_require_output_key_dictionaries() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-map-upsert.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-map-upsert-effects.ssrg"),
    ] {
        emit_rust_module(&checked(source), "main").unwrap();
    }
    for name in ["upsert", "mergeWith", "mapKeysWith"] {
        emit_rust_module(&checked(&source(name)), "main").unwrap();
    }
}

#[test]
fn map_update_functions_refuse_forged_shapes_stages_and_wrong_output_key_dictionaries() {
    let string_core = checked(&source("mergeWith"));
    let int_core = checked(&source("mapKeysWith"));
    let selected = |core: &seseragi_lowering::CoreModule| {
        let CoreExpr::Call { evidence, .. } = &core
            .functions
            .iter()
            .find(|function| function.symbol.ends_with("::operation"))
            .unwrap()
            .body
        else {
            panic!()
        };
        evidence.clone()
    };
    let strings = selected(&string_core);
    let ints = selected(&int_core);
    for name in ["upsert", "mergeWith", "mapKeysWith"] {
        let core = checked(&source(name));
        let index = core
            .functions
            .iter()
            .position(|function| function.symbol.ends_with("::operation"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[index].body).clone();
        for mutation in 0..21 {
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
            let named = |name: &str| CoreType::Named {
                name: format!("std/prelude::{name}"),
                arguments: vec![],
            };
            match mutation {
                0 => *type_ref = CoreType::Hole,
                1 => {
                    *type_ref = CoreType::Named {
                        name: "fake::Map".into(),
                        arguments: vec![named("Int"), named("Int")],
                    }
                }
                2 => arguments.clear(),
                3 => arguments.push(arguments[0].clone()),
                4..=6 => {
                    let index = mutation - 4;
                    arguments[index] = CoreExpr::Unit {
                        origin: core_expr_origin(&arguments[index]).clone(),
                    };
                }
                7 => evidence.clear(),
                8 => evidence.swap(0, 1),
                9 => {
                    *evidence = if name == "mapKeysWith" {
                        strings.clone()
                    } else {
                        ints.clone()
                    }
                }
                10 => {
                    *trait_dispatch = Some(seseragi_lowering::CoreTraitDispatch {
                        trait_identity: "std/prelude::Hash".into(),
                        method: "hash".into(),
                        method_contract: None,
                    })
                }
                11 => deferred_evidence_parameters.push(CoreType::Hole),
                12 => deferred_evidence_type_constructor_parameters.push("F".into()),
                13 => *evidence_argument_index = Some(0),
                14..=16 => {
                    let CoreExpr::Variable { type_ref, .. } = &mut arguments[2] else {
                        panic!()
                    };
                    *type_ref = CoreType::Named {
                        name: if mutation == 14 {
                            "fake::Map"
                        } else {
                            "std/map::Map"
                        }
                        .into(),
                        arguments: if mutation == 15 {
                            vec![named("String")]
                        } else {
                            vec![named("String"), named("Bool")]
                        },
                    };
                }
                17 => {
                    *type_ref = CoreType::Named {
                        name: "std/map::Map".into(),
                        arguments: vec![
                            named(if name == "mapKeysWith" {
                                "Int"
                            } else {
                                "String"
                            }),
                            named("Bool"),
                        ],
                    }
                }
                18 => {
                    *type_ref = CoreType::Named {
                        name: "std/map::Map".into(),
                        arguments: vec![named("Bool"), named("Int")],
                    }
                }
                19 => evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 99 },
                _ => {
                    let extra = evidence[0].clone();
                    let CoreInstanceEvidence::Standard {
                        evidence_arguments, ..
                    } = &mut evidence[1].evidence
                    else {
                        panic!()
                    };
                    evidence_arguments.push(extra);
                }
            }
            let error = emit_rust_module(&bad, "main").expect_err("unproven Map update contract");
            assert_eq!(error.code, "SES-RUST-0001", "{name}/{mutation}");
            assert_eq!(error.origin.unwrap(), origin, "{name}/{mutation}");
        }
    }
}

#[test]
fn unused_map_update_functions_stay_out_of_program_wasm() {
    for (name, signature) in [
        ("upsert", "String -> (Maybe<Int> -> Int) -> maps.Map<String,Int> -> maps.Map<String,Int>"),
        ("mergeWith", "(Int -> Int -> Int) -> maps.Map<String,Int> -> maps.Map<String,Int> -> maps.Map<String,Int>"),
        ("mapKeysWith", "(Int -> Int -> Int) -> (String -> Int) -> maps.Map<String,Int> -> maps.Map<Int,Int>"),
    ] {
        let source = format!("import * as maps from \"std/map\"\npub fn probe value: Int -> Int = {{ let saved: {signature} = maps.{name}; value }}\n");
        let error = emit_program_wasm(&[&checked(&source)], "native/update", &["probe".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(error.message.contains("native collection requires a target-specific ABI proof"), "{name}: {error}");
        assert_eq!(error.origin.unwrap().source, "map-upsert-雪🙂.ssrg");
    }
}
