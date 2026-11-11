//! A module-level `rec` group (spec 2.9) and an unannotated lambda whose
//! parameter type comes from a sibling argument run the same on both targets.
use std::{
    fs,
    path::{Path, PathBuf},
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
            "seseragi-rec-groups-{}-{}",
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

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/spec/fixtures/rust-backend")
        .join(name)
}

fn runs_on_both_targets(name: &str) {
    let source = fixture(&format!("{name}.ssrg"));
    let stdout = fs::read_to_string(fixture(&format!("{name}.stdout"))).unwrap();
    let scratch = Scratch::new();
    for target in ["process", "native"] {
        let output = successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .args(["run", "--target", target])
                .arg(&source),
            &scratch.0,
        ));
        assert_eq!(String::from_utf8_lossy(&output.stdout), stdout, "{target}");
    }
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["format", "--check"])
            .arg(&source),
        &scratch.0,
    ));
}

fn refused(source: &str, message: &str) {
    let scratch = Scratch::new();
    let file = scratch.0.join("main.ssrg");
    fs::write(&file, source).unwrap();
    let output = bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("run")
            .arg(&file),
        &scratch.0,
    );
    assert!(!output.status.success(), "{source}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(message), "{source}\n{stderr}");
}

#[test]
fn a_module_level_rec_group_runs_on_both_targets() {
    runs_on_both_targets("top-level-rec-group");
}

#[test]
fn a_module_level_rec_group_takes_no_modifiers_and_only_functions() {
    let main = "\n\npub effect fn main -> Unit with Console fails ConsoleError = println \"x\"\n";
    refused(
        &format!("pub rec {{\n  fn a n: Int -> Int = n\n}}{main}"),
        "Declaration modifiers are incomplete or invalid",
    );
    refused(
        &format!("rec {{\n  let x = 1\n}}{main}"),
        "Module top level only permits declarations",
    );
    refused(
        &format!("rec {{}}{main}"),
        "Module top level only permits declarations",
    );
}

#[test]
fn a_lambda_wrapped_on_the_left_of_apply_takes_its_parameter_type_from_the_right() {
    runs_on_both_targets("applicative-lambda-inference");
}
