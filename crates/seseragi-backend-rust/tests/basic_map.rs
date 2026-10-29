use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreInstanceEvidence, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new("map-雪🙂.ssrg", "native/map", source))
        .unwrap_or_else(|error| panic!("{source}\n{error:?}"))
        .core_ir
}

#[test]
fn basic_map_accepts_immutable_snapshots_selected_keys_generic_and_saved_functions() {
    let core = checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-map.ssrg"
    ));
    let artifact = emit_rust_module(&core, "main").unwrap();
    assert!(artifact.metadata.capabilities.iter().any(|c| c == "map"));
}

#[test]
fn generic_map_partial_keeps_its_checked_remaining_source_stage() {
    let core = checked("import * as maps from \"std/map\"\nfn put<K, V> key: K -> value: V -> maps.Map<K, V> -> maps.Map<K, V> where Eq<K>, Hash<K> = maps.insert key value\npub fn main -> Int = 0\n");
    emit_rust_module(&core, "main").unwrap_or_else(|error| {
        let put = core
            .functions
            .iter()
            .find(|f| f.symbol.ends_with("::put"))
            .unwrap();
        panic!("{error:?}\n{:#?}", put.body)
    });
    let index = core
        .functions
        .iter()
        .position(|f| f.symbol.ends_with("::put"))
        .unwrap();
    let origin = core_expr_origin(&core.functions[index].body).clone();
    for mutation in 0..6 {
        let mut changed = core.clone();
        let CoreExpr::Call {
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            ..
        } = &mut changed.functions[index].body
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
                *canonical = "fake::Map".into();
            }
            3 => {
                let CoreType::ExternalNamed { arguments, .. } =
                    &mut deferred_evidence_parameters[0]
                else {
                    panic!()
                };
                arguments.swap(0, 1);
            }
            4 => *evidence_argument_index = Some(0),
            _ => deferred_evidence_type_constructor_parameters.push("F".into()),
        }
        let error =
            emit_rust_module(&changed, "main").expect_err("forged deferred stages must be refused");
        assert_eq!(error.code, "SES-RUST-0001", "mutation{mutation}");
        assert_eq!(error.origin.unwrap(), origin, "mutation{mutation}");
    }
}

#[test]
fn map_rejects_forged_key_owner_dictionary_scope_kind_and_source_stages() {
    let core=checked("// 雪🙂\nimport * as maps from \"std/map\"\nfn probe values: maps.Map<String, Int> -> Maybe<Int> = maps.get \"snow\" values\npub fn main -> Int = 0\n");
    let index = core
        .functions
        .iter()
        .position(|f| f.symbol.ends_with("::probe"))
        .unwrap();
    let origin = core_expr_origin(&core.functions[index].body).clone();
    emit_rust_module(&core, "main").unwrap();
    for mutation in 0..16 {
        let mut changed = core.clone();
        let CoreExpr::Call {
            arguments,
            evidence,
            type_ref,
            trait_dispatch,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            ..
        } = &mut changed.functions[index].body
        else {
            panic!()
        };
        match mutation {
            0 => *type_ref = CoreType::Hole,
            1 => {
                *type_ref = CoreType::ExternalNamed {
                    name: "Maybe".into(),
                    canonical: "fake::Maybe".into(),
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
                    name: "Map".into(),
                    canonical: "fake::Map".into(),
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
        let error = emit_rust_module(&changed, "main")
            .err()
            .unwrap_or_else(|| panic!("mutation{mutation} accepted"));
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap(), origin, "mutation{mutation}");
    }
}

#[test]
fn native_map_references_do_not_widen_program_wasm_admission() {
    for (name, ty) in [
        ("empty", "Unit -> maps.Map<String, Int>"),
        ("singleton", "String -> Int -> maps.Map<String, Int>"),
        ("get", "String -> maps.Map<String, Int> -> Maybe<Int>"),
        (
            "insert",
            "String -> Int -> maps.Map<String, Int> -> maps.Map<String, Int>",
        ),
        (
            "remove",
            "String -> maps.Map<String, Int> -> maps.Map<String, Int>",
        ),
        ("keys", "maps.Map<String, Int> -> Array<String>"),
        ("values", "maps.Map<String, Int> -> Array<Int>"),
        ("size", "maps.Map<String, Int> -> Int"),
        ("isEmpty", "maps.Map<String, Int> -> Bool"),
    ] {
        let core=checked(&format!("import * as maps from \"std/map\"\npub fn probe value: Int -> Int = {{ let saved: {ty} = maps.{name}; value }}\npub fn main -> Int = 0\n"));
        emit_rust_module(&core, "main").unwrap_or_else(|e| panic!("{name}:{e:?}"));
        let error = emit_program_wasm(&[&core], "native/map", &["probe".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(
            error.message.contains("requires a target-specific"),
            "{name}:{error:?}"
        );
        assert_eq!(error.origin.unwrap().source, "map-雪🙂.ssrg");
    }
}
