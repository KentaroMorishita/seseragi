#[path = "support/generic_impl_process.rs"]
mod process;

use process::{bounded, runtime, successful};
use seseragi_driver::{compile_module, BuildProfile, CompileInput};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

struct Program {
    name: String,
    executable: PathBuf,
    entry: Option<PathBuf>,
}

impl Program {
    fn run(&self, input: &[u8], directory: &Path) -> Output {
        let input_path = directory.join("input");
        fs::write(&input_path, input).unwrap();
        let mut command = Command::new(&self.executable);
        if let Some(entry) = &self.entry {
            command.arg(entry);
        }
        let output = successful(bounded(
            command
                .env("PATH", "")
                .env("SESERAGI_NATIVE_METRICS", "1")
                .stdin(Stdio::from(File::open(input_path).unwrap())),
            directory,
        ));
        if self.entry.is_none() {
            assert!(
                String::from_utf8_lossy(&output.stderr).contains("managed-live=0"),
                "{}: {output:?}",
                self.name
            );
        } else {
            assert!(output.stderr.is_empty(), "{}: {output:?}", self.name);
        }
        output
    }
}

fn programs(text: &str, directory: &Path) -> Vec<Program> {
    let source = directory.join("main.ssrg");
    fs::write(&source, text).unwrap();
    let mut programs = Vec::new();
    for (label, profile) in [
        ("development", BuildProfile::Development),
        ("release", BuildProfile::Release),
    ] {
        let dist = directory.join(label);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(&source)
                .args([
                    "--target",
                    "process",
                    "--source-map",
                    "omit",
                    "--profile",
                    label,
                    "--out-dir",
                ])
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
        let files = match profile {
            BuildProfile::Development => vec![raw, dist.join("main.ts"), dist.join("entry.ts")],
            BuildProfile::Release => vec![raw],
        };
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
    let native = directory.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(&source)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native),
        directory,
    ));
    programs.push(Program {
        name: "native".into(),
        executable: native.join(format!("program{}", std::env::consts::EXE_SUFFIX)),
        entry: None,
    });
    programs
}

#[test]
fn issue_examples_pass_strict_typescript_and_run_on_all_hosts() {
    for text in [
        "import * as console from \"std/console\"\npub effect fn main = { let saved = console.flush; saved () }\n",
        "import * as stdin from \"std/stdin\"\nimport * as effects from \"std/effect\"\npub effect fn main = do { let saved = stdin.readLine; saved (); effects.succeed () }\n",
    ] {
        let scratch = tempfile::tempdir().unwrap();
        for program in programs(text, scratch.path()) {
            assert_eq!(program.run(b"line\n", scratch.path()).stdout, b"", "{}", program.name);
        }
    }
}

const SAVED: &str = r#"import * as console from "std/console"
import * as stdin from "std/stdin"
import * as effects from "std/effect"

effect fn report action: Effect<{stdin: Stdin}, StdinError, Maybe<String>> -> Unit
with Stdin, Console fails String = do {
  result <- effects.attempt action
  println (show result) |> effects.mapError show
}

pub effect fn main -> Unit with Stdin, Console fails String = do {
  let flush = console.flush
  let read = stdin.readLine
  let unused = stdin.readLine
  let cold = read ()
  println "saved" |> effects.mapError show
  flush () |> effects.mapError show
  report (read ())
  report (read ())
  report cold
  report cold
  flush () |> effects.mapError show
}
"#;

#[test]
fn saved_functions_and_reused_effects_preserve_stdin_results_and_errors() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    let programs = programs(SAVED, directory);
    for (input, expected) in [
        (b"first\nsecond\nthird\nfourth".as_slice(), "saved\nRight Just first\nRight Just second\nRight Just third\nRight Just fourth\n"),
        (b"", "saved\nRight Nothing\nRight Nothing\nRight Nothing\nRight Nothing\n"),
        ("\r\n雪🙂\n\u{feff}last\r".as_bytes(), "saved\nRight Just \nRight Just 雪🙂\nRight Just last\r\nRight Nothing\n"),
        (b"\xff\nok\n\xc3\x28\nlast\n", "saved\nLeft InvalidStdinUtf8 { offset: 0 }\nRight Just ok\nLeft InvalidStdinUtf8 { offset: 5 }\nRight Just last\n"),
    ] {
        for program in &programs {
            assert_eq!(program.run(input, directory).stdout, expected.as_bytes(), "{}", program.name);
        }
    }
    for length in [1_048_576, 1_048_577] {
        let input = [vec![b'x'; length], b"\nnext\n".to_vec()].concat();
        let first = if length == 1_048_576 {
            format!("Right Just {}", "x".repeat(length))
        } else {
            "Left StdinLineTooLong { limitBytes: 1048576 }".into()
        };
        let expected = format!("saved\n{first}\nRight Just next\nRight Nothing\nRight Nothing\n");
        for program in &programs {
            assert_eq!(
                program.run(&input, directory).stdout,
                expected.as_bytes(),
                "{} / {length}",
                program.name
            );
        }
    }
}

#[test]
fn saved_functions_stay_cold_and_call_the_service_on_each_execution() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    let text = r#"import * as console from "std/console"
import * as stdin from "std/stdin"
import * as effects from "std/effect"
pub effect fn main -> Unit with Stdin, Console fails ConsoleError = do {
  let flush = console.flush
  let read = stdin.readLine
  let unusedFlush = console.flush
  let unusedRead = stdin.readLine
  let coldFlush = flush ()
  let coldRead = read ()
  a <- effects.attempt (flush ())
  println (show a)
  b <- effects.attempt coldFlush
  println (show b)
  c <- effects.attempt coldFlush
  println (show c)
  x <- effects.attempt (read ())
  println (show x)
  y <- effects.attempt coldRead
  println (show y)
  z <- effects.attempt coldRead
  println (show z)
}
"#;
    for program in programs(text, directory) {
        assert_eq!(
            program.run(b"first\nsecond\nthird\n", directory).stdout,
            b"Right ()\nRight ()\nRight ()\nRight Just first\nRight Just second\nRight Just third\n",
            "{}",
            program.name
        );
    }
    for profile in ["development", "release"] {
        let dist = directory.join(profile);
        let probe = dist.join("probe.ts");
        let mut script = fs::read_to_string(dist.join("raw.ts")).unwrap();
        script.push_str(r#"
import assert from "node:assert/strict";
import { run as runProbe } from "@seseragi/runtime/effect";
import { liveConsole, type ConsoleEnvironment as ProbeConsole } from "@seseragi/runtime/console";
import { StdinReadFailure, type StdinEnvironment as ProbeStdin } from "@seseragi/runtime/stdin";
import { Just as probeJust } from "@seseragi/runtime/sum";
import { serviceSuccess as probeSuccess, serviceFailure as probeFailure } from "@seseragi/runtime/service";
let flushes = 0;
let reads = 0;
const lines: string[] = [];
const environment: ProbeConsole & ProbeStdin = {
  console: {
    ...liveConsole,
    println(value) { lines.push(value); return probeSuccess(undefined); },
    flush() {
      flushes++;
      return flushes === 2
        ? probeFailure({ kind: "console-error", message: "flush failed" })
        : probeSuccess(undefined);
    },
  },
  stdin: {
    readChunk() { throw new Error("unexpected chunk read"); },
    readLine() {
      reads++;
      return reads === 2
        ? probeFailure(StdinReadFailure)
        : probeSuccess(probeJust(`line${reads}`));
    },
  },
};
const plan = main(undefined);
assert.equal(flushes, 0);
assert.equal(reads, 0);
assert.deepEqual(lines, []);
assert.equal((await runProbe(plan, environment)).kind, "success");
assert.equal(flushes, 3);
assert.equal(reads, 3);
assert.deepEqual(lines, [
  "Right ()", "Left ConsoleError: flush failed", "Right ()",
  "Right Just line1", "Left StdinReadFailure", "Right Just line3",
]);
"#);
        fs::write(&probe, &script).unwrap();
        let config_path = dist.join("tsconfig.json");
        let mut config: serde_json::Value =
            serde_json::from_slice(&fs::read(&config_path).unwrap()).unwrap();
        config["files"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!(probe));
        fs::write(&config_path, serde_json::to_vec(&config).unwrap()).unwrap();
        successful(bounded(
            Command::new(runtime("node", directory))
                .arg(root().join("node_modules/typescript/bin/tsc"))
                .arg("--project")
                .arg(&config_path),
            directory,
        ));
        let runtime_prefix = root().join("runtime/ts/src").canonicalize().unwrap();
        fs::write(
            &probe,
            script.replace(
                "@seseragi/runtime/",
                &format!("{}/", runtime_prefix.display()),
            ),
        )
        .unwrap();
        let bundle = dist.join("probe.mjs");
        successful(bounded(
            Command::new(runtime("bun", directory))
                .arg("build")
                .arg(&probe)
                .args(["--target", "node", "--outfile"])
                .arg(&bundle),
            directory,
        ));
        for engine in ["bun", "node"] {
            let output = successful(bounded(
                Command::new(runtime(engine, directory))
                    .arg(&bundle)
                    .env("PATH", ""),
                directory,
            ));
            assert!(output.stdout.is_empty(), "{profile}/{engine}: {output:?}");
            assert!(output.stderr.is_empty(), "{profile}/{engine}: {output:?}");
        }
    }
}
