#[allow(dead_code)]
#[path = "support/generic_impl_process.rs"]
mod process;

use process::{bounded, runtime, successful};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_seseragi"))
}

fn package(directory: &Path, source: &str) -> PathBuf {
    let package = directory.join("package");
    fs::create_dir_all(package.join("src")).unwrap();
    fs::write(package.join("src/main.ssrg"), source).unwrap();
    fs::write(package.join("seseragi.toml"), "[package]\nname = \"fixture/entropy\"\nversion = \"0.0.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n[run]\nentry = \"main\"\ntarget = \"process\"\n").unwrap();
    successful(bounded(
        cli().args(["lock", "update"]).arg(&package),
        directory,
    ));
    package
}

fn build_native(source: &Path, directory: &Path) -> PathBuf {
    let output = directory.join("native");
    successful(bounded(
        cli()
            .args(["experimental", "build-native"])
            .arg(source)
            .args(["--entry", "main", "--out-dir"])
            .arg(&output),
        directory,
    ));
    output.join(format!("program{}", std::env::consts::EXE_SUFFIX))
}

fn native_output(binary: &Path, directory: &Path) -> Output {
    bounded(
        Command::new(binary)
            .env("PATH", "")
            .env("SESERAGI_RANDOM_SEED", "invalid unused Random seed")
            .env("SESERAGI_NATIVE_METRICS", "1"),
        directory,
    )
}

fn assert_native(output: Output, expected: &[u8]) {
    let output = successful(output);
    assert_eq!(output.stdout, expected);
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("managed-live=0"),
        "{output:?}"
    );
}

fn strict_typescript(source: &str, profile: &str, directory: &Path) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let generated = seseragi_driver::compile_module(
        seseragi_driver::CompileInput::new("entropy.ssrg", "native/entropy", source).with_profile(
            if profile == "development" {
                seseragi_driver::BuildProfile::Development
            } else {
                seseragi_driver::BuildProfile::Release
            },
        ),
    )
    .unwrap();
    let raw = directory.join("original.ts");
    fs::write(&raw, generated.generated.typescript).unwrap();
    let config = directory.join("tsconfig.json");
    fs::write(
        &config,
        serde_json::to_vec(&serde_json::json!({
            "compilerOptions": {
                "strict": true, "noEmit": true, "skipLibCheck": false,
                "allowImportingTsExtensions": true, "target": "ES2022",
                "module": "ESNext", "moduleResolution": "Bundler",
                "types": ["bun", "node"],
                "typeRoots": [root.join("node_modules/@types")],
                "baseUrl": root,
                "paths": {"@seseragi/runtime/*": ["runtime/ts/src/*"]},
                "lib": ["ES2023", "DOM", "DOM.Iterable"]
            },
            "files": [raw]
        }))
        .unwrap(),
    )
    .unwrap();
    successful(bounded(
        Command::new(runtime("node", directory))
            .arg(root.join("node_modules/typescript/bin/tsc"))
            .arg("--project")
            .arg(config),
        directory,
    ));
}

#[test]
fn entropy_matches_ts_bun_and_native_for_configuration_and_full_size_boundaries() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/entropy.ssrg");
    let expected = include_bytes!("../../../examples/spec/fixtures/rust-backend/entropy.stdout");
    let package = package(directory, source);
    let input = package.join("src/main.ssrg");
    let binary = build_native(&input, directory);
    for profile in ["development", "release"] {
        let output = directory.join(profile);
        successful(bounded(
            cli()
                .arg("build")
                .arg(&package)
                .args(["--target", "process", "--profile", profile, "--out-dir"])
                .arg(&output),
            directory,
        ));
        strict_typescript(source, profile, &output);
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(output.join("artifact-manifest.json")).unwrap())
                .unwrap();
        let result = successful(bounded(
            Command::new(runtime("bun", directory))
                .arg(output.join(manifest["entry"].as_str().unwrap()))
                .env("PATH", ""),
            directory,
        ));
        assert_eq!(&result.stdout, expected, "{profile}");
        assert!(result.stderr.is_empty(), "{result:?}");
        assert_native(native_output(&binary, directory), &result.stdout);
    }
    assert_eq!(fs::read_to_string(input).unwrap(), source);
}

#[test]
fn entropy_imported_opaque_aliases_and_saved_functions_match_ts_and_native() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    fs::create_dir(directory.join("src")).unwrap();
    fs::write(directory.join("seseragi.toml"), "[package]\nname = \"fixture/entropy\"\nversion = \"0.0.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n[run]\nentry = \"main\"\ntarget = \"process\"\n").unwrap();
    fs::write(directory.join("src/api.ssrg"), r#"
import * as entropy from "std/entropy"
import * as bytes from "std/bytes"
pub alias Size = entropy.EntropySize
pub alias Problem = entropy.EntropyError
pub fn configure -> Int -> Either<entropy.EntropyConfigError,Size> = entropy.entropySize
pub fn generate -> Size -> Effect<{entropy:entropy.Entropy},Problem,bytes.Bytes> = entropy.secureBytes
"#).unwrap();
    fs::write(
        directory.join("src/main.ssrg"),
        r#"
import { Size, Problem, configure, generate } from "./api"
import * as entropy from "std/entropy"
import * as effects from "std/effect"
import * as bytes from "std/bytes"
pub effect fn main = do {
  let size = configure ()
  let saved: Size -> Effect<{entropy:entropy.Entropy},Problem,bytes.Bytes> = generate ()
  count <- size 32 |> effects.fromEither |> effects.mapError show
  let cold = saved count
  first <- cold |> effects.mapError show
  second <- cold |> effects.mapError show
  println (show (bytes.length first) + ":" + show (bytes.length second)) |> effects.mapError show
}
"#,
    )
    .unwrap();
    successful(bounded(
        cli().args(["lock", "update"]).arg(directory),
        directory,
    ));
    let ts = successful(bounded(cli().arg("run").arg(directory), directory));
    assert_eq!(ts.stdout, b"32:32\n");
    assert!(ts.stderr.is_empty(), "{ts:?}");
    let binary = build_native(directory, directory);
    assert_native(native_output(&binary, directory), &ts.stdout);
}

#[test]
fn entropy_provider_failures_and_cold_reexecution_match_ts_without_partial_results() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    seseragi_runtime::stage_typescript_package(directory).unwrap();
    let ts = directory.join("probe.ts");
    fs::write(&ts, r#"
import { entropySize, secureBytes } from "@seseragi/runtime/entropy"
import { run } from "@seseragi/runtime/effect"
import { createProviderEntropy } from "@seseragi/runtime/provider-entropy"
import { createEntropyProvider } from "./node_modules/seseragi/entropy.ts"
const size = entropySize(4)
if (size.tag !== "Right") throw new Error("valid size rejected")
for (const mode of ["success", "unavailable", "read-failure"]) {
  let calls = 0
  const entropy = createProviderEntropy({
    provider: "fixture/entropy#entropy", service: "std/entropy::Entropy",
    entry: createEntropyProvider("fixture/entropy#entropy", ["bun-process"], {
      available: () => mode !== "unavailable",
      fill: bytes => {
        calls++
        bytes.set([1, 2, 3, 4])
        if (mode === "read-failure") throw new Error("read failed after partial fill")
      },
    }),
  })
  const plan = secureBytes(size.value)
  console.log(`${mode}:created:${calls}`)
  for (let execution = 0; execution < 2; execution++) {
    const result = await run(plan, { entropy })
    if (result.kind === "success") console.log(`success:${Array.from(result.value).join(",")}:${calls}`)
    else if (result.kind === "failure") console.log(`${result.error.tag}:${calls}`)
    else throw new Error("unexpected entropy outcome")
  }
}
"#).unwrap();
    let expected = b"success:created:0\nsuccess:1,2,3,4:1\nsuccess:1,2,3,4:2\nunavailable:created:0\nEntropyUnavailable:0\nEntropyUnavailable:0\nread-failure:created:0\nEntropyReadFailure:1\nEntropyReadFailure:2\n";
    let bundled = directory.join("probe.mjs");
    successful(bounded(
        Command::new(runtime("bun", directory))
            .arg("build")
            .arg(&ts)
            .args(["--target", "node", "--outfile"])
            .arg(&bundled),
        directory,
    ));
    for engine in ["bun", "node"] {
        let output = successful(bounded(
            Command::new(runtime(engine, directory)).arg(&bundled),
            directory,
        ));
        assert_eq!(output.stdout, expected, "{engine}");
        assert!(output.stderr.is_empty(), "{output:?}");
    }
    let probe = directory.join("probe");
    fs::create_dir(&probe).unwrap();
    fs::write(probe.join("Cargo.toml"), "[package]\nname = \"entropy-probe\"\nversion = \"0.0.0\"\nedition = \"2021\"\n[workspace]\n[[bin]]\nname = \"probe\"\npath = \"probe.rs\"\n").unwrap();
    fs::write(probe.join("probe.rs"), format!("#![allow(dead_code)]\n{}\n{}", seseragi_backend_rust::native_runtime::SOURCE, r#"
thread_local! { static ENTROPY_PROBE_CALLS: Cell<u8> = const { Cell::new(0) }; }
fn entropy_probe(captures: &[V], _: &mut NativeContext) -> NativeOutcome {
    native_entropy_secure_bytes(&captures[1], |bytes| {
        if matches!(captures[0], V::Int(1)) { return Err(NativeEntropyError::Unavailable); }
        ENTROPY_PROBE_CALLS.with(|calls| calls.set(calls.get() + 1));
        bytes.copy_from_slice(&[1, 2, 3, 4]);
        if matches!(captures[0], V::Int(2)) { Err(NativeEntropyError::ReadFailure) } else { Ok(()) }
    })
}

fn main() {
    for (mode, label) in [(0, "success"), (1, "unavailable"), (2, "read-failure")] {
        ENTROPY_PROBE_CALLS.with(|calls| calls.set(0));
        let size = native_entropy_operation("std/entropy::entropySize", vec![V::Int(4)]).payload();
        let plan = V::leaf(vec![V::Int(mode), size], entropy_probe);
        println!("{label}:created:{}", ENTROPY_PROBE_CALLS.with(Cell::get));
        let mut context = NativeContext::default();
        for _ in 0..2 {
            match context.root(plan.clone()) {
                Ok(bytes) => println!("success:{}:{}", native_bytes(&bytes).iter().map(u8::to_string).collect::<Vec<_>>().join(","), ENTROPY_PROBE_CALLS.with(Cell::get)),
                Err(NativeExit::Failure(error)) => {
                    let V::Adt(value) = &error.value else { panic!() };
                    println!("{}:{}", value.get().0.rsplit("::").next().unwrap(), ENTROPY_PROBE_CALLS.with(Cell::get));
                }
                _ => panic!("unexpected entropy outcome"),
            }
        }
        drop(plan);
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
"#)).unwrap();
    let output = successful(bounded(
        Command::new("cargo")
            .args([
                "build",
                "--offline",
                "--message-format=json-render-diagnostics",
                "--manifest-path",
            ])
            .arg(probe.join("Cargo.toml")),
        directory,
    ));
    let executable = output
        .stdout
        .split(|byte| *byte == b'\n')
        .filter_map(|line| serde_json::from_slice::<serde_json::Value>(line).ok())
        .find(|message| {
            message["reason"] == "compiler-artifact" && message["executable"].is_string()
        })
        .unwrap();
    let output = successful(bounded(
        Command::new(executable["executable"].as_str().unwrap()).env("PATH", ""),
        directory,
    ));
    assert_eq!(output.stdout, expected);
    assert!(output.stderr.is_empty(), "{output:?}");
}

#[test]
fn native_entropy_rejects_pinned_providers_without_publishing_outputs() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    fs::create_dir(directory.join("src")).unwrap();
    fs::write(
        directory.join("src/main.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/entropy.ssrg"),
    )
    .unwrap();
    fs::write(directory.join("seseragi.toml"), "[package]\nname = \"fixture/entropy\"\nversion = \"0.0.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n[run]\nentry = \"main\"\ntarget = \"process\"\n[providers]\n\"std/entropy::Entropy\" = \"custom/entropy#entropy\"\n").unwrap();
    successful(bounded(
        cli().args(["lock", "update"]).arg(directory),
        directory,
    ));
    let output = directory.join("native");
    let result = bounded(
        cli()
            .args(["experimental", "build-native"])
            .arg(directory)
            .args(["--entry", "main", "--out-dir"])
            .arg(&output),
        directory,
    );
    assert_eq!(result.status.code(), Some(2), "{result:?}");
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("unsupported native Entropy provider"),
        "{result:?}"
    );
    assert!(!output.exists());
}

#[test]
fn entropy_configuration_failures_and_argument_defects_release_scoped_resources() {
    for (body, expected, code) in [
        (
            "let cold = entropy.entropySize 0 |> effects.fromEither |> effects.mapError show; say \"created\"; cold; effects.succeed ()",
            "acquired\ncreated\nreleased\nNonPositiveEntropySize 0\n",
            0,
        ),
        (
            "size <- entropy.entropySize 4 |> effects.fromEither |> effects.mapError show; entropy.secureBytes (broken size) |> effects.mapError show; effects.succeed ()",
            "acquired\nreleased\n",
            70,
        ),
    ] {
        let scratch = tempfile::tempdir().unwrap();
        let directory = scratch.path();
        let text = format!(r#"
import * as entropy from "std/entropy"
import {{ Entropy }} from "std/entropy"
import * as effects from "std/effect"
fn broken size:entropy.EntropySize -> entropy.EntropySize = {{ let value = 1 / 0; size }}
effect fn ignore _error:ConsoleError -> Unit with Console, Entropy = effects.succeed ()
effect fn say text:String -> Unit with Console, Entropy = println text |> effects.recover ignore
pub effect fn main = effects.scoped (do {{
  effects.acquireRelease (say "acquired") (\_:Unit -> say "released")
  {body}
}}) |> effects.recover (\message:String -> say message)
"#);
        let package = package(directory, &text);
        let binary = build_native(&package, directory);
        let mut command = cli();
        command.arg("run").arg(&package);
        if code == 70 {
            command.args(["--diagnostic-format", "json"]);
        }
        let ts = bounded(&mut command, directory);
        let native = native_output(&binary, directory);
        for (is_native, result) in [(false, ts), (true, native)] {
            assert_eq!(result.status.code(), Some(code), "{is_native}: {result:?}");
            assert_eq!(result.stdout, expected.as_bytes());
            let stderr = String::from_utf8_lossy(&result.stderr);
            if code == 70 {
                assert!(stderr.contains("Int division by zero"), "{stderr}");
            } else if !is_native {
                assert!(stderr.is_empty(), "{stderr}");
            }
            if is_native {
                assert!(stderr.contains("managed-live=0"), "{stderr}");
            }
        }
    }
}
