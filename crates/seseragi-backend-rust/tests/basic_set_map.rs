use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreInstanceEvidence, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new("set-map-雪🙂.ssrg", "native/set", source))
        .unwrap_or_else(|error| panic!("{source}\n{error:?}"))
        .core_ir
}
const OPERATION: &str = "// 雪🙂\nimport * as sets from \"std/set\"\nfn operation callback:(String -> Int) -> values:sets.Set<String> -> sets.Set<Int> = sets.map callback values\npub fn main -> Int = 0\n";

#[test]
fn set_mapping_keeps_generic_saved_partial_functions_and_only_requires_output_evidence() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-set-map.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-set-map-effects.ssrg"),
        "import * as sets from \"std/set\"\nfn transform<A,B> callback:(A -> B) -> sets.Set<A> -> sets.Set<B> where Eq<B>, Hash<B> = sets.map callback\npub fn main -> Int = 0\n",
    ] {
        emit_rust_module(&checked(source), "main").unwrap();
    }
}

#[test]
fn set_mapping_rejects_forged_input_output_callback_dictionaries_and_source_stages() {
    let core = checked(OPERATION);
    let strings = checked("import * as sets from \"std/set\"\nfn stringKey value:String -> values:sets.Set<String> -> Bool = sets.contains value values\npub fn main -> Int = 0\n");
    let CoreExpr::Call {
        evidence: strings, ..
    } = &strings
        .functions
        .iter()
        .find(|function| function.symbol.ends_with("::stringKey"))
        .unwrap()
        .body
    else {
        panic!()
    };
    let index = core
        .functions
        .iter()
        .position(|function| function.symbol.ends_with("::operation"))
        .unwrap();
    let origin = core_expr_origin(&core.functions[index].body).clone();
    emit_rust_module(&core, "main").unwrap();
    for mutation in 0..22 {
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
            1..=4 => {
                *type_ref = CoreType::Named {
                    name: if mutation == 1 {
                        "fake::Set"
                    } else {
                        "std/set::Set"
                    }
                    .into(),
                    arguments: match mutation {
                        2 => vec![],
                        3 => vec![named("Int"), named("Int")],
                        4 => vec![named("String")],
                        _ => vec![named("Int")],
                    },
                }
            }
            5 => arguments.clear(),
            6 => arguments.push(arguments[0].clone()),
            7 => {
                arguments[0] = CoreExpr::Unit {
                    origin: origin.clone(),
                }
            }
            8 => {
                arguments[1] = CoreExpr::Unit {
                    origin: origin.clone(),
                }
            }
            9 => evidence.clear(),
            10 => evidence.swap(0, 1),
            11 => evidence[0].constraint.trait_identity = Some("fake::Eq".into()),
            12 => evidence[0].constraint.arguments[0] = CoreType::Hole,
            13 => *evidence = strings.clone(),
            14 => {
                *trait_dispatch = Some(seseragi_lowering::CoreTraitDispatch {
                    trait_identity: "std/prelude::Eq".into(),
                    method: "eq".into(),
                    method_contract: None,
                })
            }
            15 => deferred_evidence_parameters.push(CoreType::Hole),
            16 => deferred_evidence_type_constructor_parameters.push("F".into()),
            17 => *evidence_argument_index = Some(0),
            18..=19 => {
                let CoreExpr::Variable { type_ref, .. } = &mut arguments[1] else {
                    panic!()
                };
                *type_ref = CoreType::Named {
                    name: if mutation == 18 {
                        "fake::Set"
                    } else {
                        "std/set::Set"
                    }
                    .into(),
                    arguments: vec![CoreType::Hole],
                };
            }
            20 => {
                let CoreExpr::Variable { type_ref, .. } = &mut arguments[0] else {
                    panic!()
                };
                *type_ref = CoreType::Function {
                    parameter: Box::new(named("String")),
                    result: Box::new(named("Bool")),
                };
            }
            21 => evidence[0].evidence = CoreInstanceEvidence::Parameter { index: usize::MAX },
            _ => unreachable!(),
        }
        let error =
            emit_rust_module(&bad, "main").expect_err(&format!("Set.map mutation{mutation}"));
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap(), origin, "mutation{mutation}");
    }
}

#[test]
fn unused_set_map_references_do_not_expand_program_wasm() {
    for source in [OPERATION.to_owned(), "import * as sets from \"std/set\"\nfn unused -> ((String -> Int) -> sets.Set<String> -> sets.Set<Int>) = sets.map\npub fn main -> Int = 0\n".into()] {
        let core = checked(&source);
        emit_rust_module(&core, "main").unwrap();
        let error = emit_program_wasm(&[&core], &core.module, &["main".to_owned()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(error.origin.is_some());
    }
}
