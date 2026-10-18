use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreExpr, CoreInstanceEvidence, CoreStatement, CoreType};
use std::{fs, process::Command};

fn execute(source: &str, name: &str, stdout: &str, code: i32) {
    let core = compile_core_module(CompileInput::new("operator-λ.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    let artifact = emit_rust_module(&core, "main").unwrap();
    let directory =
        std::env::temp_dir().join(format!("seseragi-operator-{name}-{}", std::process::id()));
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
fn primitive_and_selected_operator_partials_keep_values_and_reuse() {
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/operator-partials.ssrg"),
        "values",
        include_str!("../../../examples/spec/fixtures/rust-backend/operator-partials.stdout"),
        0,
    );
}

#[test]
fn a_saved_operator_runs_its_selected_first_stage_before_the_next_statement() {
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/operator-partial-defect.ssrg"),
        "selected-defect",
        "",
        70,
    );
    execute("pub effect fn main = do { let saved: Int -> Int = add (9007199254740991 + 1); println \"unreachable\" }", "primitive-defect", "", 70);
}

#[test]
fn selected_operator_edges_preserve_cold_effects_finalizers_and_cycles() {
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/operator-partial-effects.ssrg"),
        "effects",
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/operator-partial-effects.stdout"
        ),
        0,
    );
}

#[test]
fn a_selected_higher_kinded_child_dictionary_runs_in_the_first_stage() {
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/operator-partial-children.ssrg"),
        "hkt-child",
        "",
        70,
    );
}

#[test]
fn primitive_arithmetic_refuses_unproved_identity_types_children_and_stages() {
    let source = "// 雫🙂\npub fn probe -> Int = { let saved: Int -> Int = add 40; saved 2 }";
    let core = compile_core_module(CompileInput::new("operator-λ.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    for mutation in 0..17 {
        let mut changed = core.clone();
        let CoreExpr::Sequence { statements, .. } = &mut changed.functions[0].body else {
            panic!()
        };
        let CoreStatement::PureLet {
            value:
                CoreExpr::Call {
                    arguments,
                    evidence,
                    trait_dispatch,
                    type_ref,
                    deferred_evidence_parameters,
                    deferred_evidence_type_constructor_parameters,
                    evidence_argument_index,
                    ..
                },
            ..
        } = &mut statements[0]
        else {
            panic!()
        };
        let bool_type = CoreType::Named {
            name: "Bool".into(),
            arguments: vec![],
        };
        match mutation {
            0 => evidence[0].constraint.trait_identity = Some("fake::Add".into()),
            1 => evidence[0].constraint.name = "Sub".into(),
            2 => evidence[0].constraint.arguments.pop().map(|_| ()).unwrap(),
            3 => evidence[0].constraint.arguments[0] = bool_type,
            4 => evidence[0].constraint.arguments[2] = CoreType::Hole,
            5 => {
                if let CoreInstanceEvidence::Standard { type_arguments, .. } =
                    &mut evidence[0].evidence
                {
                    type_arguments.push(bool_type);
                }
            }
            6 => {
                let child = evidence[0].clone();
                if let CoreInstanceEvidence::Standard {
                    evidence_arguments, ..
                } = &mut evidence[0].evidence
                {
                    evidence_arguments.push(child);
                }
            }
            7 => trait_dispatch.as_mut().unwrap().trait_identity = "fake::Add".into(),
            8 => trait_dispatch.as_mut().unwrap().method = "sub".into(),
            9 => *type_ref = bool_type,
            10 => {
                if let CoreType::Function { parameter, .. } = type_ref {
                    *parameter = Box::new(bool_type);
                }
            }
            11 => {
                if let CoreType::Function { result, .. } = type_ref {
                    *result = Box::new(bool_type);
                }
            }
            12 => deferred_evidence_parameters.push(bool_type),
            13 => deferred_evidence_type_constructor_parameters.push("F".into()),
            14 => *evidence_argument_index = Some(0),
            15 => arguments.push(arguments[0].clone()),
            _ => {
                evidence[0].constraint.arguments[1] = CoreType::ExternalNamed {
                    name: "Int".into(),
                    canonical: "fake::Int".into(),
                    arguments: vec![],
                }
            }
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001", "mutation {mutation}");
        assert_eq!(error.origin.unwrap().source, "operator-λ.ssrg");
    }
}

#[test]
fn primitive_arithmetic_does_not_replace_a_scoped_binder_named_int() {
    let source = "fn generic<A> value: A -> (A -> A) where Add<A, A, A> = add value\npub fn probe -> Int = { let saved = generic 40; saved 2 }";
    let core = compile_core_module(CompileInput::new("operator-λ.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    let mut changed = core;
    let function = changed
        .functions
        .iter_mut()
        .find(|f| f.symbol.ends_with("::generic"))
        .unwrap();
    // Alpha-rename the checked lexical binder, including its parameter and
    // constraint uses. Its spelling must never select the primitive ABI.
    fn rename(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::String(name) if name == "A" => *name = "Int".into(),
            serde_json::Value::Object(fields) => fields.values_mut().for_each(rename),
            serde_json::Value::Array(values) => values.iter_mut().for_each(rename),
            _ => (),
        }
    }
    let mut encoded = serde_json::to_value(&*function).unwrap();
    rename(&mut encoded);
    *function = serde_json::from_value(encoded).unwrap();
    emit_rust_module(&changed, "probe").unwrap();
    let function = changed
        .functions
        .iter_mut()
        .find(|f| f.symbol.ends_with("::generic"))
        .unwrap();
    let CoreExpr::Call { evidence, .. } = &mut function.body else {
        panic!()
    };
    evidence[0].evidence = CoreInstanceEvidence::Standard {
        identity: "std/int::Add".into(),
        type_arguments: vec![],
        evidence_arguments: vec![],
    };
    let error = emit_rust_module(&changed, "probe").unwrap_err();
    assert_eq!(error.code, "SES-RUST-0001");
    assert_eq!(error.origin.unwrap().source, "operator-λ.ssrg");
}
