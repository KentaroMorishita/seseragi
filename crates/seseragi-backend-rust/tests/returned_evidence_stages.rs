use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreExpr, CoreType};

const METHOD: &str = "trait Render<A> { fn render value:A -> (Unit -> String) }\ninstance Render<Int> { fn render value:Int -> (Unit -> String) = \\_:Unit -> show value }\npub fn probe -> String = render<Int> 42 ()\n";
const FUNCTION: &str = "fn render<A> value:A -> (Unit -> String) where Show<A> = \\_:Unit -> show value\npub fn probe -> String = render<Int> 42 ()\n";

#[test]
fn continuous_returned_calls_keep_checked_dictionary_and_source_stages() {
    for source in [METHOD, FUNCTION] {
        let core = compile_core_module(CompileInput::new(
            "returned-雫.ssrg",
            "fixture/returned",
            source,
        ))
        .unwrap_or_else(|e| panic!("{e:#?}"))
        .core_ir;
        emit_rust_module(&core, "probe").unwrap_or_else(|e| panic!("{source}\n{e}"));
    }
}

#[test]
fn returned_calls_reject_unproved_boundaries_types_kinds_and_dictionaries() {
    for source in [METHOD, FUNCTION] {
        let core = compile_core_module(CompileInput::new(
            "returned-雫.ssrg",
            "fixture/returned",
            source,
        ))
        .unwrap()
        .core_ir;
        for mutation in 0..15 {
            let mut changed = core.clone();
            if mutation < 12 {
                let probe = changed
                    .functions
                    .iter_mut()
                    .find(|f| f.symbol.ends_with("::probe"))
                    .unwrap();
                let CoreExpr::Call {
                    arguments,
                    evidence,
                    evidence_argument_index,
                    deferred_evidence_parameters,
                    deferred_evidence_type_constructor_parameters,
                    type_ref,
                    origin,
                    ..
                } = &mut probe.body
                else {
                    panic!("{probe:#?}")
                };
                match mutation {
                    0 => *evidence_argument_index = Some(0),
                    1 => *evidence_argument_index = Some(2),
                    2 => *evidence_argument_index = Some(usize::MAX),
                    3 => {
                        *type_ref = CoreType::Named {
                            name: "Bool".into(),
                            arguments: vec![],
                        }
                    }
                    4 => {
                        arguments[0] = CoreExpr::Boolean {
                            value: true,
                            origin: origin.clone(),
                        }
                    }
                    5 => {
                        arguments[1] = CoreExpr::Integer {
                            value: "1".into(),
                            origin: origin.clone(),
                        }
                    }
                    6 => evidence.clear(),
                    7 => evidence[0].constraint.trait_identity = Some("fake::trait(Render)".into()),
                    8 => deferred_evidence_parameters.push(CoreType::Hole),
                    9 => deferred_evidence_type_constructor_parameters.push("F".into()),
                    10 => evidence[0].constraint.arguments[0] = CoreType::Hole,
                    11 => {
                        evidence[0].constraint.arguments[0] = CoreType::ExternalNamed {
                            name: "Int".into(),
                            canonical: "fake::Int".into(),
                            arguments: vec![],
                        }
                    }
                    _ => unreachable!(),
                }
            } else if source == METHOD {
                match mutation {
                    12 => changed.trait_methods[0].signature.parameters.clear(),
                    13 => {
                        changed.trait_methods[0].signature.result = CoreType::Named {
                            name: "Bool".into(),
                            arguments: vec![],
                        }
                    }
                    14 => changed.trait_methods[0].trait_parameters[0].arity = 1,
                    _ => unreachable!(),
                }
            } else {
                let function = changed
                    .functions
                    .iter_mut()
                    .find(|f| f.symbol.ends_with("::render"))
                    .unwrap();
                match mutation {
                    12 => function.value_type = None,
                    13 => {
                        function.value_type = Some(CoreType::Named {
                            name: "Bool".into(),
                            arguments: vec![],
                        })
                    }
                    14 => function.type_parameters[0].arity = 1,
                    _ => unreachable!(),
                }
            }
            let error = emit_rust_module(&changed, "probe")
                .expect_err(&format!("mutation {mutation}: {source}"));
            assert_eq!(error.code, "SES-RUST-0001");
            assert_eq!(
                error.origin.as_ref().map(|o| o.source.as_str()),
                Some("returned-雫.ssrg"),
                "mutation {mutation}: {error:#?}"
            );
        }
    }
}
