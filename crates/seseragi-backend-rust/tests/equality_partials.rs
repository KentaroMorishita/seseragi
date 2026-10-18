use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{
    CoreExpr, CoreInstanceEvidence, CoreInstanceImplementation, CoreStatement, CoreType,
};
use std::{fs, process::Command};

fn execute(source: &str, name: &str, stdout: &str, code: i32) {
    let core = compile_core_module(CompileInput::new("equality-λ.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    let artifact = emit_rust_module(&core, "main").unwrap();
    let directory =
        std::env::temp_dir().join(format!("seseragi-equality-{name}-{}", std::process::id()));
    fs::create_dir(&directory).unwrap();
    let input = directory.join("program.rs");
    let exe = directory.join(if cfg!(windows) {
        "program.exe"
    } else {
        "program"
    });
    fs::write(&input, &artifact.source).unwrap();
    let built = Command::new("rustc")
        .args(["--edition=2021", "-O", "-o"])
        .arg(&exe)
        .arg(&input)
        .output()
        .unwrap();
    let result = built.status.success().then(|| {
        Command::new(&exe)
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1")
            .output()
            .unwrap()
    });
    fs::remove_dir_all(&directory).unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let result = result.unwrap();
    assert_eq!(
        result.status.code(),
        Some(code),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&result.stdout), stdout);
    assert!(String::from_utf8_lossy(&result.stderr).contains("managed-live=0"));
}

#[test]
fn equality_references_and_partial_values_preserve_selected_dictionaries() {
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/equality-partials.ssrg"),
        "values",
        include_str!("../../../examples/spec/fixtures/rust-backend/equality-partials.stdout"),
        0,
    );
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/equality-partial-trace.ssrg"),
        "trace-values",
        include_str!("../../../examples/spec/fixtures/rust-backend/equality-partial-trace.stdout"),
        0,
    );
}
#[test]
fn selected_equality_body_runs_before_the_next_statement() {
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/equality-partial-defect.ssrg"),
        "defect",
        "",
        70,
    );
    execute("type Mark = | Mark Int\ninstance Eq<Mark> { fn eq left: Mark -> (Mark -> Bool) = { let bad = 9007199254740991 + 1; \\right -> True } }\npub effect fn main = do { let op: Mark -> Mark -> Bool = (!=); let saved = op (Mark 1); println \"unreachable\" }", "negated-defect", "",70);
    for (operator, name) in [("==", "global-equal"), ("!=", "global-negated")] {
        let source = format!("type Mark = | Mark Int\ninstance Eq<Mark> {{ fn eq left: Mark -> (Mark -> Bool) = {{ let bad = 9007199254740991 + 1; \\right -> True }} }}\nlet selectedEq: Mark -> Mark -> Bool = ({operator})\nlet partial: Mark -> Bool = selectedEq (Mark 7)\npub effect fn main = println \"MAIN_EXECUTED_MARKER\"\n");
        execute(&source, name, "", 70);
    }
}

#[test]
fn equality_stages_keep_ref_cycles_cold_effects_and_finalizers() {
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/equality-partial-effects.ssrg"),
        "effects",
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/equality-partial-effects.stdout"
        ),
        0,
    );
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/equality-partial-children.ssrg"),
        "children",
        "",
        70,
    );
}

#[test]
fn intrinsic_equality_rejects_extra_standard_type_and_child_arguments() {
    let core = compile_core_module(CompileInput::new(
        "equality-λ.ssrg",
        "native/main",
        "pub fn probe -> Bool = 40 == 40",
    ))
    .unwrap()
    .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    for child in [false, true] {
        let mut changed = core.clone();
        let CoreExpr::Binary { evidence, .. } = &mut changed.functions[0].body else {
            panic!()
        };
        let extra = evidence[0].clone();
        let CoreInstanceEvidence::Standard {
            type_arguments,
            evidence_arguments,
            ..
        } = &mut evidence[0].evidence
        else {
            panic!()
        };
        if child {
            evidence_arguments.push(extra);
        } else {
            type_arguments.push(CoreType::Named {
                name: "Int".into(),
                arguments: vec![],
            });
        }
        assert!(
            emit_rust_module(&changed, "probe").is_err(),
            "accepted extra equality argument child={child}"
        );
    }
}

#[test]
fn equality_references_reject_counterfeit_owners_stages_and_evidence() {
    let source =
        "// 雫🙂\npub fn probe -> Bool = { let same: Int -> Int -> Bool = (==); same 40 40 }";
    let core = compile_core_module(CompileInput::new("equality-λ.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    for mutation in 0..16 {
        let mut changed = core.clone();
        let CoreExpr::Sequence { statements, .. } = &mut changed.functions[0].body else {
            panic!()
        };
        let CoreStatement::PureLet {
            value: CoreExpr::Variable {
                evidence, type_ref, ..
            },
            ..
        } = &mut statements[0]
        else {
            panic!()
        };
        match mutation {
            0 => evidence[0].constraint.trait_identity = Some("fake::Eq".into()),
            1 => evidence[0].constraint.name = "Ord".into(),
            2 => evidence[0].constraint.arguments.clear(),
            3 => evidence[0].constraint.arguments[0] = CoreType::Hole,
            4 => {
                evidence[0].constraint.arguments[0] = CoreType::ExternalNamed {
                    name: "Int".into(),
                    canonical: "fake::Int".into(),
                    arguments: vec![],
                }
            }
            5 => {
                let CoreInstanceEvidence::Standard { type_arguments, .. } =
                    &mut evidence[0].evidence
                else {
                    panic!()
                };
                type_arguments.push(CoreType::Hole);
            }
            6 => {
                let extra = evidence[0].clone();
                let CoreInstanceEvidence::Standard {
                    evidence_arguments, ..
                } = &mut evidence[0].evidence
                else {
                    panic!()
                };
                evidence_arguments.push(extra);
            }
            7 => {
                let CoreInstanceEvidence::Standard { identity, .. } = &mut evidence[0].evidence
                else {
                    panic!()
                };
                *identity = "std/char::Eq".into();
            }
            8 => evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 0 },
            9 => *type_ref = CoreType::Hole,
            10 => {
                *type_ref = CoreType::Named {
                    name: "Bool".into(),
                    arguments: vec![],
                }
            }
            11 => {
                let CoreType::Function { parameter, .. } = type_ref else {
                    panic!()
                };
                **parameter = CoreType::Hole;
            }
            12 => {
                let CoreType::Function { result, .. } = type_ref else {
                    panic!()
                };
                **result = CoreType::Named {
                    name: "Bool".into(),
                    arguments: vec![],
                };
            }
            13 => evidence.push(evidence[0].clone()),
            14 => evidence.clear(),
            _ => evidence[0].constraint.arguments.push(CoreType::Hole),
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001", "case {mutation}");
        assert_eq!(error.origin.unwrap().source, "equality-λ.ssrg");
    }
}

#[test]
fn fully_applied_equality_runs_its_first_stage_before_the_right_operand() {
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/equality-partial-trace.ssrg");
    let core = compile_core_module(CompileInput::new("equality-λ.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    for (entry, value, trace) in [
        (
            "binaryProbe",
            "True",
            "[\"leftValue\", \"firstStage\", \"rightValue\"]",
        ),
        (
            "negatedBinaryProbe",
            "False",
            "[\"leftValue\", \"firstStage\", \"rightValue\"]",
        ),
    ] {
        let mut rust = emit_rust_module(&core, entry).unwrap().source;
        let main = rust.rfind("fn main()").unwrap();
        rust.truncate(main);
        let id = |name: &str| {
            format!(
                "v_{}",
                format!("native/main::{name}")
                    .as_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            )
        };
        // Instrument only source helper entry, leaving generated Eq expressions intact.
        for name in ["firstStage", "leftValue", "rightValue"] {
            let start = rust.find(&format!("fn {}_direct(", id(name))).unwrap();
            let body = start + rust[start..].find("-> V {").unwrap() + "-> V {".len();
            rust.insert_str(
                body,
                &format!(" TRACE.with(|t| t.borrow_mut().push({name:?})); "),
            );
        }
        rust.push_str(&format!("\nthread_local! {{ static TRACE: RefCell<Vec<&'static str>> = const {{ RefCell::new(Vec::new()) }}; }}\nfn main() {{ let result = {}(vec![]).apply(V::Unit); assert_eq!(result.render(), {value:?}); TRACE.with(|t| assert_eq!(&*t.borrow(), &{trace})); drop(result); clear_dictionaries(); assert_eq!(LIVE.with(Cell::get), 0); }}", id(entry)));
        let directory = std::env::temp_dir().join(format!(
            "seseragi-equality-trace-{entry}-{}",
            std::process::id()
        ));
        fs::create_dir(&directory).unwrap();
        let file = directory.join("program.rs");
        let exe = directory.join("program");
        fs::write(&file, rust).unwrap();
        let built = Command::new("rustc")
            .args(["--edition=2021", "-O", "-o"])
            .arg(&exe)
            .arg(&file)
            .output()
            .unwrap();
        let executed = built
            .status
            .success()
            .then(|| Command::new(&exe).env("PATH", "").output().unwrap());
        fs::remove_dir_all(&directory).unwrap();
        assert!(
            built.status.success(),
            "{}",
            String::from_utf8_lossy(&built.stderr)
        );
        let executed = executed.unwrap();
        assert!(
            executed.status.success(),
            "{}",
            String::from_utf8_lossy(&executed.stderr)
        );
    }
}

#[test]
fn equality_rejects_unproven_method_contracts_before_emission() {
    let source = "// 雫🙂\ntype Mark = | Mark Int\ninstance Eq<Mark> { fn eq left: Mark -> right: Mark -> Bool = 1 == 1 }\npub fn probe -> Bool = Mark 1 == Mark 2";
    let core = compile_core_module(CompileInput::new("equality-λ.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    for mutation in 0..8 {
        let mut changed = core.clone();
        let instance = &mut changed.instances[0];
        let CoreInstanceImplementation::UserDefined { methods } = &mut instance.implementation
        else {
            panic!()
        };
        match mutation {
            0 => methods[0].parameters.clear(),
            1 => {
                let extra = methods[0].parameters[0].clone();
                methods[0].parameters.push(extra);
            }
            2 => methods[0].name = "compare".into(),
            3 => methods.push(methods[0].clone()),
            4 => {
                methods[0].parameters[0].type_ref = CoreType::Named {
                    name: "Int".into(),
                    arguments: vec![],
                }
            }
            5 => instance.supertrait_count = 1,
            6 => instance.arguments.clear(),
            _ => {
                let CoreExpr::Binary { type_ref, .. } = &mut methods[0].body else {
                    panic!()
                };
                *type_ref = CoreType::ExternalNamed {
                    name: "Bool".into(),
                    canonical: "fake::Bool".into(),
                    arguments: vec![],
                };
            }
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001", "case {mutation}");
        assert_eq!(error.origin.unwrap().source, "equality-λ.ssrg");
    }
}
