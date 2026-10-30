use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreInstanceEvidence, CoreType};

#[test]
fn reduce_until_accepts_original_saved_generic_function_accumulators_and_cold_inputs() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-reduce-until.ssrg"),
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/basic-reduce-until-effects.ssrg"
        ),
    ] {
        let core = compile_core_module(CompileInput::new(
            "reduce-until-雪🙂.ssrg",
            "native/reduce-until",
            source,
        ))
        .unwrap_or_else(|error| panic!("{error:#?}"))
        .core_ir;
        let artifact = emit_rust_module(&core, "main").unwrap();
        assert!(artifact
            .metadata
            .capabilities
            .iter()
            .any(|c| c == "collection-control"));
    }
}

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "reduce-until-雪🙂.ssrg",
        "native/reduce-until",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:#?}"))
    .core_ir
}
fn cases() -> Vec<(&'static str, &'static str, &'static str, &'static str)> {
    vec![
        (
            "Next",
            "Next 7",
            "control.ReduceStep<Int>",
            "Int -> control.ReduceStep<Int>",
        ),
        (
            "Done",
            "Done 7",
            "control.ReduceStep<Int>",
            "Int -> control.ReduceStep<Int>",
        ),
        (
            "reduceUntil",
            "reduceUntil 0 advance [1,2]",
            "Int",
            "Int -> (Int -> Int -> control.ReduceStep<Int>) -> Array<Int> -> Int",
        ),
    ]
}
fn prefix() -> &'static str {
    "import * as control from \"std/collection\"\nfn advance total:Int -> value:Int -> control.ReduceStep<Int> = control.Next (total + value)\n"
}
#[test]
fn reduce_until_rejects_forged_payload_accumulator_callback_source_dictionary_and_stages_at_origin()
{
    let eq = checked("pub fn equal -> Bool = 1 == 1");
    let CoreExpr::Binary {
        evidence: other, ..
    } = &eq.functions[0].body
    else {
        panic!()
    };
    for (name, expression, result, _) in cases() {
        let core = checked(&format!(
            "// 雪🙂\n{}pub fn probe -> {result} = control.{expression}\n",
            prefix()
        ));
        emit_rust_module(&core, "probe").unwrap();
        let index = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::probe"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[index].body).clone();
        let CoreExpr::Call { arguments, .. } = &core.functions[index].body else {
            panic!()
        };
        let argument_count = arguments.len();
        let extras = if name == "reduceUntil" { 6 } else { 2 };
        for mutation in 0..10 + argument_count + extras {
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
            let boolean = CoreType::Named {
                name: "std/prelude::Bool".into(),
                arguments: vec![],
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
                4 => arguments.extend(vec![arguments[0].clone(); 4]),
                5 => {
                    if evidence.is_empty() {
                        evidence.push(other[0].clone());
                    } else {
                        evidence.clear();
                    }
                }
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
                n if n < 10 + argument_count => {
                    let i = n - 10;
                    let origin = core_expr_origin(&arguments[i]).clone();
                    arguments[i] = CoreExpr::Boolean {
                        value: true,
                        origin,
                    };
                }
                n => match (name, n - 10 - argument_count) {
                    ("reduceUntil", 0) => *type_ref = boolean,
                    ("reduceUntil", 1) => evidence[0].constraint.arguments[0] = boolean,
                    ("reduceUntil", 2) => evidence[0].constraint.arguments[1] = boolean,
                    ("reduceUntil", 3) => {
                        let CoreInstanceEvidence::Standard { identity, .. } =
                            &mut evidence[0].evidence
                        else {
                            panic!()
                        };
                        *identity = "std/list::Iterable".into();
                    }
                    ("reduceUntil", which) => {
                        let ty = match &mut arguments[1] {
                            CoreExpr::Variable { type_ref, .. }
                            | CoreExpr::Call { type_ref, .. } => type_ref,
                            other => panic!("{other:?}"),
                        };
                        let CoreType::Function { parameter, result } = ty else {
                            panic!()
                        };
                        if which == 4 {
                            **parameter = CoreType::Named {
                                name: "std/prelude::Float".into(),
                                arguments: vec![],
                            };
                        } else {
                            let CoreType::Function { result, .. } = result.as_mut() else {
                                panic!()
                            };
                            **result = CoreType::Named {
                                name: "std/collection::ReduceStep".into(),
                                arguments: vec![boolean],
                            };
                        }
                    }
                    (_, which) => {
                        *type_ref = CoreType::Named {
                            name: "std/collection::ReduceStep".into(),
                            arguments: if which == 0 { vec![boolean] } else { vec![] },
                        }
                    }
                },
            }
            let error = emit_rust_module(&bad, "probe")
                .err()
                .unwrap_or_else(|| panic!("accepted {name}/{mutation}"));
            assert_eq!(error.code, "SES-RUST-0001");
            assert_eq!(error.origin.unwrap(), origin, "{name}/{mutation}");
        }
    }
}
#[test]
fn reduce_until_saved_references_validate_the_original_curried_source_signature() {
    fn selected(expr: &mut CoreExpr) -> &mut CoreExpr {
        match expr {
            CoreExpr::Lambda { body, .. } => selected(body),
            _ => expr,
        }
    }
    for (name, _, _, signature) in cases() {
        let core = checked(&format!(
            "{}fn operation -> ({signature}) = control.{name}\npub fn main -> Int = 0\n",
            prefix()
        ));
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
fn reduce_until_does_not_admit_unused_program_wasm_control_references() {
    for (name, _, _, signature) in cases() {
        let core=checked(&format!("{}pub fn probe value:Int -> Int = {{let saved:{signature} = control.{name};value}}\npub fn main -> Int = 0\n",prefix()));
        emit_rust_module(&core, "main").unwrap();
        let error =
            emit_program_wasm(&[&core], "native/reduce-until", &["probe".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap().source, "reduce-until-雪🙂.ssrg");
    }
}
