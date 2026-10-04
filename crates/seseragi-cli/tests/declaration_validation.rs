use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Self::with_nonce(nonce)
    }

    fn with_nonce(nonce: u128) -> Self {
        let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "seseragi-declaration-validation-{}-{nonce}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }

    fn run(&self, command: &str) -> Output {
        let mut process = Command::new(env!("CARGO_BIN_EXE_seseragi"));
        process.arg(command).arg(self.0.join("main.ssrg"));
        if command == "build" {
            process.arg("--out-dir").arg(self.0.join("dist"));
        }
        process.output().unwrap()
    }
}

#[test]
fn fixtures_with_the_same_clock_reading_keep_sources_and_cleanup_isolated() {
    let first = Fixture::with_nonce(0);
    let second = Fixture::with_nonce(0);
    fs::write(
        first.0.join("main.ssrg"),
        "pub effect fn main = println 1\n",
    )
    .unwrap();
    fs::write(
        second.0.join("main.ssrg"),
        "pub effect fn main = println 2\n",
    )
    .unwrap();
    drop(first);
    let output = second.run("run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"2\n");
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn lint_build_and_run_reject_invalid_used_and_unused_declarations_before_output() {
    for (header, usage, message) in [
        (
            "fn add left -> Int = left",
            "add 1",
            "Ordinary functions require",
        ),
        (
            "fn add left: Int = left",
            "add 1",
            "Ordinary functions require",
        ),
        (
            "pub let count = 3",
            "count",
            "Public let declarations require",
        ),
        ("let 1value = 1", "1value", "invalid or reserved name"),
        ("let match = 1", "match", "invalid or reserved name"),
        (
            "println \"unexpected\"",
            "1",
            "Module top level only permits declarations",
        ),
    ] {
        for expression in ["1", usage] {
            let fixture = Fixture::new();
            let source = format!("{header}\npub effect fn main = println (show ({expression}))\n");
            fs::write(fixture.0.join("main.ssrg"), &source).unwrap();
            for command in ["lint", "build", "run"] {
                let output = fixture.run(command);
                let stderr = String::from_utf8_lossy(&output.stderr);
                assert!(!output.status.success(), "{command} accepted {source}");
                assert!(
                    stderr.contains("SES-P0001") && stderr.contains(message),
                    "{command}: {stderr}"
                );
                assert!(
                    output.stdout.is_empty(),
                    "{command} produced output before rejection: {:?}",
                    output.stdout
                );
                assert!(
                    !fixture.0.join("dist").exists(),
                    "invalid source emitted an artifact"
                );
            }
        }
    }
}

#[test]
fn lint_build_and_run_preserve_valid_declarations_and_local_expressions() {
    let fixture = Fixture::new();
    fs::write(
        fixture.0.join("main.ssrg"),
        r#"import * as effects from "std/effect"
pub let count: Int = 3
let 次の値 = count
let account' = 次の値
fn add value: Int -> Int = { let local = 1; value + local }
let pending = effects.succeed (add account')
pub effect fn main = do {
  value <- pending
  println (show value)
}

"#,
    )
    .unwrap();
    for command in ["lint", "build", "run"] {
        let output = fixture.run(command);
        assert!(
            output.status.success(),
            "{command}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        if command == "run" {
            assert_eq!(output.stdout, b"4\n");
        }
    }
}

#[test]
fn package_build_rejects_a_module_expression_before_emission() {
    let fixture = Fixture::new();
    fs::create_dir(fixture.0.join("src")).unwrap();
    fs::write(
        fixture.0.join("seseragi.toml"),
        "[package]\nname = \"fixture/module-expression\"\nversion = \"0.0.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n\n[run]\nentry = \"main\"\ntarget = \"process\"\n",
    ).unwrap();
    fs::write(
        fixture.0.join("src/main.ssrg"),
        "println \"not a declaration\"\npub effect fn main = println \"main\"\n",
    )
    .unwrap();
    let locked = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(["lock", "update"])
        .arg(&fixture.0)
        .output()
        .unwrap();
    assert!(
        locked.status.success(),
        "{}",
        String::from_utf8_lossy(&locked.stderr)
    );
    let built = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("build")
        .arg(&fixture.0)
        .arg("--out-dir")
        .arg(fixture.0.join("dist"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&built.stderr);
    assert!(!built.status.success());
    assert!(
        stderr.contains("SES-P0001")
            && stderr.contains("Module top level only permits declarations"),
        "{stderr}"
    );
    assert!(stderr.contains("main.ssrg:1:"), "{stderr}");
    assert!(!fixture.0.join("dist").exists());
}
