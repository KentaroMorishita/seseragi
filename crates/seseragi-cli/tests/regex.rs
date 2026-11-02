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
    fs::write(package.join("seseragi.toml"), "[package]\nname = \"fixture/regex\"\nversion = \"0.0.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n[run]\nentry = \"main\"\ntarget = \"process\"\n").unwrap();
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
        seseragi_driver::CompileInput::new("regex.ssrg", "native/regex", source).with_profile(
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
fn regex_matches_strict_ts_bun_node_and_native_for_syntax_captures_and_boundaries() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/regex-basic.ssrg");
    let package = package(directory, source);
    let input = package.join("src/main.ssrg");
    let binary = build_native(&input, directory);
    let native = successful(native_output(&binary, directory));
    assert!(
        String::from_utf8_lossy(&native.stderr).contains("managed-live=0"),
        "{native:?}"
    );
    let mut expected = None;
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
        for engine in ["bun", "node"] {
            let result = successful(bounded(
                Command::new(runtime(engine, directory))
                    .arg(output.join(manifest["entry"].as_str().unwrap()))
                    .env("PATH", ""),
                directory,
            ));
            assert!(result.stderr.is_empty(), "{profile}/{engine}: {result:?}");
            if let Some(expected) = &expected {
                assert_eq!(&result.stdout, expected, "{profile}/{engine}");
            } else {
                let text = String::from_utf8_lossy(&result.stdout);
                assert!(text.starts_with("False:False:False\n"), "{text}");
                assert!(text.contains("DuplicateCaptureName"), "{text}");
                assert!(text.contains("InvalidRegexRange"), "{text}");
                assert!(text.contains("5..12"), "{text}");
                assert!(text.contains("Nothing"), "{text}");
                expected = Some(result.stdout.clone());
            }
            assert_eq!(native.stdout, result.stdout, "{profile}/{engine}");
        }
    }
    assert_eq!(fs::read_to_string(input).unwrap(), source);
}

#[test]
fn regex_imported_opaque_aliases_and_saved_functions_match_ts_and_native() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    let source = r#"
import { Pattern, Problem, Hit, defaults, compile, configured, matches, search, literal, every, parts, replace, replaceWith } from "./api"
import * as regex from "std/regex"
import * as effects from "std/effect"
import * as text from "std/text"
fn keep<A> value:A -> A = value
pub effect fn main = do {
  let options = defaults () ()
  let make = keep (configured () options)
  let escape = keep (literal ())
  let plain = keep (compile ())
  let left = plain "("
  match left {
    Left error -> println (show error) |> effects.mapError show
    Right _ -> println "unexpected success" |> effects.mapError show
  }
  match make (escape "雫.🙂") {
    Left error -> println (show error) |> effects.mapError show
    Right pattern -> do {
      let test = keep (matches () pattern)
      let find = keep (search () pattern)
      println (show (test "x雫.🙂") + ":" + show (test "x雫a🙂")) |> effects.mapError show
      match find "x雫.🙂" {
        Nothing -> println "missing match" |> effects.mapError show
        Just hit -> println (show hit.span.start + ":" + show hit.span.end + ":" + hit.text) |> effects.mapError show
      }
      let all = keep (every () pattern)
      let split = keep (parts () pattern)
      let replacement = keep (replace () pattern)
      let replaceLiteral = keep (replacement "$1")
      let callback = keep (replaceWith () pattern)
      let replaceCallback = keep (callback (\hit:Hit -> show hit.span.start + ":" + hit.text))
      println (text.join "|" (map (\hit:Hit -> hit.text) (all "雫.🙂x雫.🙂"))) |> effects.mapError show
      println (text.join ":" (split "雫.🙂x雫.🙂")) |> effects.mapError show
      println (replaceLiteral "雫.🙂x雫.🙂") |> effects.mapError show
      println (replaceCallback "雫.🙂x雫.🙂") |> effects.mapError show
    }
  }
}
"#;
    let package_root = directory.join("package");
    fs::create_dir_all(package_root.join("src")).unwrap();
    fs::write(
        package_root.join("src/api.ssrg"),
        r#"
import * as regex from "std/regex"
pub alias Pattern = regex.Regex
pub alias Problem = regex.RegexCompileError
pub alias Options = regex.RegexOptions
pub alias Hit = regex.RegexMatch
pub fn defaults -> Unit -> Options = regex.defaultOptions
pub fn compile -> String -> Either<Problem,Pattern> = regex.compile
pub fn configured -> Options -> String -> Either<Problem,Pattern> = regex.compileWith
pub fn matches -> Pattern -> String -> Bool = regex.isMatch
pub fn search -> Pattern -> String -> Maybe<Hit> = regex.find
pub fn literal -> String -> String = regex.escape
pub fn every -> Pattern -> String -> Array<Hit> = regex.findAll
pub fn parts -> Pattern -> String -> Array<String> = regex.split
pub fn replace -> Pattern -> String -> String -> String = regex.replaceAll
pub fn replaceWith -> Pattern -> (Hit -> String) -> String -> String = regex.replaceAllWith
"#,
    )
    .unwrap();
    let package = package(directory, source);
    let binary = build_native(&package, directory);
    let expected = "RegexCompileError { kind: UnexpectedRegexEnd, offset: 1 }\nTrue:False\n1:9:雫.🙂\n雫.🙂|雫.🙂\n:x:\n$1x$1\n0:雫.🙂x9:雫.🙂\n".as_bytes();
    assert_native(native_output(&binary, directory), expected);
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
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(output.join("artifact-manifest.json")).unwrap())
                .unwrap();
        for engine in ["bun", "node"] {
            let result = successful(bounded(
                Command::new(runtime(engine, directory))
                    .arg(output.join(manifest["entry"].as_str().unwrap()))
                    .env("PATH", ""),
                directory,
            ));
            assert_eq!(result.stdout, expected, "{profile}/{engine}");
            assert!(result.stderr.is_empty(), "{result:?}");
        }
    }
}

#[test]
fn regex_collections_match_strict_ts_bun_node_and_native() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/regex-collections.ssrg");
    let package = package(directory, source);
    let input = package.join("src/main.ssrg");
    let binary = build_native(&input, directory);
    let native = successful(native_output(&binary, directory));
    assert!(
        String::from_utf8_lossy(&native.stderr).contains("managed-live=0"),
        "{native:?}"
    );
    let text = String::from_utf8_lossy(&native.stdout);
    assert_eq!(text.lines().count(), 47, "{text}");
    for expected in [
        "RegexCompileError { kind: UnexpectedRegexEnd, offset: 3 }",
        "RegexCompileError { kind: DuplicateCaptureName 名前, offset: 15 }",
        "RegexCompileError { kind: InvalidRegexRange, offset: 2 }",
        "RegexCompileError { kind: InvalidRegexQuantifier, offset: 1 }",
        "RegexCompileError { kind: InvalidRegexEscape, offset: 0 }",
        "RegexCompileError { kind: UnsupportedRegexFeature backreference, offset: 3 }",
        "RegexCompileError { kind: UnsupportedRegexFeature lazy quantifier, offset: 2 }",
    ] {
        assert!(text.contains(expected), "missing {expected}: {text}");
    }
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
        for engine in ["bun", "node"] {
            let result = successful(bounded(
                Command::new(runtime(engine, directory))
                    .arg(output.join(manifest["entry"].as_str().unwrap()))
                    .env("PATH", ""),
                directory,
            ));
            assert!(result.stderr.is_empty(), "{profile}/{engine}: {result:?}");
            assert_eq!(native.stdout, result.stdout, "{profile}/{engine}");
        }
    }
    assert_eq!(fs::read_to_string(input).unwrap(), source);
}

#[test]
fn regex_replacement_effects_keep_cold_execution_error_kinds_and_finalizers() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    for (body, expected, error) in [
        (
            r#"say (regex.replaceAllWith pattern render "none")"#,
            "acquired\nnone\ncontinued\nreleased\n",
            None,
        ),
        (
            r#"let cold = effects.defer (\_:Unit -> say (regex.replaceAllWith pattern render "a1b3"))
        say "created"
        cold
        cold"#,
            "acquired\ncreated\na[1]b[3]\na[1]b[3]\ncontinued\nreleased\n",
            None,
        ),
        (
            r#"say (regex.replaceAllWith pattern render "a1🙂2b3")"#,
            "acquired\nreleased\n",
            Some(("Defect", "Seseragi Int division by zero")),
        ),
        (
            r#"say (regex.replaceAllWith pattern render (show (42 / 0)))"#,
            "acquired\nreleased\n",
            Some(("Defect", "Seseragi Int division by zero")),
        ),
        (
            r#"say (regex.replaceAllWith pattern (if 42 / 0 == 0 then render else render) "none")"#,
            "acquired\nreleased\n",
            Some(("Defect", "Seseragi Int division by zero")),
        ),
        (
            r#"say (regex.replaceAllWith pattern render "a1b3")
        effects.fail "typed""#,
            "acquired\na[1]b[3]\nreleased\n",
            Some(("TypedFailure", "typed")),
        ),
    ] {
        let source = format!(
            r#"
import * as regex from "std/regex"
import * as effects from "std/effect"
effect fn ignore _error:ConsoleError -> Unit with Console = effects.succeed ()
effect fn say value:String -> Unit with Console = println value |> effects.recover ignore
fn render hit:regex.RegexMatch -> String = if hit.span.start == 6 then show (42 / 0) else "[" + hit.text + "]"
pub effect fn main = do {{
  effects.acquireRelease (say "acquired") (\_:Unit -> say "released")
  match regex.compile "(?<digit>[0-9])" {{
    Left error -> say (show error)
    Right pattern -> do {{
      {body}
      say "continued"
    }}
  }}
}}
"#
        );
        let package = package(directory, &source);
        let ts = bounded(
            cli()
                .arg("run")
                .arg(&package)
                .args(["--diagnostic-format", "json"]),
            directory,
        );
        let exit = match error {
            Some(("Defect", _)) => 70,
            Some(("TypedFailure", _)) => 1,
            None => 0,
            _ => panic!(),
        };
        assert_eq!(ts.status.code(), Some(exit), "{body}: {ts:?}");
        assert_eq!(ts.stdout, expected.as_bytes(), "{body}");
        if let Some((kind, message)) = error {
            let diagnostic: serde_json::Value = serde_json::from_slice(&ts.stderr).unwrap();
            assert_eq!(diagnostic["kind"], kind);
            assert_eq!(diagnostic["message"], message);
        } else {
            assert!(ts.stderr.is_empty(), "{ts:?}");
        }
        let binary = build_native(&package, directory);
        let native = native_output(&binary, directory);
        assert_eq!(native.status.code(), ts.status.code(), "{body}: {native:?}");
        assert_eq!(native.stdout, ts.stdout, "{body}");
        let stderr = String::from_utf8(native.stderr).unwrap();
        let mut lines = stderr.lines();
        if let Some((kind, message)) = error {
            let diagnostic = lines.next().unwrap();
            let message = if kind == "Defect" {
                format!("Seseragi Defect: {message}")
            } else {
                message.to_owned()
            };
            assert!(
                diagnostic.ends_with(&format!(": {message}")),
                "{diagnostic}"
            );
        }
        assert!(
            lines.next().unwrap().starts_with("native managed-live=0 "),
            "{stderr}"
        );
        assert_eq!(lines.next(), None, "{stderr}");
    }
}
