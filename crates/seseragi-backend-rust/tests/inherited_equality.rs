use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreExpr, CoreInstanceEvidence, CoreType};

#[test]
fn ord_parameter_provides_eq_for_direct_saved_and_forwarded_calls() {
    for (index, source) in [
        "fn same<A> value: A -> Bool where Ord<A> = value == value\npub fn probe -> Bool = same 42",
        "fn same<A> value: A -> Bool where Ord<A> = { let selected: A -> A -> Bool = (==); let held = selected value; held value }\npub fn probe -> Bool = same 42",
        "fn equal<A> left: A -> right: A -> Bool where Eq<A> = left == right\nfn same<A> value: A -> Bool where Ord<A> = equal value value\npub fn probe -> Bool = same 42",
    ]
    .into_iter()
    .enumerate()
    {
        let core = compile_core_module(CompileInput::new(
            "inherited-eq-雫.ssrg",
            "native/main",
            source,
        ))
        .unwrap()
        .core_ir;
        if index == 0 {
            let CoreExpr::Binary { evidence, .. } = &core.functions[0].body else {
                panic!("direct equality")
            };
            assert_eq!(
                evidence[0].constraint.trait_identity.as_deref(),
                Some("std/prelude::Eq")
            );
            assert!(matches!(
                evidence[0].evidence,
                CoreInstanceEvidence::Parameter { index: 0 }
            ));
            assert_eq!(
                core.functions[0].constraints[0].trait_identity.as_deref(),
                Some("std/prelude::Ord")
            );
        }
        if let Ok(directory) = std::env::var("SESERAGI_INHERITED_EQ_CAPTURE") {
            let directory = std::path::Path::new(&directory);
            std::fs::create_dir_all(directory).unwrap();
            std::fs::write(
                directory.join(format!("core-{index}.json")),
                serde_json::to_vec_pretty(&core).unwrap(),
            )
            .unwrap();
        }
        emit_rust_module(&core, "probe").unwrap();
    }
}

#[test]
fn inherited_eq_refuses_unknown_owners_kinds_arguments_and_dictionary_frames() {
    let source = "// 雫🙂\nfn same<A> value: A -> Bool where Ord<A> = value == value\npub fn probe -> Bool = same 42";
    let core = compile_core_module(CompileInput::new(
        "inherited-eq-雫.ssrg",
        "native/main",
        source,
    ))
    .unwrap()
    .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    for mutation in 0..14 {
        let mut changed = core.clone();
        let CoreExpr::Binary { evidence, .. } = &mut changed.functions[0].body else {
            panic!("equality body")
        };
        match mutation {
            0 => evidence[0].constraint.trait_identity = Some("fake::Eq".into()),
            1 => evidence[0].constraint.trait_identity = None,
            2 => evidence[0].constraint.name = "Ord".into(),
            3 => evidence[0].constraint.arguments.clear(),
            4 => evidence[0].constraint.arguments[0] = CoreType::Hole,
            5 => evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 1 },
            6 => evidence[0].constraint.arguments.push(CoreType::Hole),
            7 => {
                evidence[0].constraint.arguments[0] = CoreType::ExternalNamed {
                    name: "A".into(),
                    canonical: "fake::A".into(),
                    arguments: vec![],
                }
            }
            8 => changed.functions[0].constraints[0].trait_identity = Some("fake::Ord".into()),
            9 => changed.functions[0].constraints[0].trait_identity = None,
            10 => changed.functions[0].constraints[0].name = "Eq".into(),
            11 => changed.functions[0].constraints[0].arguments[0] = CoreType::Hole,
            12 => changed.functions[0].type_parameters[0].arity = 1,
            _ => {
                changed.functions[0].constraints[0].arguments[0] = CoreType::ExternalNamed {
                    name: "A".into(),
                    canonical: "other::A".into(),
                    arguments: vec![],
                }
            }
        }
        let error = emit_rust_module(&changed, "probe")
            .err()
            .unwrap_or_else(|| panic!("counterfeit inherited Eq case {mutation} was accepted"));
        assert_eq!(error.code, "SES-RUST-0001", "case {mutation}");
        let origin = error.origin.unwrap();
        assert_eq!(origin.source, "inherited-eq-雫.ssrg", "case {mutation}");
        assert!(origin.start <= source.find("value == value").unwrap());
        assert!(origin.end >= source.find("value == value").unwrap() + "value == value".len());
    }
}

#[test]
fn canonical_ord_providers_must_supply_their_checked_eq_parent() {
    let source = "type Mark = | Mark Int\ninstance Eq<Mark> { fn eq left: Mark -> right: Mark -> Bool = True }\ninstance Ord<Mark> { fn compare left: Mark -> right: Mark -> Ordering = Equal }\nfn same<A> value: A -> Bool where Ord<A> = value == value\npub fn probe -> Bool = same (Mark 42)";
    let core = compile_core_module(CompileInput::new(
        "inherited-eq-雫.ssrg",
        "native/main",
        source,
    ))
    .unwrap()
    .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    for mutation in 0..5 {
        let mut changed = core.clone();
        let provider = changed
            .instances
            .iter_mut()
            .find(|instance| instance.identity.starts_with("std/prelude::Ord<"))
            .unwrap();
        match mutation {
            0 => {
                provider.supertrait_count = 0;
                provider.constraints.clear();
            }
            1 => provider.supertrait_count = 2,
            2 => provider.constraints[0].trait_identity = Some("fake::Eq".into()),
            3 => provider.constraints[0].arguments[0] = CoreType::Hole,
            _ => provider.constraints[0].arguments.clear(),
        }
        let error = emit_rust_module(&changed, "probe")
            .err()
            .unwrap_or_else(|| panic!("counterfeit Ord provider case {mutation} was accepted"));
        assert_eq!(error.code, "SES-RUST-0001", "case {mutation}");
        assert_eq!(error.origin.unwrap().source, "inherited-eq-雫.ssrg");
    }
}

fn execute(source: &str, name: &str, expected: &str, exit: i32) {
    use std::{fs, process::Command};
    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let core = compile_core_module(CompileInput::new(
        "inherited-eq-雫.ssrg",
        "native/main",
        source,
    ))
    .unwrap()
    .core_ir;
    let artifact = emit_rust_module(&core, "main").unwrap();
    let directory = Scratch(std::env::temp_dir().join(format!(
        "seseragi-inherited-eq-{}-{name}",
        std::process::id(),
    )));
    fs::create_dir(&directory.0).unwrap();
    let file = directory.0.join("program.rs");
    fs::write(&file, artifact.source).unwrap();
    let exe = directory.0.join(if cfg!(windows) {
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
    let result = Command::new(&exe)
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
fn inherited_eq_executes_the_selected_parent_for_standard_and_user_ord() {
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/ord-inherited-eq.ssrg"),
        "values",
        include_str!("../../../examples/spec/fixtures/rust-backend/ord-inherited-eq.stdout"),
        0,
    );
}

#[test]
fn inherited_eq_keeps_cold_refs_cycles_and_finalizer_lifetimes() {
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/ord-inherited-eq-effects.ssrg"),
        "effects",
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/ord-inherited-eq-effects.stdout"
        ),
        0,
    );
}

#[test]
fn inherited_eq_runs_its_first_source_stage_before_the_next_statement() {
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/ord-inherited-eq-defect.ssrg");
    execute(source, "defect", "acquired\nreleased\n", 70);
    execute(
        &source
            .replace(
                "fn eq left: Mark -> (Mark -> Bool) = {",
                "fn eq left: Mark -> right: Mark -> Bool = {",
            )
            .replace("\\right -> True", "True"),
        "two-source-parameters",
        "acquired\nunreachable\nreleased\n",
        0,
    );
}
