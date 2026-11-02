use seseragi_driver::{compile_module, BuildProfile, CompileInput};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[allow(dead_code)]
#[path = "support/generic_impl_process.rs"]
mod process;
use process::{bounded, runtime, successful};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_owned()
}

struct Program {
    name: String,
    executable: PathBuf,
    entry: Option<PathBuf>,
}

impl Program {
    fn command(&self) -> Command {
        let mut command = Command::new(&self.executable);
        if let Some(entry) = &self.entry {
            command.arg(entry);
        }
        command.env("PATH", "");
        command
    }
}

fn programs(text: &str, directory: &Path) -> Vec<Program> {
    let source = directory.join("main.ssrg");
    fs::write(&source, text).unwrap();
    let mut programs = vec![];
    for (label, profile) in [
        ("development", BuildProfile::Development),
        ("release", BuildProfile::Release),
    ] {
        let dist = directory.join(label);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(&source)
                .args(["--target", "process", "--profile", label, "--out-dir"])
                .arg(&dist),
            directory,
        ));
        let raw = dist.join("raw.ts");
        fs::write(
            &raw,
            compile_module(
                CompileInput::new("main.ssrg", "single-file/main", text).with_profile(profile),
            )
            .unwrap()
            .generated
            .typescript,
        )
        .unwrap();
        let mut files = vec![raw];
        if label == "development" {
            files.extend([dist.join("main.ts"), dist.join("entry.ts")]);
        }
        fs::write(
            dist.join("tsconfig.json"),
            serde_json::to_vec(&serde_json::json!({
                "compilerOptions": {
                    "strict": true, "noEmit": true, "skipLibCheck": false,
                    "allowImportingTsExtensions": true, "target": "ES2022",
                    "module": "ESNext", "moduleResolution": "Bundler",
                    "types": ["bun", "node"], "typeRoots": [root().join("node_modules/@types")],
                    "baseUrl": root(), "paths": {"@seseragi/runtime/*": ["runtime/ts/src/*"]},
                    "lib": ["ES2023", "DOM", "DOM.Iterable"]
                }, "files": files
            }))
            .unwrap(),
        )
        .unwrap();
        successful(bounded(
            Command::new(runtime("node", directory))
                .arg(root().join("node_modules/typescript/bin/tsc"))
                .arg("--project")
                .arg(dist.join("tsconfig.json")),
            directory,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(dist.join("artifact-manifest.json")).unwrap())
                .unwrap();
        for engine in ["bun", "node"] {
            programs.push(Program {
                name: format!("{label}/{engine}"),
                executable: runtime(engine, directory),
                entry: Some(dist.join(manifest["entry"].as_str().unwrap())),
            });
        }
    }
    let out = directory.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(&source)
            .args(["--entry", "main", "--out-dir"])
            .arg(&out),
        directory,
    ));
    programs.push(Program {
        name: "native".into(),
        executable: out.join(format!("program{}", std::env::consts::EXE_SUFFIX)),
        entry: None,
    });
    programs
}

#[test]
fn empty_and_nonempty_log_fields_match_strict_ts_bun_node_and_native() {
    for (fields, expected) in [
        (
            "`[]",
            "{\"level\":\"info\",\"message\":\"x\",\"fields\":[]}\n",
        ),
        (
            "`[(\"count\", logs.LogInt 42)]",
            "{\"level\":\"info\",\"message\":\"x\",\"fields\":[[\"count\",42]]}\n",
        ),
    ] {
        let scratch = tempfile::tempdir().unwrap();
        let directory = scratch.path();
        let text = format!(
            "import * as logs from \"std/log\"\nimport {{ LogEvent }} from \"std/log\"\npub effect fn main = logs.log (LogEvent {{ level: logs.LogInfo, message: \"x\", fields: {fields} }})\n"
        );
        for program in programs(&text, directory) {
            let output = successful(bounded(&mut program.command(), directory));
            assert!(output.stdout.is_empty(), "{}: {output:?}", program.name);
            assert_eq!(output.stderr, expected.as_bytes(), "{}", program.name);
        }
    }
}

#[test]
fn structured_logs_match_strict_ts_bun_node_and_native() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    let text = include_str!("../../../examples/spec/fixtures/rust-backend/structured-log.ssrg");
    let mut expected = None;
    for program in programs(text, directory) {
        let output = successful(bounded(&mut program.command(), directory));
        assert_eq!(output.stdout, b"before\n", "{}", program.name);
        if let Some(expected) = &expected {
            assert_eq!(&output.stderr, expected, "{}", program.name);
        } else {
            let lines = String::from_utf8(output.stderr.clone()).unwrap();
            let events = lines
                .lines()
                .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
                .collect::<Vec<_>>();
            assert_eq!(events.len(), 7);
            assert_eq!(
                events[0],
                serde_json::json!({"level":"trace","message":"","fields":[]})
            );
            assert_eq!(events[4]["fields"][0], serde_json::json!(["same", "first"]));
            assert_eq!(
                events[4]["fields"][1],
                serde_json::json!(["same", "second"])
            );
            assert_eq!(events[4]["fields"][15], serde_json::json!(["nan", null]));
            expected = Some(output.stderr);
        }
    }
    let saved = text.replace("pub effect fn main = do {", "fn identity<A> value: A -> A = value\n\npub effect fn main = do {\n  let string = identity logs.LogString\n  let integer = identity logs.LogInt\n  let floating = identity logs.LogFloat\n  let boolean = identity logs.LogBool")
        .replace("logs.LogString ", "string ")
        .replace("logs.LogInt ", "integer ")
        .replace("logs.LogFloat ", "floating ")
        .replace("logs.LogBool ", "boolean ")
        .replace("let cold = logs.log", "let output = identity logs.log\n  let cold = output");
    let source = directory.join("saved.ssrg");
    fs::write(&source, saved).unwrap();
    let out = directory.join("saved-native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(&source)
            .args(["--entry", "main", "--out-dir"])
            .arg(&out),
        directory,
    ));
    let output = successful(bounded(
        Command::new(out.join(format!("program{}", std::env::consts::EXE_SUFFIX))).env("PATH", ""),
        directory,
    ));
    assert_eq!(output.stdout, b"before\n");
    assert_eq!(output.stderr, expected.unwrap());
}

#[cfg(unix)]
#[test]
fn log_write_failure_matches_ts_error_kind_message_and_debug_redaction() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    let text = "import * as logs from \"std/log\"\nimport * as effects from \"std/effect\"\nimport { LogEvent, LogValue } from \"std/log\"\npub effect fn main = do { let fields: List<(String, LogValue)> = `[]; result <- effects.attempt (logs.log (LogEvent { level: logs.LogInfo, message: \"test\", fields })); match result { Left error -> do { println (show error); println (debug error) }; Right _ -> println \"unexpected success\" } }\n";
    let sink = directory.join("readonly-stderr");
    fs::write(&sink, "").unwrap();
    let wrapper = directory.join("host.mjs");
    // Node also emits stream errors; the host consumes them to observe LogError.
    fs::write(&wrapper, "import { pathToFileURL } from 'node:url';\nprocess.stderr.on('error', () => {});\nawait import(pathToFileURL(process.argv[2]).href);\n").unwrap();
    let shell = runtime("sh", directory);
    let inline = "import * as logs from \"std/log\"\nimport * as effects from \"std/effect\"\nimport { LogEvent } from \"std/log\"\npub effect fn main = do { result <- effects.attempt (logs.log (LogEvent { level: logs.LogInfo, message: \"test\", fields: `[] })); match result { Left error -> do { println (show error); println (debug error) }; Right _ -> println \"unexpected success\" } }\n";
    for text in [text, inline] {
        for program in programs(text, directory) {
            let mut command = Command::new(&shell);
            command
                .args(["-c", "exec \"$@\" 2< \"$SESERAGI_LOG_SINK\"", "log-sink"])
                .arg(&program.executable);
            if let Some(entry) = &program.entry {
                command.arg(&wrapper).arg(entry);
            }
            let output = successful(bounded(
                command.env("SESERAGI_LOG_SINK", &sink).env("PATH", ""),
                directory,
            ));
            assert_eq!(
                output.stdout,
                b"LogError: EBADF: bad file descriptor, write\nLogError { message: \"<redacted>\" }\n",
                "{}",
                program.name
            );
            assert!(output.stderr.is_empty(), "{}: {output:?}", program.name);
            assert_eq!(fs::read(&sink).unwrap(), b"");
        }
    }
}

#[test]
fn structured_logs_preserve_failure_defect_finalizers_and_release_captures() {
    let fixture =
        include_str!("../../../examples/spec/fixtures/rust-backend/structured-log-effects.ssrg");
    for (label, text, code, stdout, messages) in [
        (
            "success",
            fixture.to_owned(),
            0,
            "Right ()\n",
            vec!["acquired", "body", "released"],
        ),
        (
            "failure",
            fixture.replace("body \"ready\"", "body \"fail\""),
            0,
            "Left body failure\n",
            vec!["acquired", "body", "released"],
        ),
        (
            "defect",
            fixture.replace("body \"ready\"", "body \"defect\""),
            70,
            "",
            vec!["acquired", "body", "released"],
        ),
        (
            "argument-defect",
            fixture.replace("event \"body\"", "event (show (9007199254740991 + 1))"),
            70,
            "",
            vec!["acquired", "released"],
        ),
    ] {
        let scratch = tempfile::tempdir().unwrap();
        let directory = scratch.path();
        let prefix = messages
            .into_iter()
            .map(|message| {
                format!("{{\"level\":\"info\",\"message\":\"{message}\",\"fields\":[]}}\n")
            })
            .collect::<String>();
        for program in programs(&text, directory) {
            let output = bounded(&mut program.command(), directory);
            assert_eq!(
                output.status.code(),
                Some(code),
                "{label}/{}: {output:?}",
                program.name
            );
            assert_eq!(output.stdout, stdout.as_bytes(), "{label}/{}", program.name);
            assert!(
                output.stderr.starts_with(prefix.as_bytes()),
                "{label}/{}: {output:?}",
                program.name
            );
            let diagnostic = &output.stderr[prefix.len()..];
            if code == 0 {
                assert!(
                    diagnostic.is_empty(),
                    "{label}/{}: {output:?}",
                    program.name
                );
            } else if program.entry.is_some() {
                assert_eq!(
                    diagnostic, b"seseragi: runtime defect\n",
                    "{label}/{}",
                    program.name
                );
            } else {
                let message = if label == "defect" {
                    "body defect"
                } else {
                    "Seseragi Int overflow"
                };
                assert!(
                    String::from_utf8_lossy(diagnostic).contains(message),
                    "{label}: {output:?}"
                );
            }
            if program.entry.is_none() {
                let metrics = bounded(
                    program.command().env("SESERAGI_NATIVE_METRICS", "1"),
                    directory,
                );
                assert_eq!(metrics.status.code(), Some(code), "{label}: {metrics:?}");
                assert_eq!(metrics.stdout, output.stdout, "{label}");
                assert!(
                    metrics.stderr.starts_with(&output.stderr),
                    "{label}: {metrics:?}"
                );
                let suffix = String::from_utf8_lossy(&metrics.stderr[output.stderr.len()..]);
                assert!(
                    suffix.starts_with("native managed-live=0 collected="),
                    "{label}: {suffix}"
                );
            }
        }
    }
}

#[test]
fn log_rejects_invalid_inputs_before_ts_or_native_outputs() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    for (index, expression) in [
        "logs.log True",
        "logs.LogString True",
        "logs.LogInt \"wrong\"",
        "logs.LogFloat \"wrong\"",
        "logs.LogBool 42",
    ]
    .iter()
    .enumerate()
    {
        let source = directory.join(format!("invalid-{index}.ssrg"));
        let signature = if index == 0 {
            "effect fn main"
        } else {
            "fn main -> logs.LogValue"
        };
        fs::write(
            &source,
            format!("import * as logs from \"std/log\"\npub {signature} = {expression}\n"),
        )
        .unwrap();
        for native in [false, true] {
            let out = directory.join(format!("{index}-{native}"));
            let mut command = Command::new(env!("CARGO_BIN_EXE_seseragi"));
            if native {
                command
                    .args(["experimental", "build-native"])
                    .arg(&source)
                    .args(["--entry", "main"]);
            } else {
                command
                    .arg("build")
                    .arg(&source)
                    .args(["--target", "process"]);
            }
            let output = bounded(command.arg("--out-dir").arg(&out), directory);
            assert_eq!(
                output.status.code(),
                Some(2),
                "{expression}/{native}: {output:?}"
            );
            assert!(
                output.stdout.is_empty(),
                "{expression}/{native}: {output:?}"
            );
            assert!(
                String::from_utf8_lossy(&output.stderr).contains("SES-T0101"),
                "{expression}/{native}: {output:?}"
            );
            assert!(!out.exists(), "{expression}/{native}");
        }
    }
}
