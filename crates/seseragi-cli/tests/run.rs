use std::io::Write;
use std::process::{Command, Stdio};

fn repository_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

#[test]
fn runs_the_phase_one_program_without_fixture_metadata() {
    let root = repository_root();
    let program = root.join("examples/spec/artifacts/schema-1/rock-paper-scissors-cli/main.ssrg");
    let mut child = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(program)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"rock\nscissors\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "Player 1 wins!\n");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn renders_typed_failure_and_preserves_the_program_exit_class() {
    let root = repository_root();
    let program = root.join("examples/spec/artifacts/schema-1/rock-paper-scissors-cli/main.ssrg");
    let mut child = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(program)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"lizard\n").unwrap();
    let output = child.wait_with_output().unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "UnknownHand lizard\n"
    );
}

#[test]
fn reports_compiler_diagnostics_with_source_ranges() {
    let program =
        repository_root().join("examples/spec/fixtures/diagnostics/invalid-numeric-literal.ssrg");
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&program)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(&format!("{}:1:", program.display())));
    assert!(stderr.contains("error[SES-"));
}
