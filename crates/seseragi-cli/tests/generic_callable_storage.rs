use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

struct Temporary(PathBuf);
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn diagnoses_generic_storage_before_build_or_execution() {
    let temporary = Temporary(
        std::env::temp_dir().join(format!("seseragi-generic-storage-{}", std::process::id())),
    );
    fs::create_dir_all(&temporary.0).unwrap();
    let source = temporary.0.join("main.ssrg");
    fs::write(&source, "fn identity<A> value: A -> A = value\nlet stored = [identity]\npub effect fn main = println \"unexpected\"\n").unwrap();
    for command in ["lint", "build", "run"] {
        let mut process = Command::new(env!("CARGO_BIN_EXE_seseragi"));
        process.arg(command).arg(&source);
        if command == "build" {
            process.arg("--out-dir").arg(temporary.0.join("output"));
        }
        let output = process.output().unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("SES-T0101"), "{error}");
        assert!(
            error.contains("Generic callable value type could not be inferred"),
            "{error}"
        );
        assert!(!temporary.0.join("output").exists());
    }
}

#[test]
fn executes_contextual_callable_storage() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/spec/fixtures/compile/generic-callable-storage.ssrg");
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&fixture)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        fs::read_to_string(fixture.with_extension("stdout")).unwrap()
    );
    assert!(output.stderr.is_empty());
}
