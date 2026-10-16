use seseragi_backend_rust::{emit_rust_module, RustArtifact};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{
    core_free_variables, is_closed_local_evidence, known_pure_call_shape, CoreCallShape, CoreExpr,
};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

static SERIAL: AtomicUsize = AtomicUsize::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "seseragi-rust-target-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn compile(&self, source: &str) -> PathBuf {
        let input = self.0.join("generated.rs");
        let binary = self.0.join(if cfg!(windows) {
            "native.exe"
        } else {
            "native"
        });
        fs::write(&input, source).unwrap();
        let result = Command::new("rustc")
            .args(["--edition=2021", "-O", "-o"])
            .arg(&binary)
            .arg(&input)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        binary
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn emit(source: &str) -> RustArtifact {
    let core = compile_core_module(CompileInput::new("main.ssrg", "native/main", source)).unwrap();
    emit_rust_module(&core.core_ir, "probe").unwrap()
}

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

#[test]
fn persistent_list_values_patterns_display_and_operations_execute() {
    let artifact = emit(include_str!(
        "../../../examples/spec/fixtures/rust-backend/list-core.ssrg"
    ));
    assert!(artifact.metadata.capabilities.iter().any(|x| x == "list"));
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
        include_str!("../../../examples/spec/fixtures/rust-backend/list-core.stdout")
    );
}

#[test]
fn deep_list_construction_traversal_and_drop_are_separately_stack_safe() {
    let artifact = emit(include_str!(
        "../../../examples/spec/fixtures/rust-backend/list-tail.ssrg"
    ));
    assert_eq!(artifact.metadata.self_tail_calls, 2);
    let main = artifact.source.rfind("fn main()").unwrap();
    let source = format!("{}\nfn main() {{ std::thread::Builder::new().stack_size(256 * 1024).spawn(|| {{ let result = {}(vec![]).apply(V::Unit); assert_eq!(result.render(), \"5000050000\"); drop(result); clear_dictionaries(); assert_eq!(LIVE.with(Cell::get), 0); }}).unwrap().join().unwrap(); }}", &artifact.source[..main], rust_id(&artifact.metadata.entry));
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&source))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn list_evidence_and_literal_reject_counterfeit_identity_type_child_and_dispatch() {
    let core = compile_core_module(CompileInput::new(
        "list.ssrg",
        "native/main",
        "pub fn probe -> String = show `[1, 2]\n",
    ))
    .unwrap()
    .core_ir;
    assert!(emit_rust_module(&core, "probe").is_ok());
    for mutation in 0..8 {
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
                evidence[0].constraint.arguments[0] = seseragi_lowering::CoreType::Named {
                    name: "user::List".into(),
                    arguments: vec![],
                }
            }
            2 | 3 | 4 | 5 => {
                let seseragi_lowering::CoreInstanceEvidence::Standard {
                    type_arguments,
                    evidence_arguments,
                    ..
                } = &mut evidence[0].evidence
                else {
                    panic!("standard")
                };
                match mutation {
                    2 => type_arguments.clear(),
                    3 => evidence_arguments.clear(),
                    4 => evidence_arguments[0].constraint.name = "Debug".into(),
                    _ => {
                        evidence_arguments[0].constraint.arguments[0] =
                            seseragi_lowering::CoreType::Named {
                                name: "Char".into(),
                                arguments: vec![],
                            }
                    }
                }
            }
            6 => trait_dispatch.as_mut().unwrap().trait_identity = "user::Show".into(),
            _ => trait_dispatch.as_mut().unwrap().method = "debug".into(),
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(error.origin.is_some());
    }
    for source in [
        "pub fn probe -> List<Int> = `[1]\n",
        "pub fn probe -> List<Int> = 1 : `[]\n",
    ] {
        let mut core = compile_core_module(CompileInput::new("list.ssrg", "native/main", source))
            .unwrap()
            .core_ir;
        let (CoreExpr::List { type_ref, .. } | CoreExpr::Binary { type_ref, .. }) =
            &mut core.functions[0].body
        else {
            panic!("list")
        };
        *type_ref = seseragi_lowering::CoreType::Named {
            name: "native/main::List".into(),
            arguments: vec![],
        };
        let error = emit_rust_module(&core, "probe").unwrap_err();
        assert!(error.origin.is_some());
    }
}

#[test]
fn unicode_character_program_preserves_scalar_operations_patterns_and_dictionaries() {
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/char-unicode.ssrg");
    run(
        source,
        r#"[0, 955, 128578, 1114111, 0, 55295, -1, -1, 57344, 1114111, -1, -1, -1, 955, -1, 0, 1, 2, 3, 4, 2, True, False, True, True, True, True, True, True, True, True, λ, 🙂, 🙂, '\0', '\b', '\f', '\u{1F}', '\u{7F}', '\u{9F}', '\'', '\\', '\n', '́', '🙂']"#,
    );
}

#[test]
fn float_values_preserve_ieee_arithmetic_selected_evidence_and_canonical_display() {
    run(
        include_str!("../../../examples/spec/fixtures/rust-backend/float-values.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/float-values.stdout").trim_end(),
    );
}

#[test]
fn float_evidence_rejects_mismatched_canonical_constraints() {
    let core = compile_core_module(CompileInput::new(
        "float.ssrg",
        "native/main",
        "pub fn probe -> Float = 1.25 + 2.5\n",
    ))
    .unwrap()
    .core_ir;
    assert!(emit_rust_module(&core, "probe").is_ok());
    for mutation in 0..5 {
        let mut changed = core.clone();
        let CoreExpr::Binary { evidence, .. } = &mut changed.functions[0].body else {
            panic!("binary")
        };
        match mutation {
            0 => {
                evidence[0].constraint.arguments[0] = seseragi_lowering::CoreType::Named {
                    name: "Int".into(),
                    arguments: vec![],
                }
            }
            1 => evidence[0].constraint.trait_identity = Some("user::Add".into()),
            2 => evidence[0].constraint.name = "Sub".into(),
            3 => {
                let seseragi_lowering::CoreInstanceEvidence::Standard { type_arguments, .. } =
                    &mut evidence[0].evidence
                else {
                    panic!("standard")
                };
                type_arguments.push(seseragi_lowering::CoreType::Named {
                    name: "Float".into(),
                    arguments: vec![],
                });
            }
            _ => {
                let seseragi_lowering::CoreInstanceEvidence::Standard { identity, .. } =
                    &mut evidence[0].evidence
                else {
                    panic!("standard")
                };
                *identity = "std/float::Sub".into();
            }
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(error.origin.is_some());
    }
}

#[test]
fn generic_arithmetic_uses_the_selected_user_method() {
    run(
        r#"
type Mark = | Mark Float
instance Add<Mark, Mark, Mark> {
  fn add left: Mark -> right: Mark -> Mark = Mark 42.0
}
fn plus<A> left: A -> right: A -> A where Add<A, A, A> = left + right
pub fn probe -> Float = match plus (Mark 1.0) (Mark 2.0) { Mark value -> value }
"#,
        "42.0",
    );
}

#[test]
fn float_canonical_dispatch_rejects_another_trait_or_method() {
    let core = compile_core_module(CompileInput::new(
        "float.ssrg",
        "native/main",
        "pub fn probe -> String = show 1.0\n",
    ))
    .unwrap()
    .core_ir;
    for method in [false, true] {
        let mut changed = core.clone();
        let CoreExpr::Call {
            trait_dispatch: Some(dispatch),
            ..
        } = &mut changed.functions[0].body
        else {
            panic!("dispatch")
        };
        if method {
            dispatch.method = "debug".into()
        } else {
            dispatch.trait_identity = "user::Show".into()
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(error.origin.is_some());
    }
}

#[test]
fn float_dictionary_arithmetic_preserves_a_user_methods_first_source_stage() {
    let artifact = emit(
        r#"
type Mark = | Mark Int
instance Add<Mark, Mark, Mark> {
  fn add left: Mark -> (Mark -> Mark) = {
    let bad = 9007199254740991 + 1
    \right -> Mark 0
  }
}
pub fn probe -> Mark = (Mark 0) + (Mark (9007199254740991 * 9007199254740991))
"#,
    );
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&artifact.source))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(!output.status.success());
    // The method's first stage raises the safe-range defect, before the
    // second source argument can raise its different i64 multiplication defect.
    assert!(String::from_utf8_lossy(&output.stderr).contains("Seseragi Int overflow"));
}

#[test]
fn integer_bases_patterns_and_safe_boundaries_use_shared_canonical_values() {
    run(
        include_str!("../../../examples/spec/fixtures/rust-backend/integer-literals.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/integer-literals.stdout")
            .trim_end(),
    );
}

#[test]
fn char_intrinsics_preserve_the_full_domain_without_per_call_callable_nodes() {
    let artifact = emit(include_str!(
        "../../../examples/spec/fixtures/rust-backend/char-unicode.ssrg"
    ));
    let name = rust_id("native/main::pointRoundtrip");
    let direct = artifact
        .source
        .lines()
        .find(|line| line.starts_with(&format!("fn {name}_direct(")))
        .unwrap();
    assert!(direct.contains("native_char_from_code_point("));
    assert!(!direct.contains("V::fun("));
    let end = artifact.source.rfind("fn main()").unwrap();
    let harness = r#"
fn main() {
 let before=ALLOCATIONS.with(Cell::get);
 let mut checksum=0_i64;
 for point in 0..=0x110000_i64 {
  let expected=if point <= 0x10ffff && !(0xd800..=0xdfff).contains(&point) { point } else {-1};
  let value=<ROUNDTRIP>_direct(vec![],checked_int(point));
  let V::Int(value)=value else {panic!("wrong return type")};
  assert_eq!(value,expected,"point={point:X}");
  checksum+=value;
 }
 assert_eq!(checksum,<CHECKSUM>);
 // At most the result Maybe node; no transient callable node for each intrinsic.
 assert!(ALLOCATIONS.with(Cell::get)-before <= 0x110001);
 clear_dictionaries(); assert_eq!(LIVE.with(Cell::get),0);
}
"#
    .replace("<ROUNDTRIP>", &name)
    .replace("<CHECKSUM>", "620506872831");
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&(artifact.source[..end].to_owned() + &harness)))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn generic_character_comparisons_use_the_selected_user_dictionary() {
    run(
        r#"
type Mark = | Mark Int
instance Eq<Mark> {
  fn eq left: Mark -> right: Mark -> Bool = True
}
fn same<A> left: A -> right: A -> Bool where Eq<A> = left == right
pub fn probe -> Bool = same (Mark 1) (Mark 2)
"#,
        "True",
    );
}

#[test]
fn primitive_partial_binding_rejects_unproven_staging_and_source_arity() {
    for source in [
        "fn same<A> left: A -> right: A -> Bool where Eq<A> = left == right\npub fn probe -> Bool = { let partial = same 'λ'; partial 'λ' }\n",
        "fn same<A> left: A -> right: A -> A where Add<A, A, A> = left + right\npub fn probe -> Float = { let partial = same 1.25; partial 2.5 }\n",
    ] {
    let core = compile_core_module(CompileInput::new("char.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    assert!(emit_rust_module(&core, "probe").is_ok());
    for mutation in 0..5 {
        let mut changed = core.clone();
        let function = changed
            .functions
            .iter_mut()
            .find(|function| function.symbol.ends_with("::probe"))
            .unwrap();
        let CoreExpr::Sequence { statements, .. } = &mut function.body else {
            panic!("expected sequence")
        };
        let seseragi_lowering::CoreStatement::PureLet { value, .. } = &mut statements[0] else {
            panic!("expected partial binding")
        };
        let CoreExpr::Call {
            callee,
            arguments,
            evidence,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            origin,
            ..
        } = value
        else {
            panic!("expected partial call")
        };
        match mutation {
            0 => deferred_evidence_parameters.push(seseragi_lowering::CoreType::Named {
                name: "Char".into(),
                arguments: vec![],
            }),
            1 => deferred_evidence_type_constructor_parameters.push("F".into()),
            2 => evidence[0].constraint.trait_identity = Some("user::Eq".into()),
            3 => *callee = "unknown::same".into(),
            _ => arguments.push(CoreExpr::Char {
                value: "λ".into(),
                origin: origin.clone(),
            }),
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001", "mutation={mutation}");
        assert!(error.origin.is_some());
    }
}
}

#[test]
fn malformed_char_values_and_nominal_standard_evidence_are_rejected() {
    let mut core = compile_core_module(CompileInput::new(
        "char.ssrg",
        "native/main",
        "pub fn probe -> Char = 'λ'\n",
    ))
    .unwrap()
    .core_ir;
    let CoreExpr::Char { value, .. } = &mut core.functions[0].body else {
        panic!("expected Char")
    };
    *value = "e\u{301}".into();
    let error = emit_rust_module(&core, "probe").unwrap_err();
    assert_eq!(error.code, "SES-RUST-0001");
    assert!(error.origin.is_some());
    let mut core = compile_core_module(CompileInput::new(
        "char.ssrg",
        "native/main",
        "pub fn probe -> String = show 'λ'\n",
    ))
    .unwrap()
    .core_ir;
    let CoreExpr::Call { evidence, .. } = &mut core.functions[0].body else {
        panic!("expected call")
    };
    evidence[0].constraint.arguments[0] = seseragi_lowering::CoreType::ExternalNamed {
        name: "Char".into(),
        canonical: "user::Char".into(),
        arguments: vec![],
    };
    let error = emit_rust_module(&core, "probe").unwrap_err();
    assert_eq!(error.code, "SES-RUST-0001");
    assert!(error.message.contains("canonical Char"));
}

#[test]
fn scalar_loop_operators_preserve_nested_arguments_ordering_and_negation() {
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/scalar-tail-operators.ssrg");
    let artifact = emit(source);
    let walk = rust_id("native/main::walk");
    let direct = artifact
        .source
        .lines()
        .find(|line| line.starts_with(&format!("fn {walk}_direct(")))
        .unwrap();
    assert!(!direct.contains(".clone()"));
    assert!(direct.contains("int_less_equal_value("));
    assert!(direct.contains("int_less_value("));
    assert!(direct.contains("int_multiply_value("));
    assert!(direct.contains(".checked_neg()"));
    run(source, "(4, -4, 9007199254740991)");
}

#[test]
fn scalar_global_reads_preserve_defect_order_zero_step_and_partial_lifetimes() {
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/scalar-global.ssrg");
    let artifact = emit(source);
    let sum = rust_id("native/main::sum");
    let retained = rust_id("native/main::retained");
    let direct = artifact
        .source
        .lines()
        .find(|l| l.starts_with(&format!("fn {sum}_direct(")))
        .unwrap();
    assert!(direct.contains("get_global_int(0)"));
    assert!(direct.contains("int_add_value("));
    assert!(!direct.contains("binary(") && !direct.contains(".clone()"));
    let fallback = artifact
        .source
        .lines()
        .find(|l| l.starts_with(&format!("fn {retained}_direct(")))
        .unwrap();
    assert!(fallback.contains("get_global(0)") && fallback.contains("binary("));
    let end = artifact.source.rfind("fn main()").unwrap();
    let mut generated = artifact.source[..end].to_owned();
    for (name, op, result) in [
        ("int_add_value", "+", "i64"),
        ("int_subtract_value", "-", "i64"),
        ("int_equal_value", "==", "bool"),
    ] {
        let marker = format!("fn {name}(a: i64, b: i64) -> {result} {{");
        assert_eq!(generated.matches(&marker).count(), 1);
        generated = generated.replace(&marker, &format!("{marker} TRACE.with(|trace| trace.borrow_mut().push(format!(\"{op}:{{a}}:{{b}}\")));"));
    }
    let marker = "fn read_global<T>(index: usize, read: impl FnOnce(&V) -> T) -> T {";
    assert_eq!(generated.matches(marker).count(), 1);
    generated = generated.replace(
        marker,
        &format!(
            "{marker} TRACE.with(|trace| trace.borrow_mut().push(format!(\"global:{{index}}\")));"
        ),
    );
    generated.push_str(&r#"
thread_local! { static TRACE: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) }; }
fn observe(f: fn(Vec<V>,V,V)->V,n:i64,seed:i64)->(Result<i64,String>,Vec<String>) {
 TRACE.with(|t|t.borrow_mut().clear());
 let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||f(vec![],checked_int(n),checked_int(seed))));
 let result=match result { Ok(V::Int(n))=>Ok(n), Ok(_)=>panic!("wrong result"), Err(e)=>Err(e.downcast_ref::<String>().cloned().or_else(||e.downcast_ref::<&str>().map(|s|(*s).into())).unwrap()) };
 (result,TRACE.with(|t|t.borrow().clone()))
}
fn main() {
 std::panic::set_hook(Box::new(|_|{}));
 for n in [0,1] {
  let actual=observe(<SUM>_direct,n,42);let ordinary=observe(<RETAINED>_direct,n,42);assert_eq!(actual,ordinary);
  if n==0 { assert_eq!(actual.0,Ok(42));assert!(!actual.1.iter().any(|s|s.starts_with("global:"))); }
  else { assert!(actual.0.unwrap_err().contains("uninitialized global"));assert_eq!(actual.1.last().unwrap(),"global:0"); }
 }
 initialize_globals();
 const MAX:i64=9007199254740991;let mut cases=Vec::new();
 for n in 0..=10 {for seed in [0,1,MAX,MAX-1,-MAX,-MAX+1] {cases.push((n,seed));}}
 cases.extend([(-MAX,0),(-MAX,-MAX),(-MAX,MAX)]);
 for (n,seed) in cases {
  let actual=observe(<SUM>_direct,n,seed);let ordinary=observe(<RETAINED>_direct,n,seed);assert_eq!(actual,ordinary,"n={n} seed={seed}");
  if n>=0 {let exact=i128::from(seed)+i128::from(n)*3;if (-i128::from(MAX)..=i128::from(MAX)).contains(&exact) {assert_eq!(actual.0,Ok(exact as i64));}else {assert!(actual.0.unwrap_err().contains("Int overflow"));}}
  else {assert!(actual.0.unwrap_err().contains("Int overflow"));assert_eq!(actual.1.last().unwrap(),&format!("-:{}:1",-MAX));}
 }
 let partial=<SUM>(vec![]).apply(checked_int(40));assert_eq!(partial.apply(checked_int(1)).render(),"121");assert_eq!(partial.apply(checked_int(2)).render(),"122");drop(partial);
 let result=<PROBE>(vec![]).apply(V::Unit);assert_eq!(result.render(),"(6, 7, 4, 22, True)");drop(result);
 clear_globals();clear_dictionaries();assert_eq!(LIVE.with(Cell::get),0);
 println!("69 scalar/global traces agree; zero-step, uninitialized, partial and probe pass");
}
"#.replace("<SUM>",&sum).replace("<RETAINED>",&retained).replace("<PROBE>",&rust_id(&artifact.metadata.entry)));
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&generated))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn scalar_tail_helpers_preserve_abi_checked_boundaries_and_primitive_order() {
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/scalar-tail-loop.ssrg");
    let artifact = emit(source);
    let sum = rust_id("native/main::sum");
    let retained = rust_id("native/main::retained");
    let direct = artifact
        .source
        .lines()
        .find(|line| line.starts_with(&format!("fn {sum}_direct(")))
        .unwrap();
    assert!(direct.contains("loop {"));
    assert!(direct.contains("int_add_value("));
    assert!(!direct.contains(".clone()"));
    assert!(!direct.contains("binary("));
    assert!(!direct.contains("Shared::"));
    let fallback = artifact
        .source
        .lines()
        .find(|line| line.starts_with(&format!("fn {retained}_direct(")))
        .unwrap();
    assert!(fallback.contains(".clone()") && fallback.contains("binary("));
    let main = artifact.source.rfind("fn main()").unwrap();
    let mut generated = artifact.source[..main].to_owned();
    for (name, op, result) in [
        ("int_add_value", "+", "i64"),
        ("int_subtract_value", "-", "i64"),
        ("int_equal_value", "==", "bool"),
    ] {
        let marker = format!("fn {name}(a: i64, b: i64) -> {result} {{");
        assert_eq!(generated.matches(&marker).count(), 1);
        generated = generated.replace(
            &marker,
            &format!("{marker} TRACE.with(|trace| trace.borrow_mut().push((\"{op}\", a, b)));"),
        );
    }
    let harness = r#"
thread_local! { static TRACE: RefCell<Vec<(&'static str, i64, i64)>> = const { RefCell::new(Vec::new()) }; }
fn outcome(f: fn(Vec<V>, V, V) -> V, n: i64, seed: i64) -> (Result<i64, String>, Vec<(&'static str, i64, i64)>) {
 TRACE.with(|t| t.borrow_mut().clear());
 let value = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(vec![], checked_int(n), checked_int(seed))));
 let outcome = match value {
  Ok(V::Int(n)) => Ok(n),
  Ok(_) => panic!("wrong result"),
  Err(e) => Err(e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| (*s).into())).unwrap()),
 };
 (outcome, TRACE.with(|t| t.borrow().clone()))
}

fn main() {
 std::panic::set_hook(Box::new(|_| {}));
 const MAX: i64 = 9_007_199_254_740_991;
 let mut cases = Vec::new();
 for n in 0..=10 { for seed in [0, 1, MAX, MAX-1, -MAX, -MAX+1] { cases.push((n,seed)); } }
 cases.extend([(-MAX,0),(-MAX,-MAX),(-MAX,MAX)]);
 for (n,seed) in cases {
  let scalar = outcome(<SUM>_direct,n,seed);
  let ordinary = outcome(<RETAINED>_direct,n,seed);
  assert_eq!(scalar, ordinary, "n={n} seed={seed}");
  if n >= 0 {
   let exact = i128::from(seed) + i128::from(n) * i128::from(n+1) / 2;
   if (-i128::from(MAX)..=i128::from(MAX)).contains(&exact) { assert_eq!(scalar.0, Ok(exact as i64)); }
   else { assert_eq!(scalar.0.as_ref().map_err(String::as_str), Err("Seseragi Int overflow")); }
  } else {
   assert_eq!(scalar.0.as_ref().map_err(String::as_str), Err("Seseragi Int overflow"));
   assert_eq!(scalar.1.last(), Some(&("-", -MAX, 1)));
  }
 }
 let partial = <SUM>(vec![]).apply(checked_int(40));
 assert_eq!(partial.apply(checked_int(1)).render(), "821");
 assert_eq!(partial.apply(checked_int(2)).render(), "822");
 drop(partial);
 let result = <PROBE>(vec![]).apply(V::Unit);
 assert_eq!(result.render(), "(821, 822, 821, True, 35, 42, 3628800)");
 drop(result); clear_dictionaries(); assert_eq!(LIVE.with(Cell::get), 0);
}
"#.replace("<SUM>", &sum).replace("<RETAINED>", &retained).replace("<PROBE>", &rust_id(&artifact.metadata.entry));
    generated.push_str(&harness);
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&generated))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
fn run(source: &str, expected: &str) {
    let artifact = emit(source);
    let scratch = Scratch::new();
    // Keep static dictionaries between invocations, but ensure their retained
    // graph is bounded. Explicit teardown must release the entire runtime.
    let main = artifact.source.rfind("fn main()").unwrap();
    let generated = format!("{}\nfn main() {{ let mut warm = None; for n in 0..10000 {{ let result = {}(vec![]).apply(V::Unit); if n == 0 {{ println!(\"{{}}\", result.render()); }} drop(result); let live = LIVE.with(Cell::get); if let Some(first) = warm {{ assert_eq!(live, first); }} else {{ warm = Some(live); }} }} clear_dictionaries(); assert_eq!(LIVE.with(Cell::get), 0); }}", &artifact.source[..main], rust_id(&artifact.metadata.entry));
    let output = Command::new(scratch.compile(&generated))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), expected);
}
fn rust_id(name: &str) -> String {
    format!(
        "v_{}",
        name.bytes().map(|b| format!("{b:02x}")).collect::<String>()
    )
}

#[test]
fn checked_fixtures_execute_without_bun_or_ts() {
    let cases = [
        (
            include_str!("../../../examples/spec/fixtures/rust-backend/ownership.ssrg"),
            "(42, 42, 11, 12)",
        ),
        (
            include_str!("../../../examples/spec/fixtures/rust-backend/shared-values.ssrg"),
            "(shared, shared, shared)",
        ),
        (
            include_str!("../../../examples/spec/fixtures/rust-backend/recursive-adt.ssrg"),
            "(3, 5)",
        ),
        (
            include_str!("../../../examples/spec/fixtures/rust-backend/structural-record.ssrg"),
            "(row, row)",
        ),
        (
            include_str!("../../../examples/spec/fixtures/rust-backend/higher-kinded.ssrg"),
            "(42, 42)",
        ),
        (
            include_str!("../../../examples/spec/fixtures/rust-backend/nested-dictionary.ssrg"),
            "(42, 42)",
        ),
        (
            include_str!("../../../examples/spec/fixtures/rust-backend/let-shadowing.ssrg"),
            "(42, 42, 42, 42)",
        ),
    ];
    for (source, expected) in cases {
        run(source, expected);
    }
}

#[test]
fn parameter_dependent_dictionaries_do_not_reuse_another_instance() {
    let source = format!("{}\npub fn probe -> (Int, Int, Int, Int) = (nested 35, returned 35, parameterized 35, other 25)\n", include_str!("../../../examples/spec/fixtures/rust-backend/dictionaries.ssrg"));
    run(&source, "(42, 84, 42, 42)");
    let artifact = emit(&source);
    assert!(artifact.metadata.closed_dictionary_sites > 0);
}

#[test]
fn direct_calls_keep_partial_and_returned_function_abi() {
    let source = "fn add a: Int -> b: Int -> Int = a + b\nfn returned seed: Int -> (Int -> Int) = \\value -> seed + value\npub fn probe -> (Int, Int, Int, Int) = {\n let partially = add 35\n let alias = add\n (add 35 7, partially 7, returned 35 7, alias 35 7)\n}\n";
    run(source, "(42, 42, 42, 42)");
    let core = compile_core_module(CompileInput::new("main.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    assert_eq!(
        known_pure_call_shape(&core, "native/main::add", 1),
        Some(CoreCallShape::Partial)
    );
    assert_eq!(
        known_pure_call_shape(&core, "native/main::add", 2),
        Some(CoreCallShape::Saturated)
    );
    assert_eq!(
        known_pure_call_shape(&core, "native/main::returned", 2),
        Some(CoreCallShape::ReturnedApplication)
    );
    assert_eq!(known_pure_call_shape(&core, "unresolved", 2), None);
}

#[test]
fn captures_bindings_and_hidden_evidence_are_analyzed_separately() {
    let core = compile_core_module(CompileInput::new(
        "main.ssrg",
        "native/main",
        include_str!("../../../examples/spec/fixtures/rust-backend/calls.ssrg"),
    ))
    .unwrap()
    .core_ir;
    let capture = core
        .functions
        .iter()
        .find(|f| f.symbol.ends_with("::captureOne"))
        .unwrap();
    let free = core_free_variables(&capture.body).unwrap();
    assert!(free.names.contains("seed"));
    assert!(!free.names.contains("unused"));
    let source = "pub fn probe -> Int = { let seed = 40; let value = { let seed = seed + 1; seed + 1 }; value }\n";
    let core = compile_core_module(CompileInput::new("main.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    assert!(core_free_variables(&core.functions[0].body)
        .unwrap()
        .names
        .is_empty());
    run(source, "42");
}

#[test]
fn escaped_callback_does_not_retain_an_unused_array() {
    let source = format!(
        "{}\npub fn probe -> (Unit -> Int) = captureOne 42 [1, 2, 3]\n",
        include_str!("../../../examples/spec/fixtures/rust-backend/calls.ssrg")
    );
    let artifact = emit(&source);
    let main = artifact.source.rfind("fn main()").unwrap();
    let generated = format!("{}\nfn main() {{ let callback = {}(vec![]).apply(V::Unit); assert_eq!(LIVE.with(Cell::get), 1, \"only the escaped function remains\"); let V::Fun(function) = &callback else {{ panic!() }}; assert_eq!(function.get().captures.len(), 1); assert_eq!(callback.apply(V::Unit).render(), \"42\"); drop(callback); assert_eq!(LIVE.with(Cell::get), 0); }}", &artifact.source[..main], rust_id(&artifact.metadata.entry));
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&generated))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn unsupported_features_have_target_codes_and_source_ranges() {
    for source in [
        "pub fn probe -> Float = 1.5 ** 2.0\n",
        "pub fn probe -> List<Int> = `[n | n <- `[1, 2]]\n",
    ] {
        let core = compile_core_module(CompileInput::new("bad.ssrg", "native/bad", source))
            .unwrap()
            .core_ir;
        let error = emit_rust_module(&core, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        let origin = error.origin.unwrap();
        assert!(origin.end > origin.start);
    }
    let source =
        "import * as effects from \"std/effect\"\npub effect fn probe = effects.yieldNow ()\n";
    let core = compile_core_module(CompileInput::new("bad.ssrg", "native/bad", source))
        .unwrap()
        .core_ir;
    assert!(emit_rust_module(&core, "probe")
        .unwrap_err()
        .message
        .contains("yieldNow"));
}

#[test]
fn defects_keep_checked_int_bounds_and_application_order() {
    run("pub fn probe -> Int = -7 + 49\n", "42");
    for (source, expected) in [
        ("pub fn probe -> Int = 9007199254740991 + 1\n", "Seseragi Int overflow"),
        // returned's body must run before evaluating the second argument.
        ("fn returned seed: Int -> (Int -> Int) = { let bad = 9007199254740991 + 1; \\value -> value }\npub fn probe -> Int = returned 0 (9007199254740991 * 9007199254740991)\n", "Seseragi Int overflow"),
    ] {
        let artifact = emit(source); let scratch = Scratch::new();
        let output = Command::new(scratch.compile(&artifact.source)).env("PATH", "").output().unwrap();
        assert!(!output.status.success()); assert!(String::from_utf8_lossy(&output.stderr).contains(expected));
    }
}

#[test]
fn closed_fact_excludes_parameter_evidence() {
    let module = compile_core_module(CompileInput::new(
        "main.ssrg",
        "native/main",
        include_str!("../../../examples/spec/fixtures/rust-backend/dictionaries.ssrg"),
    ))
    .unwrap()
    .core_ir;
    assert!(!is_closed_local_evidence(
        &module,
        &seseragi_lowering::CoreInstanceEvidence::Parameter { index: 0 }
    ));
    let function = module
        .functions
        .iter()
        .find(|f| f.symbol.ends_with("::parameterized"))
        .unwrap();
    let CoreExpr::Call { evidence, .. } = &function.body else {
        panic!("expected trait call");
    };
    assert!(!is_closed_local_evidence(&module, &evidence[0].evidence));
    let captured = module
        .functions
        .iter()
        .find(|f| f.symbol.ends_with("::captured"))
        .unwrap();
    assert!(core_free_variables(&captured.body)
        .unwrap()
        .evidence_parameters
        .contains(&0));
}

#[test]
fn checked_effect_programs_run_on_a_small_stack_and_release_the_native_heap() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend-research/deep-effect.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend-research/ref-cycle.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/list-effects.ssrg"),
    ] {
        let core =
            compile_core_module(CompileInput::new("main.ssrg", "native/main", source)).unwrap();
        let artifact = emit_rust_module(&core.core_ir, "main").unwrap();
        let main = artifact.source.rfind("fn main()").unwrap();
        let generated = format!("{}\nfn main() {{ let code = std::thread::Builder::new().stack_size(256 * 1024).spawn(|| run_native_entry(|| {}(vec![]).apply(V::Unit), {})).unwrap().join().unwrap(); assert_eq!(code, 0); }}", &artifact.source[..main], rust_id(&artifact.metadata.entry), artifact.metadata.capabilities.iter().any(|x| x == "ref"));
        let scratch = Scratch::new();
        let output = Command::new(scratch.compile(&generated))
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stderr).contains("managed-live=0"));
    }
}

#[test]
fn pure_tail_recursion_preserves_snapshots_partial_calls_and_changing_evidence() {
    for (source, expected) in [
        (
            include_str!("../../../examples/spec/fixtures/rust-backend/tail-recursion.ssrg"),
            "500000500000",
        ),
        (
            include_str!("../../../examples/spec/fixtures/rust-backend/tail-capture.ssrg"),
            "(100009, 100019)",
        ),
        (
            include_str!("../../../examples/spec/fixtures/rust-backend/tail-evidence.ssrg"),
            "(35, 42)",
        ),
    ] {
        let artifact = emit(source);
        assert_eq!(artifact.metadata.self_tail_calls, 1);
        let main = artifact.source.rfind("fn main()").unwrap();
        let generated = format!(
            "{}\nfn main() {{ std::thread::Builder::new().stack_size(256 * 1024).spawn(|| {{ let result = {}(vec![]).apply(V::Unit); assert_eq!(result.render(), {expected:?}); drop(result); clear_dictionaries(); assert_eq!(LIVE.with(Cell::get), 0); }}).unwrap().join().unwrap(); }}",
            &artifact.source[..main], rust_id(&artifact.metadata.entry),
        );
        let scratch = Scratch::new();
        let output = Command::new(scratch.compile(&generated))
            .env("PATH", "")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    // Returning a lambda is not a tail call at the point of constructing it.
    let artifact = emit("fn later n: Int -> (Unit -> Int) = \\_ -> if n == 0 then 42 else later (n - 1) ()\npub fn probe -> Int = later 5 ()\n");
    assert_eq!(artifact.metadata.self_tail_calls, 0);
    run("fn later n: Int -> (Unit -> Int) = \\_ -> if n == 0 then 42 else later (n - 1) ()\npub fn probe -> Int = later 5 ()\n", "42");
}

#[test]
fn monomorphic_globals_keep_values_patterns_closures_and_source_arity() {
    let artifact = emit(
        r#"
struct Pair { left: Int, right: Int }
let base: Int = laterFunction ()
let (left, right): (Int, Int) = (20, 22)
let pair: Pair = Pair { left: left, right: right }
let retained: Unit -> Int = \_ -> later
let partial: Int -> Int = returned pair.left
let later: Int = 42
fn laterFunction -> Int = 7
fn returned n: Int -> (Int -> Int) = \m -> n + m
pub fn probe -> (Int, Int, Int) = (base, retained (), partial pair.right)
"#,
    );
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&artifact.source))
        .env("PATH", "")
        .env("SESERAGI_NATIVE_METRICS", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"(7, 42, 42)\n");
    assert!(String::from_utf8_lossy(&output.stderr).contains("managed-live=0"));
}

#[test]
fn globals_reject_unproved_polymorphism_before_emission() {
    let source = "let identity = \\x -> x\npub fn probe -> Int = identity 42\n";
    let core =
        compile_core_module(CompileInput::new("global.ssrg", "native/main", source)).unwrap();
    let error = emit_rust_module(&core.core_ir, "probe").err().unwrap();
    assert!(error.message.contains("global"), "{error:?}");
    assert!(error.origin.is_some());
    // The shared frontend eta-expands constrained callable aliases. Those keep
    // their source parameters and evidence ABI; zero-parameter values don't.
    let source = "fn render<A> value: A -> String where Show<A> = show value\nlet retainedValue = render\npub fn probe -> String = retainedValue 42\n";
    let mut core = compile_core_module(CompileInput::new("global.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    let artifact = emit_rust_module(&core, "probe").unwrap();
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&artifact.source))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, b"42\n");
    core.functions
        .iter_mut()
        .find(|f| f.symbol.ends_with("::retainedValue"))
        .unwrap()
        .parameters
        .clear();
    assert!(emit_rust_module(&core, "probe")
        .err()
        .unwrap()
        .message
        .contains("without source parameters"));
}

#[test]
fn global_cleanup_handles_partial_initialization_and_deep_ref_graphs() {
    let artifact = emit("pub fn probe -> Int = 42\n");
    let main = artifact.source.rfind("fn main()").unwrap();
    let harness = r#"
fn main() {
    std::thread::Builder::new().stack_size(256 * 1024).spawn(|| {
        let code = run_native_pure_entry(|| {
            let mut chain = V::Unit;
            for _ in 0..100_000 { chain = V::adt("node", chain); }
            set_global(0, chain);
            let cell = V::reference(V::Unit);
            cell.write_reference(V::fun(vec![cell.clone()], |c, _| c[0].clone()));
            set_global(1, cell);
            assert_eq!(collect_cycles().unwrap().reclaimed, 0);
            panic!("initializer stopped");
        }, true);
        assert_eq!(code, 70);
        assert_eq!(LIVE.with(Cell::get), 0);
        assert!(GLOBALS.with(|v| v.borrow().is_empty()));
        assert_eq!(run_native_pure_entry(|| { set_global(0, V::text("again")); get_global(0) }, true), 0);
        assert_eq!(LIVE.with(Cell::get), 0);
    }).unwrap().join().unwrap();
}
"#;
    let scratch = Scratch::new();
    let source = format!("{}\n{harness}", &artifact.source[..main]);
    let output = Command::new(scratch.compile(&source))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"again\n");
    assert!(String::from_utf8_lossy(&output.stderr).contains("initializer stopped"));
}
