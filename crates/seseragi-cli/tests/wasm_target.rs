//! The `wasm` project target: `seseragi build` and `seseragi dev` build a
//! package's program WASM library from `run.entry` and `run.exports`.

use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "seseragi-wasm-target-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// A child in its own process group, killed with its compiler descendants
/// when the test ends.
struct OwnedChild(Child);
impl OwnedChild {
    fn start(command: &mut Command, logs: &Path) -> Self {
        command
            .stdout(Stdio::from(File::create(logs.join("stdout")).unwrap()))
            .stderr(Stdio::from(File::create(logs.join("stderr")).unwrap()));
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        Self(command.spawn().unwrap())
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            #[cfg(unix)]
            unsafe {
                unsafe extern "C" {
                    fn kill(pid: i32, signal: i32) -> i32;
                }
                kill(-(self.0.id() as i32), 15);
                std::thread::sleep(Duration::from_secs(2));
                kill(-(self.0.id() as i32), 9);
            }
            let _ = self.0.kill();
        }
        let _ = self.0.wait();
    }
}

fn seseragi(directory: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_seseragi"));
    command.current_dir(directory);
    command
}

fn output(command: &mut Command) -> Output {
    command.output().unwrap()
}

fn successful(output: Output) -> Output {
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

const PROGRAM: &str = "pub fn label value: Int -> String = \"合計: \" + show value\npub fn twice value: Int -> Int = value * 2\npub fn same<A> left: A -> right: A -> Bool where Eq<A> = left == right\n";
const EXPORTS: [&str; 3] = ["label", "twice", "sameText=same<String>"];

/// A package whose entry module `calculation` holds `PROGRAM`, with `run`
/// as its `[run]` table.
fn package(scratch: &Scratch, run: &str) -> PathBuf {
    let root = scratch.0.join("library");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("seseragi.toml"),
        format!("[package]\nname = \"fixture/library\"\nversion = \"0.1.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n\n[run]\nentry = \"calculation\"\n{run}"),
    )
    .unwrap();
    fs::write(root.join("src/calculation.ssrg"), PROGRAM).unwrap();
    successful(output(seseragi(&root).args(["lock", "update", "."])));
    root
}

fn exports_line() -> String {
    format!(
        "exports = [{}]\n",
        EXPORTS
            .iter()
            .map(|export| format!("{export:?}"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn check_program(directory: &Path, twice: i64) {
    let script = directory.join("check.mjs");
    fs::write(
        &script,
        format!(
            r#"import {{readFile}} from 'node:fs/promises';
import assert from 'node:assert/strict';
import {{loadProgram}} from './program.mjs';
const program = await loadProgram(await readFile(new URL('./program.wasm', import.meta.url)));
assert.equal(program.exports.label(3), '合計: 3');
assert.equal(program.exports.twice(21), {});
assert.equal(program.exports.sameText('a')('a'), true);
program.dispose();
console.log('checked');
"#,
            21 * twice
        ),
    )
    .unwrap();
    let result = successful(output(Command::new("node").arg(&script)));
    assert_eq!(result.stdout, b"checked\n");
    fs::remove_file(script).unwrap();
}

#[test]
fn wasm_run_target_builds_the_entry_exports_as_build_wasm_does() {
    let scratch = Scratch::new();
    let root = package(&scratch, &format!("target = \"wasm\"\n{}", exports_line()));
    let built = successful(output(seseragi(&root).args(["build", "."])));
    assert!(String::from_utf8_lossy(&built.stdout).starts_with("Built . -> "));
    let dist = root.join("dist");
    check_program(&dist, 2);
    // The same selection through `experimental build-wasm` gives the same
    // files, byte for byte.
    let mut experimental = seseragi(&root);
    experimental.args(["experimental", "build-wasm", "."]);
    for export in EXPORTS {
        experimental.args(["--export", export]);
    }
    successful(output(experimental.args(["--out-dir", "experimental"])));
    for file in ["program.wasm", "program.mjs", "program.d.mts"] {
        assert_eq!(
            fs::read(dist.join(file)).unwrap(),
            fs::read(root.join("experimental").join(file)).unwrap(),
            "{file}"
        );
    }
    // A library has no main to run.
    let run = output(seseragi(&root).args(["run", "."]));
    assert!(!run.status.success());
    assert!(
        String::from_utf8_lossy(&run.stderr).contains("run does not support the `wasm` target"),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    // The release profile builds too, and a file outside the artifact keeps
    // the destination from being replaced.
    successful(output(seseragi(&root).args([
        "build",
        ".",
        "--profile",
        "release",
        "--out-dir",
        "release",
    ])));
    check_program(&root.join("release"), 2);
    fs::write(dist.join("notes.txt"), "mine").unwrap();
    let refused = output(seseragi(&root).args(["build", "."]));
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("refusing to replace"));
    assert_eq!(fs::read_to_string(dist.join("notes.txt")).unwrap(), "mine");
    // Options of the other targets are refused instead of ignored.
    let mapped = output(seseragi(&root).args(["build", ".", "--source-map", "emit"]));
    assert!(String::from_utf8_lossy(&mapped.stderr).contains("not supported by the wasm target"));
}

#[test]
fn wasm_invocation_target_builds_run_exports_of_another_package() {
    let scratch = Scratch::new();
    let root = package(&scratch, &exports_line());
    successful(output(seseragi(&root).args([
        "build",
        "src/calculation.ssrg",
        "--target",
        "wasm",
        "--out-dir",
        "library",
    ])));
    check_program(&root.join("library"), 2);
    // Without run.exports there is nothing to build.
    let other = Scratch::new();
    let bare = package(&other, "");
    let missing = output(seseragi(&bare).args(["build", ".", "--target", "wasm"]));
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("add run.exports"));
}

#[test]
fn dev_rebuilds_the_wasm_target_library_when_its_source_changes() {
    let scratch = Scratch::new();
    let root = package(&scratch, &format!("target = \"wasm\"\n{}", exports_line()));
    let logs = scratch.0.join("logs");
    fs::create_dir(&logs).unwrap();
    // The library has no dev server, so web options are refused.
    let served = output(seseragi(&root).args(["dev", ".", "--port", "0"]));
    assert!(String::from_utf8_lossy(&served.stderr).contains("require --target web"));
    let _dev = OwnedChild::start(seseragi(&root).args(["dev", "."]), &logs);
    let output = root.join(".seseragi/dev-wasm");
    let manifest = output.join("program-wasm.json");
    let wait = |done: &dyn Fn() -> bool| {
        let deadline = Instant::now() + Duration::from_secs(180);
        while !done() {
            assert!(
                Instant::now() < deadline,
                "{}",
                fs::read_to_string(logs.join("stderr")).unwrap_or_default()
            );
            std::thread::sleep(Duration::from_millis(100));
        }
    };
    wait(&|| manifest.is_file());
    check_program(&output, 2);
    let first = fs::read(&manifest).unwrap();
    fs::write(
        root.join("src/calculation.ssrg"),
        PROGRAM.replace("value * 2", "value * 3"),
    )
    .unwrap();
    wait(&|| fs::read(&manifest).is_ok_and(|current| current != first));
    check_program(&output, 3);
    assert!(fs::read_to_string(logs.join("stdout"))
        .unwrap()
        .contains("Watching"));
}

#[test]
fn wasm_target_takes_the_manifest_profile_unless_the_invocation_names_one() {
    let scratch = Scratch::new();
    let root = scratch.0.join("profiled");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("seseragi.toml"),
        "[package]\nname = \"fixture/profiled\"\nversion = \"0.1.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n\n[run]\nentry = \"score\"\ntarget = \"wasm\"\nexports = [\"score\"]\n\n[build]\nprofile = \"release\"\n",
    )
    .unwrap();
    fs::write(
        root.join("src/score.ssrg"),
        "pub newtype Score = Int\npub fn score value: Int -> Score = Score value\n",
    )
    .unwrap();
    successful(output(seseragi(&root).args(["lock", "update", "."])));
    // The release profile erases the newtype wrapper, as the TS target does.
    successful(output(seseragi(&root).args(["build", "."])));
    let declared = fs::read_to_string(root.join("dist/program.d.mts")).unwrap();
    assert!(
        declared.contains("export type Score = number;"),
        "{declared}"
    );
    successful(output(seseragi(&root).args([
        "build",
        ".",
        "--profile",
        "development",
        "--out-dir",
        "development",
    ])));
    let declared = fs::read_to_string(root.join("development/program.d.mts")).unwrap();
    assert!(
        declared.contains("export type Score = Readonly<{tag: \"Score\"; value: number}>;"),
        "{declared}"
    );
}
