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
        let ts = cli().arg("run").arg(&oracle_path).output().unwrap();
        assert!(
            ts.status.success(),
            "{}",
            String::from_utf8_lossy(&ts.stderr)
        );
        assert_eq!(native.stdout, ts.stdout, "{name}");
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
