use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreInstanceEvidence, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "collections-雪🙂.ssrg",
        "native/collections",
        source,
    ))
    .unwrap_or_else(|e| panic!("{source}\n{e:?}"))
    .core_ir
}

#[test]
fn basic_set_and_map_entries_accept_the_original_checked_sources() {
    for (source, capability) in [
        (
            include_str!("../../../examples/spec/fixtures/rust-backend/basic-set.ssrg"),
            "set",
        ),
        (
            include_str!("../../../examples/spec/fixtures/rust-backend/basic-map-entries.ssrg"),
            "map",
        ),
    ] {
        let core = checked(source);
        let artifact = emit_rust_module(&core, "main").unwrap_or_else(|e| panic!("{e:?}"));
        assert!(artifact
            .metadata
            .capabilities
            .iter()
            .any(|c| c == capability));
    }
}

#[test]
fn generic_set_partial_preserves_exact_caller_dictionary_and_remaining_stage() {
    let core = checked("import * as sets from \"std/set\"\nfn add<A> value: A -> sets.Set<A> -> sets.Set<A> where Eq<A>, Hash<A> = sets.insert value\npub fn main -> Int = 0\n");
    emit_rust_module(&core, "main").unwrap_or_else(|e| panic!("{e:?}\n{core:#?}"));
    let index = core
        .functions
        .iter()
        .position(|f| f.symbol.ends_with("::add"))
        .unwrap();
    let origin = core_expr_origin(&core.functions[index].body).clone();
    for mutation in 0..6 {
        let mut bad = core.clone();
        let CoreExpr::Call {
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            ..
        } = &mut bad.functions[index].body
        else {
            panic!()
        };
        assert_eq!(deferred_evidence_parameters.len(), 1);
        match mutation {
            0 => deferred_evidence_parameters.push(CoreType::Hole),
            1 => deferred_evidence_parameters[0] = CoreType::Hole,
            2 => {
                let CoreType::ExternalNamed { canonical, .. } =
                    &mut deferred_evidence_parameters[0]
                else {
                    panic!()
                };
                *canonical = "fake::Set".into();
            }
            3 => {
                let CoreType::ExternalNamed { arguments, .. } =
                    &mut deferred_evidence_parameters[0]
                else {
                    panic!()
                };
                arguments[0] = CoreType::Named {
                    name: "std/prelude::Bool".into(),
                    arguments: vec![],
                };
            }
            4 => *evidence_argument_index = Some(0),
            _ => deferred_evidence_type_constructor_parameters.push("F".into()),
        }
        let e = emit_rust_module(&bad, "main").expect_err("forged Set suffix");
        assert_eq!(e.code, "SES-RUST-0001");
        assert_eq!(e.origin.unwrap(), origin);
    }
}

#[test]
fn set_refuses_forged_owner_kind_element_evidence_and_source_stages() {
    let core = checked("// 雪🙂\nimport * as sets from \"std/set\"\nfn has values: sets.Set<String> -> Bool = sets.contains \"snow\" values\npub fn main -> Int = 0\n");
    emit_rust_module(&core, "main").unwrap();
    let index = core
        .functions
        .iter()
        .position(|f| f.symbol.ends_with("::has"))
        .unwrap();
    let origin = core_expr_origin(&core.functions[index].body).clone();
    for mutation in 0..16 {
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
                *type_ref = CoreType::Named {
                    name: "std/prelude::Int".into(),
                    arguments: vec![],
                }
            }
            2 => evidence.clear(),
            3 => evidence[0].constraint.arguments[0] = CoreType::Hole,
            4 => evidence.swap(0, 1),
            5 => {
                let CoreInstanceEvidence::Standard { identity, .. } = &mut evidence[1].evidence
                else {
                    panic!()
                };
                *identity = "std/int::Hash".into();
            }
            6 => arguments.swap(0, 1),
            7 => {
                arguments.pop();
            }
            8 => arguments.push(arguments[0].clone()),
            9 => {
                *trait_dispatch = Some(seseragi_lowering::CoreTraitDispatch {
                    trait_identity: "std/prelude::Eq".into(),
                    method: "eq".into(),
                    method_contract: None,
                })
            }
            10 => deferred_evidence_parameters.push(CoreType::Hole),
            11 => deferred_evidence_type_constructor_parameters.push("F".into()),
            12 => *evidence_argument_index = Some(0),
            13 => {
                let CoreExpr::Variable { type_ref, .. } = &mut arguments[1] else {
                    panic!()
                };
                *type_ref = CoreType::ExternalNamed {
                    name: "Set".into(),
                    canonical: "fake::Set".into(),
                    arguments: vec![],
                };
            }
            14 => evidence[1].evidence = CoreInstanceEvidence::Parameter { index: 99 },
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
        let e = emit_rust_module(&bad, "main").expect_err("forged Set ABI");
        assert_eq!(e.code, "SES-RUST-0001", "mutation{mutation}");
        assert_eq!(e.origin.unwrap(), origin, "mutation{mutation}");
    }
}

#[test]
fn map_contains_and_entries_refuse_forged_pairs_owner_evidence_and_source_stages() {
    for (name, parameter, result) in [
        ("containsKey", "\"snow\" ", "Bool"),
        ("entries", "", "Array<(String, Int)>"),
    ] {
        let core = checked(&format!("// 雪🙂\nimport * as maps from \"std/map\"\nfn operation values: maps.Map<String, Int> -> {result} = maps.{name} {parameter}values\npub fn main -> Int = 0\n"));
        emit_rust_module(&core, "main").unwrap();
        let index = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::operation"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[index].body).clone();
        for mutation in 0..6 {
            let mut bad = core.clone();
            let CoreExpr::Call {
                arguments,
                type_ref,
                evidence,
                trait_dispatch,
                deferred_evidence_parameters,
                ..
            } = &mut bad.functions[index].body
            else {
                panic!()
            };
            match mutation {
                0 => *type_ref = CoreType::Hole,
                1 => {
                    *type_ref = CoreType::Named {
                        name: "std/prelude::Array".into(),
                        arguments: vec![CoreType::Tuple {
                            elements: vec![
                                CoreType::Named {
                                    name: "std/prelude::Int".into(),
                                    arguments: vec![],
                                },
                                CoreType::Named {
                                    name: "std/prelude::String".into(),
                                    arguments: vec![],
                                },
                            ],
                        }],
                    }
                }
                2 => {
                    let CoreExpr::Variable { type_ref, .. } = arguments.last_mut().unwrap() else {
                        panic!()
                    };
                    *type_ref = CoreType::ExternalNamed {
                        name: "Map".into(),
                        canonical: "fake::Map".into(),
                        arguments: vec![],
                    };
                }
                3 => {
                    if name == "containsKey" {
                        evidence.clear();
                    } else {
                        let with_evidence=checked("import * as maps from \"std/map\"\nfn has values: maps.Map<String, Int> -> Bool = maps.containsKey \"snow\" values\npub fn main -> Int = 0\n");
                        let CoreExpr::Call {
                            evidence: selected, ..
                        } = &with_evidence
                            .functions
                            .iter()
                            .find(|f| f.symbol.ends_with("::has"))
                            .unwrap()
                            .body
                        else {
                            panic!()
                        };
                        *evidence = selected.clone();
                    }
                }
                4 => {
                    *trait_dispatch = Some(seseragi_lowering::CoreTraitDispatch {
                        trait_identity: "std/prelude::Eq".into(),
                        method: "eq".into(),
                        method_contract: None,
                    })
                }
                _ => deferred_evidence_parameters.push(CoreType::Hole),
            }
            let error = emit_rust_module(&bad, "main")
                .expect_err("forged Map enumeration or membership ABI");
            assert_eq!(error.code, "SES-RUST-0001", "{name}:{mutation}");
            assert_eq!(error.origin.unwrap(), origin, "{name}:{mutation}");
        }
    }
}

#[test]
fn collection_references_do_not_expand_program_wasm() {
    for (module, name, ty) in [
        ("set", "empty", "Unit -> collection.Set<String>"),
        ("set", "singleton", "String -> collection.Set<String>"),
        (
            "set",
            "contains",
            "String -> collection.Set<String> -> Bool",
        ),
        (
            "set",
            "insert",
            "String -> collection.Set<String> -> collection.Set<String>",
        ),
        (
            "set",
            "remove",
            "String -> collection.Set<String> -> collection.Set<String>",
        ),
        ("set", "toArray", "collection.Set<String> -> Array<String>"),
        ("set", "size", "collection.Set<String> -> Int"),
        ("set", "isEmpty", "collection.Set<String> -> Bool"),
        (
            "map",
            "containsKey",
            "String -> collection.Map<String, Int> -> Bool",
        ),
        (
            "map",
            "entries",
            "collection.Map<String, Int> -> Array<(String, Int)>",
        ),
    ] {
        let core = checked(&format!("import * as collection from \"std/{module}\"\npub fn probe value: Int -> Int = {{ let saved: {ty} = collection.{name}; value }}\npub fn main -> Int = 0\n"));
        emit_rust_module(&core, "main").unwrap_or_else(|e| panic!("{name}: {e:?}"));
        let e = emit_program_wasm(&[&core], "native/collections", &["probe".into()]).unwrap_err();
        assert_eq!(e.code, "SES-RUST-0001");
        assert!(
            e.message.contains("requires a target-specific"),
            "{name}: {e:?}"
        );
        assert_eq!(e.origin.unwrap().source, "collections-雪🙂.ssrg");
    }
}
