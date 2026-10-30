use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreInstanceEvidence, CoreType};

#[test]
fn iterable_build_accepts_original_generic_custom_and_five_standard_iterables() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-iterable-build.ssrg"),
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/basic-iterable-build-effects.ssrg"
        ),
    ] {
        let core = compile_core_module(CompileInput::new(
            "iterable-雪🙂.ssrg",
            "native/iterable-build",
            source,
        ))
        .unwrap_or_else(|error| panic!("{error:#?}"))
        .core_ir;
        let artifact = emit_rust_module(&core, "main").unwrap();
        assert!(artifact
            .metadata
            .capabilities
            .iter()
            .any(|c| c == "iterable-build"));
    }
}

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "iterable-雪🙂.ssrg",
        "native/iterable-build",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:#?}"))
    .core_ir
}
fn cases() -> Vec<(
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
)> {
    vec![
        ("array", "fromIterable", "[1,2]", "Array<Int>", "Array<Int>"),
        ("list", "fromIterable", "[1,2]", "Array<Int>", "List<Int>"),
        (
            "set",
            "fromIterable",
            "[1,2]",
            "Array<Int>",
            "sets.Set<Int>",
        ),
        (
            "map",
            "fromEntries",
            "[(1,\"x\"),(2,\"y\")]",
            "Array<(Int,String)>",
            "maps.Map<Int,String>",
        ),
    ]
}
#[test]
fn iterable_build_rejects_forged_collections_elements_provider_children_key_dictionaries_and_stages_at_origin(
) {
    let eq = checked("pub fn equal -> Bool = 1 == 1");
    let CoreExpr::Binary {
        evidence: other, ..
    } = &eq.functions[0].body
    else {
        panic!()
    };
    for (owner, name, literal, _, result) in cases() {
        let core=checked(&format!("// 雪🙂\nimport * as target from \"std/{owner}\"\nimport * as maps from \"std/map\"\nimport * as sets from \"std/set\"\npub fn probe -> {result} = target.{name} {literal}\n"));
        emit_rust_module(&core, "probe").unwrap();
        let origin = core_expr_origin(&core.functions[0].body).clone();
        for mutation in 0..19 {
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
            } = &mut bad.functions[0].body
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
                        name: "std/prelude::Int".into(),
                        arguments: vec![CoreType::Hole],
                    }
                }
                3 => arguments.clear(),
                4 => arguments.extend(vec![arguments[0].clone(); 2]),
                5 => evidence.clear(),
                6 => deferred_evidence_parameters.push(CoreType::Hole),
                7 => deferred_evidence_type_constructor_parameters.push("F".into()),
                8 => *evidence_argument_index = Some(0),
                9 => {
                    *trait_dispatch = Some(seseragi_lowering::CoreTraitDispatch {
                        trait_identity: "std/prelude::Iterable".into(),
                        method: "iterate".into(),
                        method_contract: None,
                    })
                }
                10 => {
                    evidence[0].constraint.arguments[0] = CoreType::Named {
                        name: "std/prelude::Bool".into(),
                        arguments: vec![],
                    }
                }
                11 => {
                    evidence[0].constraint.arguments[1] = CoreType::Named {
                        name: "std/prelude::Bool".into(),
                        arguments: vec![],
                    }
                }
                12 => evidence[0].constraint.trait_identity = Some("fake::Iterable".into()),
                13 => {
                    let CoreInstanceEvidence::Standard { identity, .. } = &mut evidence[0].evidence
                    else {
                        panic!()
                    };
                    *identity = "std/list::Iterable".into();
                }
                14 => {
                    let CoreInstanceEvidence::Standard { type_arguments, .. } =
                        &mut evidence[0].evidence
                    else {
                        panic!()
                    };
                    type_arguments.push(CoreType::Hole);
                }
                15 => {
                    let CoreInstanceEvidence::Standard {
                        evidence_arguments, ..
                    } = &mut evidence[0].evidence
                    else {
                        panic!()
                    };
                    evidence_arguments.push(other[0].clone());
                }
                16 => {
                    let origin = core_expr_origin(&arguments[0]).clone();
                    arguments[0] = CoreExpr::Boolean {
                        value: true,
                        origin,
                    };
                }
                17 => evidence[0].constraint.name = "Counterfeit".into(),
                _ => {
                    if evidence.len() == 1 {
                        evidence.push(other[0].clone());
                    } else {
                        evidence[1].constraint.arguments[0] = CoreType::Named {
                            name: "std/prelude::Bool".into(),
                            arguments: vec![],
                        };
                    }
                }
            }
            let error = emit_rust_module(&bad, "probe")
                .err()
                .unwrap_or_else(|| panic!("accepted {owner}/{mutation}"));
            assert_eq!(error.code, "SES-RUST-0001");
            assert_eq!(error.origin.unwrap(), origin, "{owner}/{mutation}");
        }
    }
}
#[test]
fn iterable_build_saved_references_preserve_and_validate_the_complete_source_stage() {
    fn selected(expr: &mut CoreExpr) -> &mut CoreExpr {
        match expr {
            CoreExpr::Lambda { body, .. } => selected(body),
            _ => expr,
        }
    }
    for (owner, name, _, input, output) in cases() {
        let core=checked(&format!("import * as target from \"std/{owner}\"\nimport * as maps from \"std/map\"\nimport * as sets from \"std/set\"\nfn operation -> ({input} -> {output}) = target.{name}\npub fn main -> Int = 0\n"));
        emit_rust_module(&core, "main").unwrap();
        let index = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::operation"))
            .unwrap();
        for mutation in 0..3 {
            let mut bad = core.clone();
            let expr = selected(&mut bad.functions[index].body);
            let origin = core_expr_origin(expr).clone();
            match expr {
                CoreExpr::Call {
                    arguments,
                    type_ref,
                    evidence,
                    ..
                } => match mutation {
                    0 => *type_ref = CoreType::Hole,
                    1 => {
                        let origin = core_expr_origin(&arguments[0]).clone();
                        arguments[0] = CoreExpr::Boolean {
                            value: true,
                            origin,
                        };
                    }
                    _ => evidence[0].constraint.arguments[1] = CoreType::Hole,
                },
                CoreExpr::Variable { type_ref, .. } => {
                    if mutation == 0 {
                        *type_ref = CoreType::Hole;
                    } else {
                        let CoreType::Function { parameter, result } = type_ref else {
                            panic!()
                        };
                        if mutation == 1 {
                            **parameter = CoreType::Hole;
                        } else {
                            **result = CoreType::Hole;
                        }
                    }
                }
                other => panic!("{other:?}"),
            }
            let error = emit_rust_module(&bad, "main").unwrap_err();
            assert_eq!(error.code, "SES-RUST-0001");
            assert_eq!(error.origin.unwrap(), origin);
        }
    }
}
#[test]
fn iterable_build_does_not_admit_unused_program_wasm_references() {
    for (owner, name, _, input, output) in cases() {
        let core=checked(&format!("import * as target from \"std/{owner}\"\nimport * as maps from \"std/map\"\nimport * as sets from \"std/set\"\npub fn probe value:Int -> Int = {{let saved:{input} -> {output} = target.{name};value}}\npub fn main -> Int = 0\n"));
        emit_rust_module(&core, "main").unwrap();
        let error =
            emit_program_wasm(&[&core], "native/iterable-build", &["probe".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap().source, "iterable-雪🙂.ssrg");
    }
}
