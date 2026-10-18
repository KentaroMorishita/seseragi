use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use std::{fs, process::Command};

#[test]
fn partial_captures_keep_cold_effects_ref_cycles_and_finalizers() {
    for (index, source, expected, exit) in [
        (
            0,
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/strict-partial-effects.ssrg"
            ),
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/strict-partial-effects.stdout"
            ),
            0,
        ),
        (
            1,
            include_str!("../../../examples/spec/fixtures/rust-backend/strict-partial-defect.ssrg"),
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/strict-partial-defect.stdout"
            ),
            70,
        ),
    ] {
        let core =
            compile_core_module(CompileInput::new("effect-λ.ssrg", "native/main", source)).unwrap();
        let rust = emit_rust_module(&core.core_ir, "main").unwrap();
        let path = std::env::temp_dir().join(format!(
            "seseragi-strict-effect-{}-{index}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        fs::write(path.join("program.rs"), rust.source).unwrap();
        let binary = path.join(if cfg!(windows) {
            "program.exe"
        } else {
            "program"
        });
        let built = Command::new("rustc")
            .args(["--edition=2021", "-O", "-o"])
            .arg(&binary)
            .arg(path.join("program.rs"))
            .output()
            .unwrap();
        assert!(
            built.status.success(),
            "{}",
            String::from_utf8_lossy(&built.stderr)
        );
        let output = Command::new(&binary)
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1")
            .output()
            .unwrap();
        fs::remove_dir_all(&path).unwrap();
        assert_eq!(
            output.status.code(),
            Some(exit),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
        assert!(String::from_utf8_lossy(&output.stderr).contains("managed-live=0"));
    }
}

#[test]
fn partials_reject_counterfeit_remaining_source_and_evidence_frames() {
    use seseragi_lowering::{CoreExpr, CoreStatement, CoreType};
    let source = "// 雫🙂\npub fn probe -> String = { fn prefix<A> value: A -> suffix: String -> String where Show<A> = show value + suffix; let saved = prefix [41][0]; saved \"!\" }";
    let core = compile_core_module(CompileInput::new("partial-λ.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    assert!(emit_rust_module(&core, "probe").is_ok());
    for case in 0..7 {
        let mut changed = core.clone();
        let CoreExpr::Sequence { statements, .. } = &mut changed.functions[0].body else {
            panic!("sequence")
        };
        let CoreStatement::PureLet {
            value:
                CoreExpr::Call {
                    evidence,
                    deferred_evidence_parameters,
                    deferred_evidence_type_constructor_parameters,
                    evidence_argument_index,
                    ..
                },
            ..
        } = &mut statements[1]
        else {
            panic!("partial")
        };
        match case {
            0 => deferred_evidence_parameters.clear(),
            1 => deferred_evidence_parameters.push(CoreType::Hole),
            2 => deferred_evidence_parameters[0] = CoreType::Hole,
            3 => evidence[0].constraint.trait_identity = Some("counterfeit::Show".into()),
            4 => *evidence_argument_index = Some(0),
            5 => deferred_evidence_type_constructor_parameters.push("F".into()),
            _ => evidence.clear(),
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001", "case {case}");
        assert_eq!(error.origin.unwrap().source, "partial-λ.ssrg");
    }
}

#[test]
fn supplied_arguments_run_once_before_partial_reuse() {
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/strict-partial-trace.ssrg");
    let core = compile_core_module(CompileInput::new("trace.ssrg", "native/main", source)).unwrap();
    let rust = emit_rust_module(&core.core_ir, "probe").unwrap();
    let main = rust.source.rfind("fn main()").unwrap();
    let mut generated = rust.source[..main].to_owned();
    let id = |name: &str| {
        format!(
            "v_{}",
            name.bytes().map(|b| format!("{b:02x}")).collect::<String>()
        )
    };
    // Instrument the ordinary source helper bodies in this test copy only.
    // Observing the marker separates partial creation from both later uses.
    for name in [
        "globalValue",
        "localValue",
        "tag",
        "methodValue",
        "beforeApply",
    ] {
        let start = generated
            .find(&format!(
                "fn {}_direct(",
                id(&format!("native/main::{name}"))
            ))
            .unwrap();
        let body = start + generated[start..].find("-> V {").unwrap() + "-> V {".len();
        generated.insert_str(
            body,
            &format!(" TRACE.with(|trace| trace.borrow_mut().push({name:?})); "),
        );
    }
    generated.push_str(&format!(r#"
thread_local! {{ static TRACE: RefCell<Vec<&'static str>> = const {{ RefCell::new(Vec::new()) }}; }}
fn main() {{
 let value = {}(vec![]).apply(V::Unit);
 println!("{{}}", value.render());
 TRACE.with(|trace| assert_eq!(&*trace.borrow(), &["globalValue", "localValue", "tag", "methodValue", "beforeApply"]));
 drop(value); clear_dictionaries(); assert_eq!(LIVE.with(Cell::get), 0);
}}
"#, id(&rust.metadata.entry)));
    let path = std::env::temp_dir().join(format!("seseragi-strict-trace-{}", std::process::id()));
    fs::create_dir(&path).unwrap();
    fs::write(path.join("program.rs"), generated).unwrap();
    let binary = path.join(if cfg!(windows) {
        "program.exe"
    } else {
        "program"
    });
    let built = Command::new("rustc")
        .args(["--edition=2021", "-O", "-o"])
        .arg(&binary)
        .arg(path.join("program.rs"))
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let output = Command::new(&binary).env("PATH", "").output().unwrap();
    fs::remove_dir_all(&path).unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        include_str!("../../../examples/spec/fixtures/rust-backend/strict-partial-trace.stdout")
    );
}

#[test]
fn saved_partials_keep_polymorphic_locals_hkt_children_and_returned_source_arity() {
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/strict-partials.ssrg");
    let core =
        compile_core_module(CompileInput::new("strict.ssrg", "native/main", source)).unwrap();
    let rust = emit_rust_module(&core.core_ir, "probe").unwrap();
    let main = rust.source.rfind("fn main()").unwrap();
    let entry = format!(
        "v_{}",
        rust.metadata
            .entry
            .bytes()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    let generated = format!("{}\nfn main() {{ for n in 0..1000 {{ let value = {entry}(vec![]).apply(V::Unit); if n == 0 {{ println!(\"{{}}\", value.render()); }} drop(value); }} clear_dictionaries(); assert_eq!(LIVE.with(Cell::get), 0); }}", &rust.source[..main]);
    let path = std::env::temp_dir().join(format!("seseragi-strict-saved-{}", std::process::id()));
    fs::create_dir(&path).unwrap();
    fs::write(path.join("program.rs"), generated).unwrap();
    let binary = path.join(if cfg!(windows) {
        "program.exe"
    } else {
        "program"
    });
    let built = Command::new("rustc")
        .args(["--edition=2021", "-O", "-o"])
        .arg(&binary)
        .arg(path.join("program.rs"))
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let output = Command::new(&binary).env("PATH", "").output().unwrap();
    fs::remove_dir_all(&path).unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        include_str!("../../../examples/spec/fixtures/rust-backend/strict-partials.stdout")
    );
}

#[test]
fn unused_global_local_and_method_partials_evaluate_supplied_arguments() {
    for (index, source) in [
        "fn prefix<A> value: A -> suffix: String -> String where Show<A> = show value + suffix\npub fn probe -> Unit = { let partial = prefix [9007199254740991 + 1][0]; () }",
        "pub fn probe -> Unit = { fn prefix<A> value: A -> suffix: String -> String where Show<A> = show value + suffix; let partial = prefix [9007199254740991 + 1]; () }",
        "type Token = | Token\ntrait Render<T> { fn render<A> tag: T -> value: A -> suffix: String -> String where Show<A> }\ninstance Render<Token> { fn render<A> tag: Token -> value: A -> suffix: String -> String where Show<A> = show value + suffix }\npub fn probe -> Unit = { let partial = render Token [9007199254740991 + 1]; () }",
    ].iter().enumerate() {
        let core = compile_core_module(CompileInput::new("strict-λ.ssrg", "native/main", source)).unwrap();
        let rust = emit_rust_module(&core.core_ir, "probe").unwrap();
        let path = std::env::temp_dir().join(format!("seseragi-strict-partial-{}-{index}", std::process::id()));
        fs::create_dir(&path).unwrap();
        fs::write(path.join("program.rs"), rust.source).unwrap();
        let binary = path.join(if cfg!(windows) { "program.exe" } else { "program" });
        let built = Command::new("rustc").args(["--edition=2021", "-O", "-o"]).arg(&binary).arg(path.join("program.rs")).output().unwrap();
        assert!(built.status.success(), "{}", String::from_utf8_lossy(&built.stderr));
        let output = Command::new(&binary).env("PATH", "").env("SESERAGI_NATIVE_METRICS", "1").output().unwrap();
        fs::remove_dir_all(&path).unwrap();
        assert_eq!(output.status.code(), Some(70), "partial {index} skipped its supplied argument");
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Int overflow"), "{stderr}");
        assert!(stderr.contains("managed-live=0"), "{stderr}");
    }
}
