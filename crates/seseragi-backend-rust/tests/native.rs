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
        "pub fn probe -> Float = 1.5\n",
        "pub fn probe -> List<Int> = `[1, 2]\n",
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
