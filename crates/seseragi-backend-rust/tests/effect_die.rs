use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreExpr, CoreType};

#[test]
fn die_emits_an_explicit_cold_defect_only_when_the_module_uses_it() {
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/effect-die.ssrg");
    let checked = compile_core_module(CompileInput::new("effect-die.ssrg", "fixture/die", source))
        .expect("source die contract");
    let artifact =
        emit_rust_module(&checked.core_ir, "probe").expect("checked die backend contract");
    assert!(artifact
        .metadata
        .capabilities
        .contains(&"effect-defect".to_owned()));
    assert!(artifact
        .source
        .contains("fn native_effect_die(message: V) -> V"));
    assert!(artifact
        .source
        .contains("Err(NativeExit::Defect(vec![message.get().clone()]))"));

    let plain = compile_core_module(CompileInput::new(
        "plain.ssrg",
        "fixture/plain",
        "pub fn probe -> Int = 42\n",
    ))
    .unwrap();
    let artifact = emit_rust_module(&plain.core_ir, "probe").unwrap();
    assert!(!artifact.source.contains("native_effect_die"));
    assert!(!artifact
        .metadata
        .capabilities
        .contains(&"effect-defect".to_owned()));
}

#[test]
fn die_refuses_forged_failure_success_message_and_source_arity() {
    let source = "import * as effects from \"std/effect\"\npub effect fn probe -> Never = effects.die \"expected\"\n";
    let original = compile_core_module(CompileInput::new("die-雪.ssrg", "fixture/die", source))
        .unwrap()
        .core_ir;
    for mutation in 0..5 {
        let mut core = original.clone();
        let origin = core.functions[0].origin.clone();
        let body = &mut core.functions[0].body;
        let (arguments, failure, success) = match body {
            CoreExpr::EffectInvoke {
                arguments,
                failure,
                success,
                ..
            }
            | CoreExpr::EffectOperation {
                arguments,
                failure,
                success,
                ..
            } => (arguments, failure, success),
            other => panic!("checked die instruction: {other:?}"),
        };
        match mutation {
            0 => *failure = CoreType::Hole,
            1 => {
                *success = CoreType::Named {
                    name: "String".into(),
                    arguments: vec![],
                }
            }
            2 => arguments.clear(),
            3 => arguments.push(arguments[0].clone()),
            _ => {
                arguments[0] = CoreExpr::Variable {
                    name: "fixture/die::forged".into(),
                    callable_signature: None,
                    evidence: vec![],
                    type_ref: CoreType::ExternalNamed {
                        name: "String".into(),
                        canonical: "other::String".into(),
                        arguments: vec![],
                    },
                    origin,
                }
            }
        }
        let error =
            emit_rust_module(&core, "probe").expect_err("unproven die Core must be rejected");
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(error.origin.is_some(), "mutation {mutation}: {error}");
    }
}
