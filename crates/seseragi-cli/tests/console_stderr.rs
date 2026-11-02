#![cfg(unix)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[allow(dead_code)]
#[path = "support/generic_impl_process.rs"]
mod process;
use process::{bounded, runtime, successful};

struct Program {
    name: String,
    executable: PathBuf,
    entry: Option<PathBuf>,
}

impl Program {
    fn command(&self, directory: &Path) -> Command {
        let mut command = Command::new(&self.executable);
        if let Some(entry) = &self.entry {
            command.arg(directory.join("observe.mjs")).arg(entry);
        }
        command
    }

    fn assert_broken_stderr(&self, directory: &Path, code: i32, stdout: &str) {
        let command = self.command(directory);
        let output = successful(bounded(
            Command::new(runtime("python3", directory))
                .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/broken-stderr.py"))
                .arg(command.get_program())
                .args(command.get_args()),
            directory,
        ));
        assert!(output.stderr.is_empty(), "{}: {output:?}", self.name);
        let observed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            observed,
            serde_json::json!({ "code": code, "stdout": stdout }),
            "{}",
            self.name
        );
    }
}

fn programs(source: &str, directory: &Path) -> Vec<Program> {
    let input = directory.join("main.ssrg");
    fs::write(&input, source).unwrap();
    fs::write(
        directory.join("observe.mjs"),
        r#"import { pathToFileURL } from "node:url";
const listeners = process.stderr.listenerCount("error");
process.on("uncaughtExceptionMonitor", error => {
  process.stdout.write(`uncaughtException: ${error.code} ${error.message}\n`);
});
process.on("unhandledRejection", error => {
  process.stdout.write(`unhandledRejection: ${error}\n`);
  process.exitCode = 99;
});
process.on("warning", warning => {
  process.stdout.write(`warning: ${warning}\n`);
  process.exitCode = 99;
});
await import(pathToFileURL(process.argv[2]).href);
await new Promise(resolve => setImmediate(resolve));
if (process.stderr.listenerCount("error") !== listeners) {
  process.stdout.write("stderr error listener leaked\n");
  process.exitCode = 99;
}
"#,
    )
    .unwrap();
    let mut programs = vec![];
    for profile in ["development", "release"] {
        let out = directory.join(profile);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(&input)
                .args([
                    "--target",
                    "process",
                    "--source-map",
                    "omit",
                    "--profile",
                    profile,
                    "--out-dir",
                ])
                .arg(&out),
            directory,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(out.join("artifact-manifest.json")).unwrap()).unwrap();
        for engine in ["node", "bun"] {
            programs.push(Program {
                name: format!("{profile}/{engine}"),
                executable: runtime(engine, directory),
                entry: Some(out.join(manifest["entry"].as_str().unwrap())),
            });
        }
    }
    let out = directory.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(&input)
            .args(["--entry", "main", "--out-dir"])
            .arg(&out),
        directory,
    ));
    programs.push(Program {
        name: "native".into(),
        executable: out.join("program"),
        entry: None,
    });
    programs
}

#[test]
fn stderr_preserves_empty_unicode_and_newline_output_on_ts_and_native() {
    let scratch = tempfile::tempdir().unwrap();
    let source = "import * as console from \"std/console\"\npub effect fn main = do { console.error \"\"; console.error \"雪🙂\\0\"; console.errorLine \"\"; console.errorLine \"x\\n\"; println \"after\" }\n";
    for program in programs(source, scratch.path()) {
        let output = successful(bounded(
            &mut program.command(scratch.path()),
            scratch.path(),
        ));
        assert_eq!(output.stdout, b"after\n", "{}", program.name);
        assert_eq!(
            output.stderr,
            "雪🙂\0\nx\n\n".as_bytes(),
            "{}",
            program.name
        );
    }
}

#[test]
fn recovered_stderr_failure_continues_on_ts_and_native() {
    for operation in ["error", "errorLine"] {
        let scratch = tempfile::tempdir().unwrap();
        let source = format!("import * as console from \"std/console\"\nimport * as effects from \"std/effect\"\neffect fn ignore _error: ConsoleError -> Unit with Console = effects.succeed ()\npub effect fn main = do {{ console.{operation} \"x\" |> effects.recover ignore; println \"after\" }}\n");
        for program in programs(&source, scratch.path()) {
            program.assert_broken_stderr(scratch.path(), 0, "after\n");
        }
    }
}

#[test]
fn repeated_stderr_failures_keep_console_error_kind_on_ts_and_native() {
    let scratch = tempfile::tempdir().unwrap();
    let source = "import * as console from \"std/console\"\nimport * as effects from \"std/effect\"\neffect fn report error: ConsoleError -> Unit with Console fails ConsoleError = println (debug error)\npub effect fn main = do { console.error \"x\" |> effects.recover report; console.errorLine \"\" |> effects.recover report; console.errorLine \"雪🙂\" |> effects.recover report; println \"after\" }\n";
    let expected = "ConsoleError { message: \"<redacted>\" }\n".repeat(3) + "after\n";
    for program in programs(source, scratch.path()) {
        program.assert_broken_stderr(scratch.path(), 0, &expected);
    }
}

#[test]
fn unrecovered_stderr_failure_stops_following_output_on_ts_and_native() {
    let scratch = tempfile::tempdir().unwrap();
    let source = "import * as console from \"std/console\"\npub effect fn main = do { println \"before\"; console.errorLine \"x\"; println \"after\" }\n";
    for program in programs(source, scratch.path()) {
        // Native diagnostics currently panic when stderr is broken.
        let code = if program.entry.is_some() { 1 } else { 101 };
        program.assert_broken_stderr(scratch.path(), code, "before\n");
    }
}

#[test]
fn diagnostic_write_failure_preserves_ts_failure_and_defect_exit_codes() {
    for (body, code) in [
        ("effects.fail \"stopped\"", 1),
        ("effects.die \"broken\"", 70),
    ] {
        let scratch = tempfile::tempdir().unwrap();
        let source = format!("import * as effects from \"std/effect\"\npub effect fn main -> Unit fails String = {body}\n");
        for program in programs(&source, scratch.path()) {
            // Native diagnostics currently panic when stderr is broken.
            let code = if program.entry.is_some() { code } else { 101 };
            program.assert_broken_stderr(scratch.path(), code, "");
        }
    }
}
