#[path = "support/native.rs"]
mod support;

use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::CoreExpr;
use std::process::Command;
use support::{emit, run, Scratch};

#[test]
fn tuple_display_executes_selected_user_generic_and_higher_kinded_children() {
    run(
        include_str!("../../../examples/spec/fixtures/rust-backend/tuple-nominal.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/tuple-nominal.stdout")
            .trim_end(),
    );
    let artifact = emit(include_str!(
        "../../../examples/spec/fixtures/rust-backend/tuple-display.ssrg"
    ));
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&artifact.source))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        include_str!("../../../examples/spec/fixtures/rust-backend/tuple-display.stdout")
    );
}

#[test]
fn tuple_display_rejects_counterfeit_types_children_dispatch_and_partial_staging() {
    use seseragi_lowering::{CoreInstanceEvidence, CoreType};
    let core = compile_core_module(CompileInput::new(
        "tuple-λ.ssrg",
        "native/main",
        "pub fn probe -> String = show (1, True)\n",
    ))
    .unwrap()
    .core_ir;
    for mutation in 0..12 {
        let mut changed = core.clone();
        let CoreExpr::Call {
            evidence,
            trait_dispatch,
            ..
        } = &mut changed.functions[0].body
        else {
            panic!("call")
        };
        match mutation {
            0 => evidence[0].constraint.trait_identity = Some("user::Show".into()),
            1 => {
                evidence[0].constraint.arguments[0] = CoreType::Named {
                    name: "Tuple".into(),
                    arguments: vec![],
                }
            }
            2 => trait_dispatch.as_mut().unwrap().trait_identity = "user::Show".into(),
            3 => trait_dispatch.as_mut().unwrap().method = "debug".into(),
            _ => {
                let CoreInstanceEvidence::Standard {
                    type_arguments,
                    evidence_arguments,
                    ..
                } = &mut evidence[0].evidence
                else {
                    panic!("standard")
                };
                match mutation {
                    4 => type_arguments.clear(),
                    5 => {
                        type_arguments[0] = CoreType::Named {
                            name: "user::Tuple".into(),
                            arguments: vec![],
                        }
                    }
                    6 => evidence_arguments.pop().map(|_| ()).unwrap(),
                    7 => evidence_arguments.reverse(),
                    8 => {
                        evidence_arguments[0].constraint.trait_identity = Some("user::Show".into())
                    }
                    9 => evidence_arguments[0].constraint.name = "Debug".into(),
                    10 => {
                        evidence_arguments[0].evidence = CoreInstanceEvidence::Standard {
                            identity: "Show<std/prelude::Bool>".into(),
                            type_arguments: vec![],
                            evidence_arguments: vec![],
                        }
                    }
                    _ => {
                        evidence_arguments[0].evidence =
                            CoreInstanceEvidence::Parameter { index: 0 }
                    }
                }
            }
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.as_ref().unwrap().source, "tuple-λ.ssrg");
        assert_eq!(error.module.as_deref(), Some("native/main"));
    }
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/tuple-display.ssrg");
    let core = compile_core_module(CompileInput::new("tuple.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    for mutation in 0..7 {
        let mut changed = core.clone();
        if mutation < 3 {
            let function = changed
                .functions
                .iter_mut()
                .find(|f| f.symbol.ends_with("::selected"))
                .unwrap();
            let CoreExpr::Call { evidence, .. } = &mut function.body else {
                panic!("call")
            };
            let CoreInstanceEvidence::Standard {
                evidence_arguments, ..
            } = &mut evidence[0].evidence
            else {
                panic!("tuple")
            };
            match mutation {
                0 => evidence_arguments[0].evidence = CoreInstanceEvidence::Parameter { index: 1 },
                1 => function.constraints[0].name = "Debug".into(),
                _ => {
                    function.constraints[0].arguments[0] = CoreType::Named {
                        name: "Bool".into(),
                        arguments: vec![],
                    }
                }
            }
        } else {
            let function = changed
                .functions
                .iter_mut()
                .find(|f| f.symbol.ends_with("::probe"))
                .unwrap();
            let CoreExpr::Sequence { statements, .. } = &mut function.body else {
                panic!("block")
            };
            let seseragi_lowering::CoreStatement::PureLet {
                value:
                    CoreExpr::Call {
                        deferred_evidence_parameters,
                        evidence,
                        ..
                    },
                ..
            } = &mut statements[2]
            else {
                panic!("partial")
            };
            match mutation {
                3 => deferred_evidence_parameters.push(CoreType::Named {
                    name: "String".into(),
                    arguments: vec![],
                }),
                4 => {
                    deferred_evidence_parameters[0] = CoreType::Named {
                        name: "Int".into(),
                        arguments: vec![],
                    }
                }
                _ => {
                    let CoreInstanceEvidence::Standard {
                        evidence_arguments, ..
                    } = &mut evidence[0].evidence
                    else {
                        panic!("tuple")
                    };
                    let CoreInstanceEvidence::Local { identity, .. } =
                        &mut evidence_arguments[0].evidence
                    else {
                        panic!("local")
                    };
                    *identity = if mutation == 5 {
                        "std/prelude::Debug<native/main::Label>"
                    } else {
                        "std/prelude::Show<native/main::Box<$0>>"
                    }
                    .into();
                }
            }
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(error.origin.is_some());
    }
}

#[test]
fn tuple_scalar_aliases_use_canonical_identity_and_preserve_unit_entry_checks() {
    use seseragi_lowering::{CoreInstanceEvidence, CoreType};
    let core = compile_core_module(CompileInput::new(
        "aliases-λ.ssrg",
        "native/main",
        "pub fn probe -> String = show (\"x\", ())\n",
    ))
    .unwrap()
    .core_ir;
    for forged in [false, true] {
        let mut changed = core.clone();
        changed.functions[0].parameters[0].type_ref = CoreType::ExternalNamed {
            name: "EntryUnit".into(),
            canonical: "std/prelude::Unit".into(),
            arguments: vec![],
        };
        let CoreExpr::Call { evidence, .. } = &mut changed.functions[0].body else {
            panic!("call")
        };
        let string = CoreType::ExternalNamed {
            name: "Text".into(),
            canonical: if forged {
                "user::String"
            } else {
                "std/prelude::String"
            }
            .into(),
            arguments: vec![],
        };
        let unit = CoreType::ExternalNamed {
            name: "Nil".into(),
            canonical: "std/prelude::Unit".into(),
            arguments: vec![],
        };
        let tuple = CoreType::Tuple {
            elements: vec![string.clone(), unit.clone()],
        };
        evidence[0].constraint.arguments = vec![tuple.clone()];
        let CoreInstanceEvidence::Standard {
            type_arguments,
            evidence_arguments,
            ..
        } = &mut evidence[0].evidence
        else {
            panic!("tuple")
        };
        *type_arguments = vec![tuple];
        evidence_arguments[0].constraint.arguments = vec![string];
        evidence_arguments[1].constraint.arguments = vec![unit];
        let result = emit_rust_module(&changed, "probe");
        if forged {
            let error = result.unwrap_err();
            assert_eq!(error.code, "SES-RUST-0001");
            assert!(error.origin.is_some());
        } else {
            assert!(result.is_ok(), "{result:?}");
        }
    }
    let mut changed = core;
    changed.functions[0].parameters[0].type_ref = CoreType::ExternalNamed {
        name: "Unit".into(),
        canonical: "user::Unit".into(),
        arguments: vec![],
    };
    let error = emit_rust_module(&changed, "probe").unwrap_err();
    assert!(error.message.contains("unconstrained Unit"));
    assert!(error.origin.is_some());
}
