use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};

fn execute(source: &str, name: &str, expected: &str, exit: i32) {
    use std::{fs, process::Command};
    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let core = compile_core_module(CompileInput::new("ord-λ.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    let artifact = emit_rust_module(&core, "main").unwrap();
    let dir = Scratch(
        std::env::temp_dir().join(format!("seseragi-ord-method-{}-{name}", std::process::id())),
    );
    fs::create_dir(&dir.0).unwrap();
    let file = dir.0.join("program.rs");
    fs::write(&file, artifact.source).unwrap();
    let exe = dir.0.join(if cfg!(windows) {
        "program.exe"
    } else {
        "program"
    });
    let built = Command::new("rustc")
        .args(["--edition=2021", "-O", "-o"])
        .arg(&exe)
        .arg(&file)
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let result = Command::new(exe)
        .env("PATH", "")
        .env("SESERAGI_NATIVE_METRICS", "1")
        .output()
        .unwrap();
    assert_eq!(
        result.status.code(),
        Some(exit),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&result.stdout), expected);
    assert!(String::from_utf8_lossy(&result.stderr).contains("managed-live=0"));
}

#[test]
fn scoped_ord_method_runs_its_first_source_stage_on_partial_creation() {
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/ord-method-defect.ssrg");
    execute(source, "annotated", "", 70);
    execute(
        &source.replace(
            "let chosen: A -> A -> Ordering = compare",
            "let chosen = compare",
        ),
        "generalized",
        "",
        70,
    );
    execute(
        &source
            .replace(
                "fn compare left: Mark -> (Mark -> Ordering) = {",
                "fn compare left: Mark -> right: Mark -> Ordering = {",
            )
            .replace("\\right -> Equal", "Equal"),
        "two-source-parameters",
        "created\n",
        0,
    );
}

#[test]
fn ord_factories_preserve_ordering_values_saved_aliases_and_returned_functions() {
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/ord-method-values.ssrg"),
        "values",
        include_str!("../../../examples/spec/fixtures/rust-backend/ord-method-values.stdout"),
        0,
    );
}

#[test]
fn ord_method_closures_retain_cold_effects_refs_cycles_and_finalizers() {
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/ord-method-effects.ssrg"),
        "effects",
        include_str!("../../../examples/spec/fixtures/rust-backend/ord-method-effects.stdout"),
        0,
    );
}

#[test]
fn bare_ord_refuses_forged_owners_signatures_kinds_and_evidence_frames() {
    use seseragi_lowering::{CoreExpr, CoreInstanceEvidence, CoreType};
    let core = compile_core_module(CompileInput::new(
        "ord-λ.ssrg", "native/main",
        "// 雫🙂\nfn selected<A> -> (A -> A -> Ordering) where Ord<A> = compare\npub fn probe -> Bool = True",
    )).unwrap().core_ir;
    emit_rust_module(&core, "probe").unwrap();
    for mutation in 0..14 {
        let mut changed = core.clone();
        let CoreExpr::Call {
            evidence,
            trait_dispatch: Some(dispatch),
            type_ref,
            ..
        } = &mut changed.functions[0].body
        else {
            panic!("named compare")
        };
        match mutation {
            0 => dispatch.trait_identity = "fake::Ord".into(),
            1 => dispatch.method = "eq".into(),
            2 => evidence[0].constraint.trait_identity = Some("fake::Ord".into()),
            3 => evidence[0].constraint.name = "Eq".into(),
            4 => evidence[0].constraint.arguments.clear(),
            5 => evidence[0].constraint.arguments[0] = CoreType::Hole,
            6 => evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 1 },
            7 => *type_ref = CoreType::Hole,
            8 => {
                let CoreType::Function { result, .. } = type_ref else {
                    panic!()
                };
                let CoreType::Function { result, .. } = result.as_mut() else {
                    panic!()
                };
                **result = CoreType::Named {
                    name: "Bool".into(),
                    arguments: vec![],
                };
            }
            9 => {
                let CoreType::Function { result, .. } = type_ref else {
                    panic!()
                };
                let CoreType::Function { result, .. } = result.as_mut() else {
                    panic!()
                };
                **result = CoreType::ExternalNamed {
                    name: "Ordering".into(),
                    canonical: "fake::Ordering".into(),
                    arguments: vec![],
                };
            }
            10 => {
                let CoreType::Function { parameter, .. } = type_ref else {
                    panic!()
                };
                **parameter = CoreType::Named {
                    name: "Int".into(),
                    arguments: vec![],
                };
            }
            11 => evidence[0].constraint.arguments.push(CoreType::Hole),
            12 => changed.functions[0].type_parameters[0].arity = 1,
            _ => changed.functions[0].constraints[0].trait_identity = Some("fake::Ord".into()),
        }
        let error = match emit_rust_module(&changed, "probe") {
            Ok(_) => panic!("counterfeit case {mutation} was accepted"),
            Err(error) => error,
        };
        assert_eq!(error.code, "SES-RUST-0001", "case {mutation}");
        assert_eq!(
            error.origin.unwrap().source,
            "ord-λ.ssrg",
            "case {mutation}"
        );
    }
}

#[test]
fn named_int_ord_retains_canonical_type_and_dictionary_identity() {
    use seseragi_lowering::{CoreExpr, CoreInstanceEvidence, CoreStatement, CoreType};
    let core = compile_core_module(CompileInput::new(
        "ord-standard-λ.ssrg", "native/main",
        "// 雫🙂\npub fn probe -> Ordering = { let chosen: Int -> Int -> Ordering = compare; chosen 1 2 }",
    )).unwrap().core_ir;
    emit_rust_module(&core, "probe").unwrap();
    for mutation in 0..8 {
        let mut changed = core.clone();
        let CoreExpr::Sequence { statements, .. } = &mut changed.functions[0].body else {
            panic!()
        };
        let CoreStatement::PureLet {
            value: CoreExpr::Call {
                evidence, type_ref, ..
            },
            ..
        } = &mut statements[0]
        else {
            panic!()
        };
        let extra_child = evidence[0].clone();
        let CoreInstanceEvidence::Standard {
            identity,
            type_arguments,
            evidence_arguments,
        } = &mut evidence[0].evidence
        else {
            panic!()
        };
        match mutation {
            0 => *identity = "fake::Ord".into(),
            1 => *identity = "std/char::Ord".into(),
            2 => type_arguments.push(CoreType::Hole),
            3 => evidence_arguments.push(extra_child),
            4 => {
                evidence[0].constraint.arguments[0] = CoreType::ExternalNamed {
                    name: "Int".into(),
                    canonical: "fake::Int".into(),
                    arguments: vec![],
                }
            }
            5 => {
                *type_ref = CoreType::Named {
                    name: "Ordering".into(),
                    arguments: vec![],
                }
            }
            6 => {
                let CoreType::Function { result, .. } = type_ref else {
                    panic!()
                };
                let CoreType::Function { result, .. } = result.as_mut() else {
                    panic!()
                };
                **result = CoreType::Named {
                    name: "Bool".into(),
                    arguments: vec![],
                };
            }
            _ => evidence.clear(),
        }
        let error = match emit_rust_module(&changed, "probe") {
            Ok(_) => panic!("counterfeit standard case {mutation} was accepted"),
            Err(error) => error,
        };
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap().source, "ord-standard-λ.ssrg");
    }
}
