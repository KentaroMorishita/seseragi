//! An `effect fn` that calls itself in tail position without a bind before
//! the call runs in constant host stack on both targets (spec 14.8).
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

#[allow(dead_code)]
#[path = "support/generic_impl_process.rs"]
mod process;
use process::{bounded, successful};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "seseragi-effect-tail-calls-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn runs_on_both_targets(source: &str, stdout: &str) {
    let scratch = Scratch::new();
    let file = scratch.0.join("main.ssrg");
    fs::write(&file, source).unwrap();
    let ts = successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("run")
            .arg(&file),
        &scratch.0,
    ));
    assert_eq!(String::from_utf8_lossy(&ts.stdout), stdout);
    let native = successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["run", "--target", "native"])
            .arg(&file),
        &scratch.0,
    ));
    assert_eq!(String::from_utf8_lossy(&native.stdout), stdout);
}

#[test]
fn effect_self_tail_calls_from_if_and_match_run_a_million_times() {
    runs_on_both_targets(
        include_str!("../../../examples/spec/fixtures/rust-backend/effect-tail-recursion.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/effect-tail-recursion.stdout"),
    );
}

/// Arguments are evaluated when the call is made (spec 3.1), so a defect in
/// one stops the program even when the Effect the call builds never runs.
#[test]
fn effect_tail_call_arguments_are_evaluated_before_the_call() {
    let scratch = Scratch::new();
    let file = scratch.0.join("main.ssrg");
    fs::write(
        &file,
        "effect fn spin n: Int -> Unit fails Never =\n  if n == 0 then succeed () else spin (n / 0)\n\npub effect fn main -> Unit with Console fails ConsoleError = do {\n  let unused = spin 1\n  println \"after\"\n}\n",
    )
    .unwrap();
    for target in ["process", "native"] {
        let output = bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .args(["run", "--target", target])
                .arg(&file),
            &scratch.0,
        );
        assert_eq!(output.status.code(), Some(70), "{target}");
        assert_eq!(String::from_utf8_lossy(&output.stdout), "", "{target}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("Int division by zero"),
            "{target}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
