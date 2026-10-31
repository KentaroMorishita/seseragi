use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_module, CompileInput};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "seseragi-stream-validation-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self(directory)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn compare(source: &str, expected: &str) {
    let compiled = compile_module(CompileInput::new(
        "stream-validation.ssrg",
        "native/stream-validation",
        source,
    ))
    .unwrap_or_else(|error| panic!("{error:?}"));
    let rust = emit_rust_module(&compiled.core_ir, "main").unwrap().source;
    let mut typescript = compiled.generated.typescript;
    let scratch = Scratch::new();
    let program = scratch.0.join("program.rs");
    let binary = scratch
        .0
        .join(format!("program{}", std::env::consts::EXE_SUFFIX));
    fs::write(&program, rust).unwrap();
    let built = Command::new("rustc")
        .args(["--edition=2021", "-O"])
        .arg(&program)
        .arg("-o")
        .arg(&binary)
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let native = Command::new(binary)
        .env("PATH", "")
        .env("SESERAGI_NATIVE_METRICS", "1")
        .output()
        .unwrap();
    assert!(
        native.status.success(),
        "{}",
        String::from_utf8_lossy(&native.stderr)
    );
    assert_eq!(native.stdout, expected.as_bytes());
    assert!(String::from_utf8_lossy(&native.stderr).contains("managed-live=0"));

    let runtime = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../runtime/ts/src")
        .canonicalize()
        .unwrap();
    typescript.push_str(
        "\nimport { run as runFixture } from '@seseragi/runtime/effect';\nimport { liveConsole as fixtureConsole } from '@seseragi/runtime/console';\nconst result = await runFixture(main(undefined), { console: fixtureConsole });\nif (result.kind !== 'success') throw new Error(JSON.stringify(result));\n",
    );
    let typescript = typescript
        .replace("\"@seseragi/runtime/", &format!("\"{}/", runtime.display()))
        .replace("'@seseragi/runtime/", &format!("'{}/", runtime.display()));
    let program = scratch.0.join("program.ts");
    fs::write(&program, typescript).unwrap();
    let ts = Command::new("bun").arg(program).output().unwrap();
    assert!(
        ts.status.success(),
        "{}",
        String::from_utf8_lossy(&ts.stderr)
    );
    assert_eq!(ts.stdout, expected.as_bytes());
    assert_eq!(native.stdout, ts.stdout);
    assert!(
        ts.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&ts.stderr)
    );
}

#[test]
fn stream_operations_match_typescript() {
    compare(
        include_str!("../../../examples/spec/fixtures/rust-backend/stream-operations.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/stream-operations.stdout"),
    );
}
#[test]
fn stream_effects_are_cold_repeated_scoped_and_ordered() {
    compare(
        include_str!("../../../examples/spec/fixtures/rust-backend/stream-effects.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/stream-effects.stdout"),
    );
}
#[test]
fn validation_constructors_patterns_and_instances_match_typescript() {
    compare(
        include_str!("../../../examples/spec/fixtures/rust-backend/validation-operations.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/validation-operations.stdout"),
    );
}
#[test]
fn large_stream_fold_matches_typescript() {
    compare(
        include_str!("../../../examples/spec/fixtures/rust-backend/stream-large.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/stream-large.stdout"),
    );
}
#[test]
fn official_typeclass_operator_sample_matches_typescript() {
    compare(
        include_str!("../../../examples/samples/typeclass-operator-parity/main.ssrg"),
        include_str!("../../../examples/samples/typeclass-operator-parity/stdout.txt"),
    );
}

#[test]
fn public_operations_reject_unproven_owners_kinds_and_staging() {
    use seseragi_driver::compile_core_module;
    use seseragi_lowering::{CoreExpr, CoreType};
    for source in [
        "import * as s from \"std/stream\"\npub fn probe -> s.Stream<{},Never,Int> = s.fromArray [1]\npub fn main -> Int = 0",
        "import * as v from \"std/validation\"\npub fn probe -> v.Validation<String,Int> = v.Valid 1\npub fn main -> Int = 0",
        "import * as v from \"std/validation\"\npub fn probe -> v.Validation<String,Int> = v.invalid \"x\"\npub fn main -> Int = 0",
    ] {
        let core = compile_core_module(CompileInput::new("proof.ssrg", "native/proof", source)).unwrap().core_ir;
        emit_rust_module(&core, "main").unwrap();
        for mutation in 0..5 {
            let mut bad = core.clone();
            let body = &mut bad.functions.iter_mut().find(|f| f.symbol.ends_with("::probe")).unwrap().body;
            let CoreExpr::Call { arguments, type_ref, evidence_argument_index, deferred_evidence_parameters, .. } = body else { panic!("call expected") };
            match mutation {
                0 => *type_ref = CoreType::Hole,
                1 => *type_ref = CoreType::ExternalNamed { name: "Stream".into(), canonical: "fake::Stream".into(), arguments: vec![] },
                2 => *evidence_argument_index = Some(0),
                3 => deferred_evidence_parameters.push(CoreType::Hole),
                _ => arguments.push(arguments[0].clone()),
            }
            let error = emit_rust_module(&bad, "main").unwrap_err();
            assert_eq!(error.code, "SES-RUST-0001");
            assert_eq!(error.origin.unwrap().source, "proof.ssrg");
        }
    }
}

#[test]
fn validation_equality_rejects_counterfeit_factories_and_child_dictionaries() {
    use seseragi_driver::compile_core_module;
    use seseragi_lowering::{CoreExpr, CoreInstanceEvidence, CoreType};
    let source = "import { Validation } from \"std/validation\"\nfn same value:Validation<String,Int> -> Bool = value == value\npub fn main -> Int = 0";
    let core = compile_core_module(CompileInput::new("eq.ssrg", "native/validation-eq", source))
        .unwrap()
        .core_ir;
    emit_rust_module(&core, "main").unwrap();
    for mutation in 0..7 {
        let mut bad = core.clone();
        let CoreExpr::Binary { evidence, .. } = &mut bad
            .functions
            .iter_mut()
            .find(|f| f.symbol.ends_with("::same"))
            .unwrap()
            .body
        else {
            panic!("binary expected")
        };
        let selected = &mut evidence[0];
        let CoreInstanceEvidence::Standard {
            identity,
            type_arguments,
            evidence_arguments,
        } = &mut selected.evidence
        else {
            panic!("standard expected")
        };
        match mutation {
            0 => *identity = "std/either::Eq".into(),
            1 => type_arguments.swap(0, 1),
            2 => {
                evidence_arguments.pop();
            }
            3 => evidence_arguments[0].constraint.arguments[0] = CoreType::Hole,
            4 => {
                let CoreInstanceEvidence::Standard { identity, .. } =
                    &mut evidence_arguments[0].evidence
                else {
                    panic!("standard child expected")
                };
                *identity = "std/int::Eq".into();
            }
            5 => selected.constraint.trait_identity = Some("fake::Eq".into()),
            _ => evidence_arguments[0].constraint.trait_identity = Some("fake::Eq".into()),
        }
        let error = emit_rust_module(&bad, "main").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap().source, "eq.ssrg");
    }
}
