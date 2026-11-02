use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreInstanceEvidence, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "effect-control-雪🙂.ssrg",
        "native/effect-control",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:#?}"))
    .core_ir
}

#[test]
fn effect_attempt_accepts_typed_failures_and_success_values() {
    for (expression, payloads) in [
        ("effects.fail \"typed\"", "String,Never"),
        ("effects.succeed 42", "Never,Int"),
    ] {
        let core = checked(&format!(
            "import * as effects from \"std/effect\"\npub fn probe -> Effect<{{}},Never,Either<{payloads}>> = effects.attempt ({expression})\n",
        ));
        emit_rust_module(&core, "probe").unwrap();
    }
}

#[test]
fn effect_from_maybe_accepts_an_explicit_failure_and_maybe_value() {
    let core = checked(
        "import * as effects from \"std/effect\"\npub fn probe -> Effect<{}, String, Int> = effects.fromMaybe \"missing\" (Just 42)\n",
    );
    emit_rust_module(&core, "probe").unwrap();
}

#[test]
fn effect_for_each_until_accepts_selected_iterable_and_both_controls() {
    let core = checked(
        "import * as effects from \"std/effect\"\nfn step value:Int -> Effect<{}, String, effects.LoopControl> = effects.succeed (if value == 2 then effects.Break else effects.Continue)\npub fn probe -> Effect<{}, String, Unit> = effects.forEachUntil step [1,2,3]\n",
    );
    emit_rust_module(&core, "probe").unwrap();
}

#[test]
fn effect_control_supports_saved_partial_generic_and_custom_iterable_inputs() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-effect-control.ssrg"),
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/basic-effect-control-effects.ssrg"
        ),
    ] {
        let core = checked(source);
        emit_rust_module(&core, "main").unwrap();
    }
}

fn prefix() -> &'static str {
    "import * as effects from \"std/effect\"\nfn step value:Int -> Effect<{},String,effects.LoopControl> = effects.succeed effects.Continue\nfn work -> Effect<{},String,Int> = effects.fromMaybe \"missing\" (Just 42)\n"
}

fn cases() -> [(&'static str, &'static str, &'static str); 3] {
    [
        ("attempt (work ())", "Effect<{},Never,Either<String,Int>>", "Effect<{},String,Int> -> Effect<{},Never,Either<String,Int>>"),
        ("fromMaybe \"missing\" (Just 42)", "Effect<{},String,Int>", "String -> Maybe<Int> -> Effect<{},String,Int>"),
        ("forEachUntil step [1,2,3]", "Effect<{},String,Unit>", "(Int -> Effect<{},String,effects.LoopControl>) -> Array<Int> -> Effect<{},String,Unit>"),
    ]
}

#[test]
fn effect_control_rejects_forged_calls_dictionaries_and_source_stages() {
    for (expression, output, _) in cases() {
        let core = checked(&format!(
            "{}pub fn probe -> {output} = effects.{expression}\n",
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
            panic!("checked call")
        };
        let argument_count = arguments.len();
        let is_loop = expression.starts_with("forEachUntil");
        for mutation in 0..10 + argument_count + if is_loop { 6 } else { 0 } {
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
                        name: "Effect".into(),
                        canonical: "other::Effect".into(),
                        arguments: vec![],
                    }
                }
                2 => {
                    let CoreType::Named { arguments, .. } = type_ref else {
                        panic!("Effect result")
                    };
                    arguments[1] = boolean;
                }
                3 => arguments.clear(),
                4 => arguments.extend(vec![arguments[0].clone(); 3]),
                5 => {
                    if is_loop {
                        evidence.clear();
                    } else {
                        let other = checked("pub fn probe -> Bool = 1 == 1");
                        let CoreExpr::Binary {
                            evidence: selected, ..
                        } = &other.functions[0].body
                        else {
                            panic!()
                        };
                        evidence.extend(selected.clone());
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
                    arguments[i] = CoreExpr::Boolean {
                        value: true,
                        origin: core_expr_origin(&arguments[i]).clone(),
                    };
                }
                n => match n - 10 - argument_count {
                    0 => evidence[0].constraint.arguments[0] = boolean,
                    1 => evidence[0].constraint.arguments[1] = boolean,
                    2 => evidence[0].constraint.trait_identity = Some("other::Iterable".into()),
                    3 | 4 => {
                        let CoreInstanceEvidence::Standard {
                            identity,
                            type_arguments,
                            ..
                        } = &mut evidence[0].evidence
                        else {
                            panic!()
                        };
                        if n - 10 - argument_count == 3 {
                            *identity = "std/list::Iterable".into();
                        } else {
                            type_arguments.push(CoreType::Hole);
                        }
                    }
                    _ => {
                        let CoreExpr::Variable { type_ref, .. } = &mut arguments[0] else {
                            panic!("action reference")
                        };
                        let CoreType::Function { result, .. } = type_ref else {
                            panic!()
                        };
                        let CoreType::Named { arguments, .. } = result.as_mut() else {
                            panic!()
                        };
                        arguments[2] = CoreType::ExternalNamed {
                            name: "LoopControl".into(),
                            canonical: "other::LoopControl".into(),
                            arguments: vec![],
                        };
                    }
                },
            }
            let error = emit_rust_module(&bad, "probe")
                .err()
                .unwrap_or_else(|| panic!("accepted {expression}/{mutation}"));
            assert_eq!(error.code, "SES-RUST-0001");
            assert_eq!(error.origin.unwrap(), origin, "{expression}/{mutation}");
        }
    }
}

#[test]
fn effect_control_saved_functions_and_constants_validate_their_canonical_types() {
    fn selected(expr: &mut CoreExpr) -> &mut CoreExpr {
        match expr {
            CoreExpr::Lambda { body, .. } => selected(body),
            _ => expr,
        }
    }
    for (expression, _, signature) in cases().into_iter().chain([
        ("Continue", "effects.LoopControl", "effects.LoopControl"),
        ("Break", "effects.LoopControl", "effects.LoopControl"),
    ]) {
        let name = expression.split_whitespace().next().unwrap();
        let core = checked(&format!(
            "{}fn saved -> ({signature}) = effects.{name}\npub fn main -> Int = 0\n",
            prefix()
        ));
        emit_rust_module(&core, "main").unwrap();
        let index = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::saved"))
            .unwrap();
        let mut bad = core.clone();
        let expr = selected(&mut bad.functions[index].body);
        let origin = core_expr_origin(expr).clone();
        match expr {
            CoreExpr::Variable { type_ref, .. } | CoreExpr::Call { type_ref, .. } => {
                *type_ref = CoreType::Hole
            }
            other => panic!("{other:?}"),
        }
        let error = emit_rust_module(&bad, "main").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap(), origin);
    }
}

#[test]
fn effect_control_does_not_admit_unused_program_wasm_references() {
    for (expression, _, signature) in cases().into_iter().chain([
        ("Continue", "effects.LoopControl", "effects.LoopControl"),
        ("Break", "effects.LoopControl", "effects.LoopControl"),
    ]) {
        let name = expression.split_whitespace().next().unwrap();
        let core = checked(&format!("{}pub fn probe value:Int -> Int = {{ let saved:{signature} = effects.{name}; value }}\npub fn main -> Int = 0\n", prefix()));
        emit_rust_module(&core, "main").unwrap();
        let error =
            emit_program_wasm(&[&core], "native/effect-control", &["probe".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap().source, "effect-control-雪🙂.ssrg");
    }
}
