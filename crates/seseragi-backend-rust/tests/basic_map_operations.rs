use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "map-operations-雪🙂.ssrg",
        "native/map",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:?}"))
    .core_ir
}

#[test]
fn map_callbacks_keep_original_saved_generic_functions_and_changed_value_types() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-map-operations.ssrg"),
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/basic-map-operation-effects.ssrg"
        ),
    ] {
        let artifact = emit_rust_module(&checked(source), "main").unwrap();
        assert!(artifact
            .metadata
            .capabilities
            .iter()
            .any(|capability| capability == "map"));
    }
}

#[test]
fn map_callbacks_reject_forged_keys_values_callbacks_results_evidence_and_stages() {
    let other = checked("pub fn probe -> Bool = 1 == 1\n");
    let CoreExpr::Binary {
        evidence: selected, ..
    } = &other.functions[0].body
    else {
        panic!()
    };
    for (name, callback, output) in [
        ("filter", "String -> Int -> Bool", "maps.Map<String,Int>"),
        ("mapValues", "Int -> String", "maps.Map<String,String>"),
    ] {
        let source = format!("// 雪🙂\nimport * as maps from \"std/map\"\nfn operation callback: ({callback}) -> values: maps.Map<String,Int> -> {output} = maps.{name} callback values\npub fn main -> Int = 0\n");
        let core = checked(&source);
        emit_rust_module(&core, "main").unwrap();
        let index = core
            .functions
            .iter()
            .position(|function| function.symbol.ends_with("::operation"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[index].body).clone();
        for mutation in 0..17 {
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
            let string = CoreType::Named {
                name: "std/prelude::String".into(),
                arguments: vec![],
            };
            let boolean = CoreType::Named {
                name: "std/prelude::Bool".into(),
                arguments: vec![],
            };
            match mutation {
                0 => *type_ref = CoreType::Hole,
                1 => {
                    *type_ref = CoreType::Named {
                        name: "fake::Map".into(),
                        arguments: vec![string.clone(), string.clone()],
                    }
                }
                2 => arguments.clear(),
                3 => arguments.push(arguments[0].clone()),
                4 => {
                    arguments[0] = CoreExpr::Unit {
                        origin: core_expr_origin(&arguments[0]).clone(),
                    }
                }
                5 => {
                    arguments[1] = CoreExpr::Unit {
                        origin: core_expr_origin(&arguments[1]).clone(),
                    }
                }
                6 => evidence.push(selected[0].clone()),
                7 => {
                    *trait_dispatch = Some(seseragi_lowering::CoreTraitDispatch {
                        trait_identity: "std/prelude::Eq".into(),
                        method: "eq".into(),
                        method_contract: None,
                    })
                }
                8 => deferred_evidence_parameters.push(CoreType::Hole),
                9 => deferred_evidence_type_constructor_parameters.push("F".into()),
                10 => *evidence_argument_index = Some(0),
                11 | 12 => {
                    *type_ref = CoreType::Named {
                        name: "std/map::Map".into(),
                        arguments: if mutation == 11 {
                            vec![boolean.clone(), string.clone()]
                        } else {
                            vec![string.clone(), boolean.clone()]
                        },
                    };
                }
                13 => {
                    let CoreExpr::Variable { type_ref, .. } = &mut arguments[0] else {
                        panic!()
                    };
                    *type_ref = CoreType::Function {
                        parameter: Box::new(boolean.clone()),
                        result: Box::new(boolean.clone()),
                    };
                }
                _ => {
                    let CoreExpr::Variable { type_ref, .. } = &mut arguments[1] else {
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
                            vec![string.clone()]
                        } else {
                            vec![boolean.clone(), string.clone()]
                        },
                    };
                }
            }
            let error = emit_rust_module(&bad, "main").expect_err("unproven Map callback contract");
            assert_eq!(error.code, "SES-RUST-0001", "{name}/{mutation}");
            assert_eq!(error.origin.unwrap(), origin, "{name}/{mutation}");
        }
    }
}

#[test]
fn unused_map_callback_functions_do_not_expand_program_wasm() {
    for (name, signature) in [
        (
            "filter",
            "(String -> Int -> Bool) -> maps.Map<String,Int> -> maps.Map<String,Int>",
        ),
        (
            "mapValues",
            "(Int -> String) -> maps.Map<String,Int> -> maps.Map<String,String>",
        ),
    ] {
        let source = format!("import * as maps from \"std/map\"\npub fn probe value: Int -> Int = {{ let saved: {signature} = maps.{name}; value }}\n");
        let error =
            emit_program_wasm(&[&checked(&source)], "native/map", &["probe".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(
            error
                .message
                .contains("native collection requires a target-specific ABI proof"),
            "{name}: {error}"
        );
        assert_eq!(error.origin.unwrap().source, "map-operations-雪🙂.ssrg");
    }
}
