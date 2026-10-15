use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

static SERIAL: AtomicUsize = AtomicUsize::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target")
            .join(format!(
                "rust-cli-test-{}-{}",
                std::process::id(),
                SERIAL.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(&path).unwrap();
        Self(path.canonicalize().unwrap())
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_seseragi"))
}

#[test]
fn native_cli_and_ts_backend_agree_on_canonical_fixtures() {
    let scratch = Scratch::new();
    for name in [
        "ownership",
        "shared-values",
        "recursive-adt",
        "structural-record",
        "higher-kinded",
        "nested-dictionary",
        "tail-recursion",
        "tail-capture",
        "tail-evidence",
        "let-shadowing",
        "scalar-tail-loop",
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../../examples/spec/fixtures/rust-backend/{name}.ssrg"
        ));
        let destination = scratch.0.join(name);
        let built = cli()
            .args(["experimental", "build-native"])
            .arg(&path)
            .args(["--entry", "probe", "--out-dir"])
            .arg(&destination)
            .output()
            .unwrap();
        assert!(
            built.status.success(),
            "{}",
            String::from_utf8_lossy(&built.stderr)
        );
        let executable = destination.join(if cfg!(windows) {
            "program.exe"
        } else {
            "program"
        });
        let native = Command::new(&executable).env("PATH", "").output().unwrap();
        assert!(
            native.status.success(),
            "{}",
            String::from_utf8_lossy(&native.stderr)
        );
        assert!(
            native.stderr.is_empty(),
            "experimental metrics must not leak into ordinary output"
        );
        let metadata: serde_json::Value =
            serde_json::from_slice(&fs::read(destination.join("rust-backend.json")).unwrap())
                .unwrap();
        assert_eq!(metadata["backend"], "rust");
        assert_eq!(metadata["stability"], "experimental");
        assert_eq!(metadata["module"], "experimental/main");
        let source = format!(
            "{}\npub effect fn main = println (show (probe ()))\n",
            fs::read_to_string(&path).unwrap()
        );
        let oracle_path = scratch.0.join(format!("{name}-oracle.ssrg"));
        fs::write(&oracle_path, source).unwrap();
        for profile in ["development", "release"] {
            let ts = cli()
                .arg("run")
                .arg(&oracle_path)
                .args(["--profile", profile])
                .output()
                .unwrap();
            assert!(
                ts.status.success(),
                "{name}/{profile}: {}",
                String::from_utf8_lossy(&ts.stderr)
            );
            assert_eq!(native.stdout, ts.stdout, "{name}/{profile}");
        }
    }
}

#[test]
fn scalar_tail_loops_preserve_primitive_traces_on_bun_and_node() {
    let scratch = Scratch::new();
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/scalar-tail-loop.ssrg");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (name, profile) in [
        ("development", seseragi_driver::BuildProfile::Development),
        ("release", seseragi_driver::BuildProfile::Release),
    ] {
        let compiled = seseragi_driver::compile_module(
            seseragi_driver::CompileInput::new("main.ssrg", "scalar/main", source)
                .with_profile(profile),
        )
        .unwrap();
        let directory = scratch.0.join(name);
        fs::create_dir_all(&directory).unwrap();
        let emitted = directory.join("generated.ts");
        fs::write(&emitted, compiled.generated.typescript).unwrap();
        let output = Command::new("bun")
            .arg(root.join("scripts/scalar-tail-probe.ts"))
            .arg(&emitted)
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout)
                .matches("\"cases\":69")
                .count(),
            2
        );
    }
}

#[test]
fn emission_does_not_launch_rustc_and_invalid_targets_write_nothing() {
    let scratch = Scratch::new();
    let source_path = scratch.0.join("pure.ssrg");
    fs::write(&source_path, "pub fn answer unit: Unit -> Int = 42\n").unwrap();
    let output_path = scratch.0.join("generated.rs");
    let output = cli()
        .env("PATH", "")
        .args(["experimental", "emit-rust"])
        .arg(&source_path)
        .args(["--entry", "answer", "--output"])
        .arg(&output_path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_path.exists());
    let bad_path = scratch.0.join("bad.ssrg");
    fs::write(&bad_path, "pub fn answer -> Float = 1.5\n").unwrap();
    let bad_output = scratch.0.join("unsupported.rs");
    let bad = cli()
        .args(["experimental", "emit-rust"])
        .arg(&bad_path)
        .args(["--entry", "answer", "--output"])
        .arg(&bad_output)
        .output()
        .unwrap();
    assert_eq!(bad.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&bad.stderr).contains("SES-RUST-0001"));
    assert!(!bad_output.exists());
    let original = fs::read(&source_path).unwrap();
    let collision = cli()
        .args(["experimental", "emit-rust"])
        .arg(&source_path)
        .args(["--entry", "answer", "--output"])
        .arg(&source_path)
        .output()
        .unwrap();
    assert_eq!(collision.status.code(), Some(2));
    assert_eq!(fs::read(source_path).unwrap(), original);
}

#[test]
fn native_effect_cli_preserves_ts_results_scopes_and_typed_payloads() {
    let scratch = Scratch::new();
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/spec");
    for (name, relative, expected, exit) in [
        (
            "resource-lesson",
            "lessons/18-resource-scope.ssrg",
            "released: True\n",
            0,
        ),
        (
            "ref-cycle",
            "fixtures/rust-backend-research/ref-cycle.ssrg",
            "answer: 42\n",
            0,
        ),
        (
            "cold-effect",
            "fixtures/rust-backend-research/cold-effect.ssrg",
            "before: 0\nafter: 2\n",
            0,
        ),
        (
            "nested-scope",
            "fixtures/rust-backend-research/nested-scope.ssrg",
            "order: 3241\n",
            0,
        ),
        (
            "escaped-callback",
            "fixtures/rust-backend-research/escaped-callback.ssrg",
            "escaped: 42\n",
            0,
        ),
        (
            "failure-scope",
            "fixtures/rust-backend-research/failure-scope.ssrg",
            "failure released: 42\n",
            0,
        ),
        (
            "deep-effect",
            "fixtures/rust-backend-research/deep-effect.ssrg",
            "deep: 42\n",
            0,
        ),
        (
            "typed-payload",
            "fixtures/rust-backend/typed-effect-failure.ssrg",
            "typed payload: 42\n",
            0,
        ),
        (
            "ref-modify",
            "fixtures/rust-backend/ref-modify.ssrg",
            "modify: 41, 42\n",
            0,
        ),
        (
            "effect-evidence",
            "fixtures/rust-backend/effect-evidence.ssrg",
            "42\nbox 42\n",
            0,
        ),
        (
            "release-defect",
            "fixtures/rust-backend-research/release-callback-timing.ssrg",
            "body before release defect\n",
            70,
        ),
        (
            "console-values",
            "fixtures/rust-backend/console-values.ssrg",
            "true\n42\nhello\nundefined\nTrue\n",
            0,
        ),
        (
            "qualified-console",
            "fixtures/rust-backend/qualified-console.ssrg",
            "True\n",
            0,
        ),
        (
            "unhandled-failure",
            "fixtures/rust-backend/unhandled-effect-failure.ssrg",
            "",
            1,
        ),
    ] {
        let path = fixtures.join(relative);
        let destination = scratch.0.join(name);
        let built = cli()
            .args(["experimental", "build-native"])
            .arg(&path)
            .args(["--entry", "main", "--out-dir"])
            .arg(&destination)
            .output()
            .unwrap();
        assert!(
            built.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&built.stderr)
        );
        let program = destination.join(if cfg!(windows) {
            "program.exe"
        } else {
            "program"
        });
        let native = Command::new(&program)
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1")
            .output()
            .unwrap();
        assert_eq!(
            native.status.code(),
            Some(exit),
            "{name}: {}",
            String::from_utf8_lossy(&native.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&native.stdout), expected, "{name}");
        assert!(
            String::from_utf8_lossy(&native.stderr).contains("managed-live=0"),
            "{name}: {}",
            String::from_utf8_lossy(&native.stderr)
        );
        let ts = cli().arg("run").arg(&path).output().unwrap();
        assert_eq!(
            ts.status.code(),
            Some(exit),
            "{name}: {}",
            String::from_utf8_lossy(&ts.stderr)
        );
        assert_eq!(native.stdout, ts.stdout, "{name}");
        if exit != 70 {
            let native_error = String::from_utf8_lossy(&native.stderr)
                .lines()
                .filter(|line| !line.starts_with("native managed-live="))
                .collect::<Vec<_>>()
                .join("\n");
            assert_eq!(
                native_error,
                String::from_utf8_lossy(&ts.stderr).trim_end(),
                "{name}"
            );
        }
        if name == "cold-effect" {
            let ordinary = Command::new(&program)
                .env("PATH", "")
                .env_remove("SESERAGI_NATIVE_METRICS")
                .output()
                .unwrap();
            assert!(ordinary.status.success());
            assert!(ordinary.stderr.is_empty());
            assert_eq!(ordinary.stdout, ts.stdout);
        }
        let metadata: serde_json::Value =
            serde_json::from_slice(&fs::read(destination.join("rust-backend.json")).unwrap())
                .unwrap();
        assert_eq!(metadata["abi"], "rust-erased-sync-v2");
        assert_eq!(metadata["entryKind"], "effect");
    }
}

#[test]
fn unsupported_async_operation_has_a_source_range_and_writes_nothing() {
    let scratch = Scratch::new();
    let path = scratch.0.join("async.ssrg");
    fs::write(
        &path,
        "import * as effects from \"std/effect\"\npub effect fn main = effects.yieldNow ()\n",
    )
    .unwrap();
    let output = scratch.0.join("unsupported.rs");
    let result = cli()
        .env("PATH", "")
        .args(["experimental", "emit-rust"])
        .arg(&path)
        .args(["--entry", "main", "--output"])
        .arg(&output)
        .output()
        .unwrap();
    let error = String::from_utf8_lossy(&result.stderr);
    assert_eq!(result.status.code(), Some(2));
    assert!(
        error.contains("SES-RUST-0001") && error.contains("yieldNow"),
        "{error}"
    );
    assert!(error.contains("async.ssrg:"), "{error}");
    assert!(!output.exists());
    assert!(!output.with_extension("rust-backend.json").exists());
}
