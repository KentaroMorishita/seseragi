use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreExpr, CoreType};

fn compiled(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "console-雫.ssrg",
        "fixture/console",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:#?}"))
    .core_ir
}

#[test]
fn console_accepts_checked_string_arguments_direct_imported_and_saved() {
    for source in [
        "pub effect fn probe = println \"雫🙂\"\n",
        "import * as console from \"std/console\"\npub effect fn probe = console.print \"text\"\n",
        "pub effect fn probe = do { let output = println; output \"saved\" }\n",
        "import {print as output} from \"std/console\"\npub effect fn probe = do { let saved = output; saved \"renamed\" }\n",
        "pub effect fn probe = do { printValue True; println \"\"; println (show ()) }\n",
    ] {
        let artifact = emit_rust_module(&compiled(source), "probe")
            .unwrap_or_else(|error| panic!("{source}\n{error}"));
        assert!(artifact.metadata.capabilities.contains(&"console".to_owned()));
    }
}

#[test]
fn console_rejects_non_string_core_channels_and_source_arity_before_emission() {
    for source in [
        "pub effect fn probe = println \"text\"\n",
        "import * as console from \"std/console\"\npub effect fn probe = console.print \"text\"\n",
    ] {
        let original = compiled(source);
        emit_rust_module(&original, "probe").unwrap();
        for mutation in 0..8 {
            let mut changed = original.clone();
            let origin = changed.functions[0].origin.clone();
            let (arguments, requirements, failure, success) = match &mut changed.functions[0].body {
                CoreExpr::EffectOperation {
                    arguments,
                    requirements,
                    failure,
                    success,
                    ..
                }
                | CoreExpr::EffectInvoke {
                    arguments,
                    requirements,
                    failure,
                    success,
                    ..
                } => (arguments, requirements, failure, success),
                other => panic!("checked Console application: {other:?}"),
            };
            match mutation {
                0 => {
                    arguments[0] = CoreExpr::Boolean {
                        value: true,
                        origin,
                    };
                }
                1 => {
                    arguments[0] = CoreExpr::Variable {
                        name: "forged".into(),
                        callable_signature: None,
                        evidence: vec![],
                        type_ref: CoreType::ExternalNamed {
                            name: "String".into(),
                            canonical: "fake::String".into(),
                            arguments: vec![],
                        },
                        origin,
                    };
                }
                2 => arguments.clear(),
                3 => arguments.push(arguments[0].clone()),
                4 => *success = CoreType::Hole,
                5 => *failure = CoreType::Hole,
                6 => {
                    *requirements = CoreType::Record {
                        closed: true,
                        fields: vec![],
                    }
                }
                _ => {
                    let CoreType::Record { fields, .. } = requirements else {
                        panic!()
                    };
                    fields[0].type_ref = CoreType::ExternalNamed {
                        name: "Console".into(),
                        canonical: "fake::Console".into(),
                        arguments: vec![],
                    };
                }
            }
            let error = emit_rust_module(&changed, "probe").expect_err("unproven Console Core");
            assert_eq!(error.code, "SES-RUST-0001", "mutation {mutation}");
            assert_eq!(error.origin.unwrap().source, "console-雫.ssrg");
        }
    }
}

#[test]
fn a_console_reference_cannot_claim_a_scalar_parameter_or_wrong_result() {
    let original =
        compiled("pub effect fn probe = do { let output = println; output \"saved\" }\n");
    emit_rust_module(&original, "probe").unwrap();
    for mutation in 0..3 {
        let mut changed = original.clone();
        let CoreExpr::Sequence { statements, .. } = &mut changed.functions[0].body else {
            panic!()
        };
        let seseragi_lowering::CoreStatement::PureLet { value, .. } = &mut statements[0] else {
            panic!()
        };
        let CoreExpr::Variable { type_ref, .. } = value else {
            panic!("{value:?}")
        };
        let CoreType::Function { parameter, result } = type_ref else {
            panic!()
        };
        match mutation {
            0 => {
                *parameter = Box::new(CoreType::Named {
                    name: "Int".into(),
                    arguments: vec![],
                })
            }
            1 => *parameter = Box::new(CoreType::Hole),
            _ => *result = Box::new(CoreType::Hole),
        }
        let error = emit_rust_module(&changed, "probe").expect_err("unproven saved Console ABI");
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap().source, "console-雫.ssrg");
    }
}
