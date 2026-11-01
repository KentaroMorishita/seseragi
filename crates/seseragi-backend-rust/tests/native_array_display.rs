#[path = "support/native.rs"]
mod support;

use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::CoreExpr;
use support::run;

#[test]
fn array_display_executes_selected_user_generic_and_higher_kinded_children() {
    run(
        include_str!("../../../examples/spec/fixtures/rust-backend/array-display.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/array-display.stdout")
            .trim_end(),
    );
}

#[test]
fn array_nominal_names_and_prelude_aliases_retain_selected_identity() {
    run(
        include_str!("../../../examples/spec/fixtures/rust-backend/array-nominal.ssrg"),
        "([selected-int], [local=selected-int], [1, 2])",
    );
    run(
        r#"
alias Vector<A> = Array<A>
alias Count = Int
pub fn probe -> String = {
  let standard: Vector<Count> = [1, 2]
  show standard
}
"#,
        "[1, 2]",
    );
}

#[test]
fn array_child_type_arguments_cannot_rebind_builtin_arrays_to_a_nominal_shadow() {
    use seseragi_lowering::{CoreInstanceEvidence, CoreType};
    let source = r#"
type Array<A> = | LocalArray A
type Box<A> = | Box A
instance<A> Show<Box<A>> where Show<A> {
  fn show value: Box<A> -> String = match value { Box item -> show item }
}
pub fn probe -> String = show [Box [1, 2]]
"#;
    run(source, "[[1, 2]]");
    let mut core = compile_core_module(CompileInput::new(
        "array-shadow-λ.ssrg",
        "native/main",
        source,
    ))
    .unwrap()
    .core_ir;
    let function = core
        .functions
        .iter_mut()
        .find(|f| f.symbol.ends_with("::probe"))
        .unwrap();
    let CoreExpr::Call { evidence, .. } = &mut function.body else {
        panic!("call")
    };
    let CoreInstanceEvidence::Standard {
        evidence_arguments, ..
    } = &mut evidence[0].evidence
    else {
        panic!("array")
    };
    let CoreInstanceEvidence::Local { type_arguments, .. } = &mut evidence_arguments[0].evidence
    else {
        panic!("box")
    };
    let arguments = match &type_arguments[0] {
        CoreType::Named { name, arguments } if name == "Array" => arguments,
        CoreType::ExternalNamed {
            canonical,
            arguments,
            ..
        } if canonical == "std/prelude::Array" => arguments,
        _ => panic!("builtin array"),
    };
    type_arguments[0] = CoreType::ExternalNamed {
        name: "Array".into(),
        canonical: "native/main::Array".into(),
        arguments: arguments.clone(),
    };
    let error = emit_rust_module(&core, "probe").unwrap_err();
    assert_eq!(error.origin.as_ref().unwrap().source, "array-shadow-λ.ssrg");
    assert!(error.message.contains("child instance type mismatch"));
}

#[test]
fn array_generic_children_and_saved_partials_reject_unproven_evidence() {
    use seseragi_lowering::{CoreInstanceEvidence, CoreStatement, CoreType};
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/array-display.ssrg");
    let core = compile_core_module(CompileInput::new("array-λ.ssrg", "native/main", source))
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
                panic!("array")
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
            let CoreStatement::PureLet {
                value:
                    CoreExpr::Call {
                        deferred_evidence_parameters,
                        evidence,
                        ..
                    },
                ..
            } = &mut statements[3]
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
                        panic!("array")
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
        assert_eq!(error.origin.as_ref().unwrap().source, "array-λ.ssrg");
    }
}

#[test]
fn array_display_rejects_counterfeit_types_children_and_dispatch() {
    use seseragi_lowering::{CoreInstanceEvidence, CoreType};
    let core = compile_core_module(CompileInput::new(
        "array-λ.ssrg",
        "native/main",
        "pub fn probe -> String = show [1, 2]\n",
    ))
    .unwrap()
    .core_ir;
    for mutation in 0..16 {
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
            1 => evidence[0].constraint.arguments.clear(),
            2 => trait_dispatch.as_mut().unwrap().trait_identity = "user::Show".into(),
            3 => trait_dispatch.as_mut().unwrap().method = "debug".into(),
            4 | 5 => {
                evidence[0].constraint.arguments[0] = CoreType::ExternalNamed {
                    name: "Array".into(),
                    canonical: "user::Array".into(),
                    arguments: if mutation == 4 {
                        vec![CoreType::Named {
                            name: "Int".into(),
                            arguments: vec![],
                        }]
                    } else {
                        vec![]
                    },
                }
            }
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
                    6 => type_arguments.clear(),
                    7 => type_arguments.push(type_arguments[0].clone()),
                    8 => evidence_arguments.clear(),
                    9 => evidence_arguments.push(evidence_arguments[0].clone()),
                    10 => {
                        evidence_arguments[0].constraint.trait_identity = Some("user::Show".into())
                    }
                    11 => evidence_arguments[0].constraint.name = "Debug".into(),
                    12 => evidence_arguments[0].constraint.arguments.clear(),
                    13 => {
                        evidence_arguments[0].evidence = CoreInstanceEvidence::Standard {
                            identity: "Show<std/prelude::Bool>".into(),
                            type_arguments: vec![],
                            evidence_arguments: vec![],
                        }
                    }
                    14 => {
                        evidence_arguments[0].evidence =
                            CoreInstanceEvidence::Parameter { index: 0 }
                    }
                    _ => {
                        type_arguments[0] = CoreType::Named {
                            name: "Bool".into(),
                            arguments: vec![],
                        }
                    }
                }
            }
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.as_ref().unwrap().source, "array-λ.ssrg");
        assert_eq!(error.module.as_deref(), Some("native/main"));
    }
}
