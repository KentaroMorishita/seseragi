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
            "seseragi-runtime-errors-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(fs::canonicalize(path).unwrap())
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn compare(source: &str, marker: &str, message: &str, defect: bool) {
    compare_with_initialization(source, marker, message, defect, false);
}

fn compare_with_initialization(
    source: &str,
    marker: &str,
    message: &str,
    defect: bool,
    initializing: bool,
) {
    let scratch = Scratch::new();
    let file = scratch.0.join("probe-雪🙂.ssrg");
    fs::write(&file, source).unwrap();
    compare_program(
        &scratch,
        &file,
        &file,
        "probe-雪🙂.ssrg",
        source,
        marker,
        message,
        defect,
        initializing,
    );
}

fn compare_program(
    scratch: &Scratch,
    entry: &Path,
    file: &Path,
    relative_file: &str,
    source: &str,
    marker: &str,
    message: &str,
    defect: bool,
    initializing: bool,
) {
    let plain = bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("run")
            .arg(entry),
        &scratch.0,
    );
    let ts = bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("run")
            .arg(entry)
            .args(["--diagnostic-format", "json"]),
        &scratch.0,
    );
    let code = if defect { 70 } else { 1 };
    let start = source.find(marker).unwrap();
    let prefix = &source[..start];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = prefix.rsplit('\n').next().unwrap().chars().count() + 1;
    assert_eq!(ts.status.code(), Some(code), "{ts:?}");
    assert_eq!(plain.status.code(), Some(code), "{plain:?}");
    assert!(plain.stdout.is_empty());
    assert!(ts.stdout.is_empty());
    let diagnostic: serde_json::Value = serde_json::from_slice(&ts.stderr).unwrap();
    assert_eq!(
        diagnostic["kind"],
        if defect { "Defect" } else { "TypedFailure" }
    );
    assert_eq!(diagnostic["message"], message);
    if defect {
        let frame = &diagnostic["groups"][0]["frames"][0];
        assert_eq!(frame["language"], "seseragi");
        assert_eq!(frame["range"]["start"], start);
        assert_eq!(frame["generated"], false);
    }
    assert_eq!(
        String::from_utf8(plain.stderr).unwrap(),
        if defect {
            format!(
                "{}:{line}:{column}: Seseragi Defect: {message}{}\n",
                relative_file,
                if initializing {
                    " (during module initialization)"
                } else {
                    ""
                }
            )
        } else if message.ends_with('\n') {
            message.to_owned()
        } else {
            format!("{message}\n")
        }
    );
    let output = scratch.0.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(entry)
            .args(["--entry", "main", "--out-dir"])
            .arg(&output),
        &scratch.0,
    ));
    fs::remove_file(&file).unwrap();
    let native = bounded(
        &mut Command::new(output.join(format!("program{}", std::env::consts::EXE_SUFFIX))),
        &scratch.0,
    );
    assert_eq!(native.status.code(), ts.status.code(), "{native:?}");
    assert_eq!(native.stdout, ts.stdout);
    let label = if defect { "Seseragi Defect: " } else { "" };
    assert_eq!(
        String::from_utf8(native.stderr).unwrap(),
        format!(
            "{}:{line}:{column}: {label}{message}{}",
            relative_file,
            if message.ends_with('\n') { "" } else { "\n" }
        )
    );
}

#[test]
fn native_defects_keep_the_innermost_source_position_and_ts_exit_contract() {
    for (source, marker, message) in [
        ("fn boom n: Int -> Int = n / 0\n\npub effect fn main -> Unit with { console: Console } fails ConsoleError =\n  println `${boom 3}`\n", "n / 0", "Seseragi Int division by zero"),
        ("import * as effects from \"std/effect\"\nfn broken value: Int -> Int = value / 0\npub effect fn main -> Unit = do {\n  let value = broken 4\n  effects.succeed ()\n}\n", "value / 0", "Seseragi Int division by zero"),
        ("import * as effects from \"std/effect\"\npub effect fn main -> Unit = do {\n  let value = 9007199254740991 + 1\n  effects.succeed ()\n}\n", "9007199254740991 + 1", "Seseragi Int overflow"),
        ("import * as effects from \"std/effect\"\nimport * as ints from \"std/int\"\npub effect fn main -> Unit = do {\n  let value = ints.clamp 4 2 1\n  effects.succeed ()\n}\n", "ints.clamp 4 2 1", "Seseragi Int clamp lower bound exceeds upper bound"),
        ("import * as effects from \"std/effect\"\npub effect fn main -> Unit = effects.die \"explicit panic\"\n", "effects.die \"explicit panic\"", "explicit panic"),
        ("import * as effects from \"std/effect\"\nfn saved message: String -> Effect<{}, Never, Never> = {\n  let create = effects.die\n  create message\n}\npub effect fn main -> Unit = saved \"saved panic\"\n", "create message", "saved panic"),
        ("import * as effects from \"std/effect\"\npub effect fn main -> Unit = do {\n  let values = map (\\value: Int -> value / 0) [1]\n  effects.succeed ()\n}\n", "value / 0", "Seseragi Int division by zero"),
        ("import * as effects from \"std/effect\"\npub effect fn main -> Unit = do {\n  let text = \"雪🙂\"; let value = 1 / 0\n  effects.succeed ()\n}\n", "1 / 0", "Seseragi Int division by zero"),
    ] { compare(source, marker, message, true); }
    // JavaScript imports report the initialization phase as well as the exact
    // expression location; native execution retains its existing text format.
    compare_with_initialization("import * as effects from \"std/effect\"\nlet broken: Int = 1 / 0\npub effect fn main -> Unit = effects.succeed ()\n", "1 / 0", "Seseragi Int division by zero", true, true);
}

#[test]
fn unhandled_native_errors_use_user_and_standard_show_instances() {
    compare("import * as effects from \"std/effect\"\npub effect fn main -> Unit fails String = effects.fail \"first\\nsecond\\n\"\n", "effects.fail", "first\nsecond\n", false);
    compare("import * as effects from \"std/effect\"\nstruct Problem { code: Int }\ninstance Show<Problem> {\n  fn show value: Problem -> String = \"problem \" + show value.code\n}\npub effect fn main -> Unit fails Problem = effects.fail (Problem { code: 7 })\n", "effects.fail (Problem", "problem 7", false);
    compare("import * as effects from \"std/effect\"\nimport * as paths from \"std/path\"\npub effect fn main -> Unit fails paths.PathError = effects.fail (paths.InvalidPathSegment \"a/b\")\n", "effects.fail (paths", "InvalidPathSegment \"a/b\"", false);
}

#[test]
fn imported_failures_keep_the_dependency_file_and_its_show_instance() {
    let scratch = Scratch::new();
    fs::create_dir_all(scratch.0.join("src/errors")).unwrap();
    fs::write(scratch.0.join("seseragi.toml"), "[package]\nname = \"fixture/runtime-errors\"\nversion = \"0.0.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n[run]\nentry = \"main\"\ntarget = \"process\"\n").unwrap();
    fs::write(scratch.0.join("src/main.ssrg"), "import { Problem, failed } from \"./errors/main\"\npub effect fn main -> Unit fails Problem = failed ()\n").unwrap();
    let file = scratch.0.join("src/errors/main.ssrg");
    let source = "import * as effects from \"std/effect\"\npub type Problem = | Problem Int\ninstance Show<Problem> {\n  fn show error: Problem -> String = match error { Problem code -> \"remote \" + show code }\n}\npub effect fn failed -> Unit fails Problem = effects.fail (Problem 9)\n";
    fs::write(&file, source).unwrap();
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["lock", "update"])
            .arg(&scratch.0),
        &scratch.0,
    ));
    compare_program(
        &scratch,
        &scratch.0,
        &file,
        "src/errors/main.ssrg",
        source,
        "effects.fail (Problem",
        "remote 9",
        false,
        false,
    );
}

#[test]
fn imported_defects_keep_the_innermost_dependency_source_in_both_targets() {
    let scratch = Scratch::new();
    fs::create_dir_all(scratch.0.join("src/errors")).unwrap();
    fs::write(scratch.0.join("seseragi.toml"), "[package]\nname = \"fixture/runtime-errors\"\nversion = \"0.0.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n[run]\nentry = \"main\"\ntarget = \"process\"\n").unwrap();
    fs::write(scratch.0.join("src/main.ssrg"), "import { broken } from \"./errors/main\"\npub effect fn main -> Unit with { console: Console } fails ConsoleError = println (show (broken 3))\n").unwrap();
    let file = scratch.0.join("src/errors/main.ssrg");
    let source = "pub fn broken value: Int -> Int = {\n  let text = \"雪🙂\"; let result = value / 0\n  result\n}\n";
    fs::write(&file, source).unwrap();
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["lock", "update"])
            .arg(&scratch.0),
        &scratch.0,
    ));
    compare_program(
        &scratch,
        &scratch.0,
        &file,
        "src/errors/main.ssrg",
        source,
        "value / 0",
        "Seseragi Int division by zero",
        true,
        false,
    );
}

#[test]
fn path_dependencies_use_their_own_package_relative_runtime_paths() {
    let scratch = Scratch::new();
    fs::create_dir_all(scratch.0.join("src")).unwrap();
    fs::create_dir_all(scratch.0.join("vendor/crasher/src/errors")).unwrap();
    fs::write(scratch.0.join("seseragi.toml"), "[package]\nname = \"fixture/runtime-consumer\"\nversion = \"0.0.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n[run]\nentry = \"main\"\ntarget = \"process\"\n[dependencies]\ncrasher = { package = \"fixture/crasher\", path = \"vendor/crasher\" }\n").unwrap();
    fs::write(scratch.0.join("vendor/crasher/seseragi.toml"), "[package]\nname = \"fixture/crasher\"\nversion = \"1.0.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n[exports]\n\".\" = \"errors/crash\"\n").unwrap();
    fs::write(
        scratch.0.join("src/main.ssrg"),
        "import { broken } from \"crasher\"\npub effect fn main = println (show (broken 3))\n",
    )
    .unwrap();
    let file = scratch.0.join("vendor/crasher/src/errors/crash.ssrg");
    let source = "pub fn broken value: Int -> Int = value / 0\n";
    fs::write(&file, source).unwrap();
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["lock", "update"])
            .arg(&scratch.0),
        &scratch.0,
    ));
    compare_program(
        &scratch,
        &scratch.0,
        &file,
        "src/errors/crash.ssrg",
        source,
        "value / 0",
        "Seseragi Int division by zero",
        true,
        false,
    );
}
