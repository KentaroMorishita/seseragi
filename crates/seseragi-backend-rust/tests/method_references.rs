use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreExpr, CoreInstanceEvidence, CoreType};
use std::{fs, process::Command};

fn execute(source: &str, name: &str, stdout: &str, code: i32) {
    let core = compile_core_module(CompileInput::new(
        "method-reference-λ.ssrg",
        "native/main",
        source,
    ))
    .unwrap()
    .core_ir;
    let artifact = emit_rust_module(&core, "main").unwrap();
    let directory = std::env::temp_dir().join(format!(
        "seseragi-method-reference-{name}-{}",
        std::process::id()
    ));
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
fn selected_bare_methods_run_their_first_stage_when_applied() {
    for (method, implementation, result) in [
        ("add", "instance Add<Mark, Mark, Mark> { fn add left: Mark -> (Mark -> Mark) = { let bad = 9007199254740991 + 1; \\right -> left } }", "Mark"),
        ("eq", "instance Eq<Mark> { fn eq left: Mark -> (Mark -> Bool) = { let bad = 9007199254740991 + 1; \\right -> True } }", "Bool"),
    ] {
        let source = format!("type Mark = | Mark Int\n{implementation}\npub effect fn main = do {{ let selected: Mark -> Mark -> {result} = {method}; let held = selected (Mark 1); println \"unreachable\" }}");
        execute(&source, method, "", 70);
    }
}

#[test]
fn bare_methods_preserve_stored_values_and_generalized_aliases() {
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/method-references.ssrg"),
        "values",
        include_str!("../../../examples/spec/fixtures/rust-backend/method-references.stdout"),
        0,
    );
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/method-reference-trace.ssrg"),
        "trace",
        include_str!("../../../examples/spec/fixtures/rust-backend/method-reference-trace.stdout"),
        0,
    );
}

#[test]
fn bare_method_captures_keep_refs_cold_effects_and_finalizers() {
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/method-reference-effects.ssrg"),
        "effects",
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/method-reference-effects.stdout"
        ),
        0,
    );
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/method-reference-children.ssrg"),
        "children",
        "",
        70,
    );
}

#[test]
fn selected_bare_method_rejects_forged_owner_types_children_and_stages() {
    let source = "type Mark = | Mark Int\ntrait Pick<A> { fn pick left: A -> (A -> String) }\ninstance Pick<Mark> { fn pick left: Mark -> (Mark -> String) = \\right -> \"selected\" }\npub fn probe -> (Mark -> Mark -> String) = pick";
    let core = compile_core_module(CompileInput::new("forged-λ.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    for mutation in 0..18 {
        let mut changed = core.clone();
        let CoreExpr::Call {
            evidence,
            trait_dispatch,
            type_ref,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            ..
        } = &mut changed.functions[0].body
        else {
            panic!("selected method must remain a value")
        };
        let original = evidence[0].clone();
        match mutation {
            0 => trait_dispatch.as_mut().unwrap().trait_identity = "elsewhere::trait(Pick)".into(),
            1 => trait_dispatch.as_mut().unwrap().method = "unknown".into(),
            2 => evidence[0].constraint.name = "Other".into(),
            3 => evidence[0].constraint.trait_identity = Some("elsewhere::trait(Pick)".into()),
            4 => evidence[0].constraint.arguments.clear(),
            5 => {
                evidence[0].constraint.arguments[0] = CoreType::ExternalNamed {
                    name: "Mark".into(),
                    canonical: "elsewhere::Mark".into(),
                    arguments: vec![],
                }
            }
            6 => evidence[0].constraint.arguments[0] = CoreType::Hole,
            7 => {
                *type_ref = CoreType::Named {
                    name: "String".into(),
                    arguments: vec![],
                }
            }
            8 => {
                let CoreType::Function { parameter, .. } = type_ref else {
                    panic!()
                };
                *parameter = Box::new(CoreType::Named {
                    name: "Int".into(),
                    arguments: vec![],
                });
            }
            9 => {
                let CoreType::Function { result, .. } = type_ref else {
                    panic!()
                };
                *result = Box::new(CoreType::Named {
                    name: "String".into(),
                    arguments: vec![],
                });
            }
            10 => evidence.push(original.clone()),
            11 => evidence.clear(),
            12 => deferred_evidence_parameters.push(CoreType::Named {
                name: "A".into(),
                arguments: vec![],
            }),
            13 => deferred_evidence_type_constructor_parameters.push("F".into()),
            14 => *evidence_argument_index = Some(0),
            15 => {
                let CoreInstanceEvidence::Local { type_arguments, .. } = &mut evidence[0].evidence
                else {
                    panic!()
                };
                type_arguments.push(CoreType::Named {
                    name: "Int".into(),
                    arguments: vec![],
                });
            }
            16 => {
                let CoreInstanceEvidence::Local {
                    evidence_arguments, ..
                } = &mut evidence[0].evidence
                else {
                    panic!()
                };
                evidence_arguments.push(original.clone());
            }
            17 => evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 0 },
            _ => unreachable!(),
        }
        assert!(
            emit_rust_module(&changed, "probe").is_err(),
            "accepted counterfeit {mutation}"
        );
    }
}

#[test]
fn bare_method_constructor_arguments_keep_kind_and_fixed_prefix() {
    for constructor in ["Maybe", "Either<Int>"] {
        let source = format!("type Higher<F<_>> = | Higher F<Int>\ntrait Inspect<A> {{ fn inspect left: A -> (A -> Bool) }}\ninstance<F<_>> Inspect<Higher<F>> {{ fn inspect left: Higher<F> -> (Higher<F> -> Bool) = \\right -> True }}\npub fn probe -> (Higher<{constructor}> -> Higher<{constructor}> -> Bool) = inspect");
        let core = compile_core_module(CompileInput::new("kind-λ.ssrg", "native/main", &source))
            .unwrap()
            .core_ir;
        emit_rust_module(&core, "probe").unwrap();
        let mut changed = core.clone();
        let CoreExpr::Call {
            evidence, type_ref, ..
        } = &mut changed.functions[0].body
        else {
            panic!()
        };
        let integer = CoreType::Named {
            name: "Int".into(),
            arguments: vec![],
        };
        let CoreInstanceEvidence::Local { type_arguments, .. } = &mut evidence[0].evidence else {
            panic!()
        };
        type_arguments[0] = integer.clone();
        let higher = match &mut evidence[0].constraint.arguments[0] {
            CoreType::Named { arguments, .. } | CoreType::ExternalNamed { arguments, .. } => {
                arguments[0] = integer;
                evidence[0].constraint.arguments[0].clone()
            }
            _ => panic!(),
        };
        *type_ref = CoreType::Function {
            parameter: Box::new(higher.clone()),
            result: Box::new(CoreType::Function {
                parameter: Box::new(higher),
                result: Box::new(CoreType::Named {
                    name: "Bool".into(),
                    arguments: vec![],
                }),
            }),
        };
        assert!(
            emit_rust_module(&changed, "probe").is_err(),
            "accepted a value in a constructor slot"
        );
    }
}
