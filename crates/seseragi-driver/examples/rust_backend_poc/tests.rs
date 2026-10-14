use super::{backend, frontend};
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
            "seseragi-rust-poc-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn compile(&self, source: &str, test: bool) -> PathBuf {
        let input = self.0.join("generated.rs");
        let binary = self.0.join("native");
        fs::write(&input, source).unwrap();
        let mut command = Command::new("rustc");
        command
            .args(["--edition=2021", "-O", "-A", "dead_code", "-o"])
            .arg(&binary)
            .arg(&input);
        if test {
            command.arg("--test");
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        binary
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

const CASES: &[(&str, &str, &str)] = &[
    (
        "ownership",
        include_str!("../../../../examples/spec/fixtures/rust-backend-poc/ownership.ssrg"),
        "(42, 42, 11, 12)",
    ),
    (
        "shared-values",
        include_str!("../../../../examples/spec/fixtures/rust-backend-poc/shared-values.ssrg"),
        "(shared, shared, shared)",
    ),
    (
        "recursive-adt",
        include_str!("../../../../examples/spec/fixtures/rust-backend-poc/recursive-adt.ssrg"),
        "(3, 5)",
    ),
    (
        "structural-record",
        include_str!("../../../../examples/spec/fixtures/rust-backend-poc/structural-record.ssrg"),
        "(row, row)",
    ),
    (
        "higher-kinded",
        include_str!("../../../../examples/spec/fixtures/rust-backend-poc/higher-kinded.ssrg"),
        "(42, 42)",
    ),
    (
        "nested-dictionary",
        include_str!("../../../../examples/spec/fixtures/rust-backend-poc/nested-dictionary.ssrg"),
        "(42, 42)",
    ),
];

#[test]
fn checked_core_generates_native_programs_without_bun_or_typescript() {
    for (name, source, expected) in CASES {
        let core = frontend(&format!("{name}.ssrg"), source).unwrap();
        let generated = backend::emit(&core, "probe", 10_000).unwrap();
        let scratch = Scratch::new();
        let binary = scratch.compile(&generated, false);
        let output = Command::new(binary).env("PATH", "").output().unwrap();
        assert!(
            output.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            *expected,
            "{name}"
        );
        let metrics = String::from_utf8_lossy(&output.stderr);
        assert!(
            metrics.contains("iterations=10000 live=0"),
            "{name}: {metrics}"
        );
        eprintln!("{name}: {metrics}");
    }
}

#[test]
fn memory_and_cold_factory_probes() {
    let scratch = Scratch::new();
    let binary = scratch.compile(include_str!("runtime.rs"), true);
    let output = Command::new(binary).env("PATH", "").output().unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("4 passed"));
    eprintln!("{}", String::from_utf8_lossy(&output.stdout));
}

#[test]
fn unsupported_source_features_fail_before_rustc() {
    let cases = [
        (
            "pub effect fn probe = println \"hello\"\n",
            "Effect source lowering",
        ),
        (
            "pub fn probe -> List<Int> = `[1, 2]\n",
            "unsupported CoreIr expression",
        ),
        (
            "pub fn probe -> Float = 1.5\n",
            "unsupported CoreIr expression",
        ),
    ];
    for (source, expected) in cases {
        let core = frontend("unsupported.ssrg", source).unwrap();
        assert!(backend::emit(&core, "probe", 1)
            .unwrap_err()
            .contains(expected));
    }
    assert!(frontend("bad.ssrg", "pub fn probe -> Int = \"wrong\"\n").is_err());
}
