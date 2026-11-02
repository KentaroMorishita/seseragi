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
    fs::write(package.join("seseragi.toml"), "[package]\nname = \"fixture/namespace-structs\"\nversion = \"0.0.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n[run]\nentry = \"main\"\ntarget = \"process\"\n").unwrap();
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

fn assert_struct_outputs(input: &Path, directory: &Path, expected: &[u8]) {
    successful(bounded(cli().arg("lint").arg(input), directory));
    for profile in ["development", "release"] {
        let output = directory.join(profile);
        successful(bounded(
            cli()
                .arg("build")
                .arg(input)
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
    let binary = build_native(input, directory);
    assert_native(native_output(&binary, directory), expected);
}

#[test]
fn public_regex_structs_construct_from_named_and_namespace_imports() {
    for qualifier in ["", "regex."] {
        let named = if qualifier.is_empty() {
            "import { RegexSpan, RegexOptions, RegexCapture } from \"std/regex\""
        } else {
            ""
        };
        let source = format!(
            r#"
{named}
import * as regex from "std/regex"
import * as text from "std/text"
fn render span:{qualifier}RegexSpan -> String = show span.start + ":" + show span.end
pub effect fn main = do {{
  let start = 0
  let end = 1
  let span = {qualifier}RegexSpan {{ start, end }}
  let empty = {qualifier}RegexSpan {{ start: 0, end: 0 }}
  let limits = {qualifier}RegexSpan {{ start: -9007199254740991, end: 9007199254740991 }}
  let reversed = {qualifier}RegexSpan {{ start: 1, end: 0 }}
  let updated = {qualifier}RegexSpan {{ ...span, end: 2 }}
  let capture = {qualifier}RegexCapture {{ span: {qualifier}RegexSpan {{ start: 0, end: 2 }}, text: "雪🙂" }}
  let options = {qualifier}RegexOptions {{ caseInsensitive: True, multiline: False, dotMatchesNewline: False }}
  println (render span)
  println (render empty)
  println (render limits)
  println (render reversed)
  println (render updated)
  println (capture.text + ":" + render capture.span)
  println (show options.caseInsensitive + ":" + show options.multiline + ":" + show options.dotMatchesNewline)
  match regex.compileWith options "A" {{
    Left error -> println (show error)
    Right pattern -> println (show (regex.isMatch pattern "a"))
  }}
  println (show (text.InvalidScalarRange {{ start: 0, end: 1, length: 2 }}))
}}
"#
        );
        let expected = concat!(
            "0:1\n0:0\n-9007199254740991:9007199254740991\n1:0\n0:2\n",
            "雪🙂:0:2\nTrue:False:False\nTrue\n",
            "InvalidScalarRange { start: 0, end: 1, length: 2 }\n",
        )
        .as_bytes();
        let scratch = tempfile::tempdir().unwrap();
        let directory = scratch.path();
        let input = directory.join("main.ssrg");
        fs::write(&input, &source).unwrap();
        assert_struct_outputs(&input, directory, expected);
        assert_eq!(fs::read_to_string(&input).unwrap(), source);
    }
}

#[test]
fn public_generic_structs_and_record_payload_constructors_keep_their_meaning() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    fs::create_dir_all(directory.join("package/src")).unwrap();
    fs::write(
        directory.join("package/src/domain.ssrg"),
        concat!(
            "pub struct Box<A> { value: A }\n",
            "pub struct Empty {}\n",
            "pub alias Wrapped<A> = Box<A>\n",
            "pub type Event = | Event { value: Int }\n",
            "pub type Number = | Number Int\n",
        ),
    )
    .unwrap();
    let source = r#"
import { Box as Named } from "./domain"
import * as domain from "./domain"
fn read box:Named<Int> -> Int = box.value
fn empty value:domain.Empty -> String = "empty"
fn apply make:({ value: Int } -> domain.Event) -> payload:{ value: Int } -> domain.Event = make payload
pub effect fn main = do {
  let value = 42
  let named = Named { value }
  let qualified = domain.Box<Int> { value }
  let wrapped = domain.Wrapped<Int> { ...qualified, value: 7 }
  println (show named.value + ":" + show qualified.value + ":" + show wrapped.value)
  println (show (read domain.Box<Int> { value }))
  println (show domain.Box<Int> { value: 42 }.value)
  println (empty domain.Empty {})
  let number = domain.Number { value }
  println (match number { domain.Number item -> show item })
  let event = apply domain.Event { value: 42 }
  println (match event { domain.Event payload -> show payload.value })
}
"#;
    let package = package(directory, source);
    assert_struct_outputs(&package, directory, b"42:42:7\n42\n42\nempty\n42\n42\n");
    assert_eq!(
        fs::read_to_string(package.join("src/main.ssrg")).unwrap(),
        source
    );
}

fn assert_rejected_construction(input: &Path, directory: &Path, code: &str, key: &str) {
    let lint = bounded(
        cli()
            .arg("lint")
            .arg(input)
            .args(["--diagnostic-format", "json"]),
        directory,
    );
    assert_eq!(lint.status.code(), Some(2), "{lint:?}");
    let diagnostics: serde_json::Value = serde_json::from_slice(&lint.stderr).unwrap();
    let errors = diagnostics["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|document| document["diagnostics"]["diagnostics"].as_array().unwrap())
        .filter(|diagnostic| diagnostic["severity"] == "Error")
        .map(|diagnostic| {
            (
                diagnostic["code"].as_str().unwrap(),
                diagnostic["messageKey"].as_str().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(errors, [(code, key)], "{diagnostics:#}");
    let output = directory.join("rejected");
    fs::create_dir_all(&output).unwrap();
    let executable = format!("program{}", std::env::consts::EXE_SUFFIX);
    let previous = ["program.rs", executable.as_str(), "artifact-manifest.json"];
    for name in previous {
        fs::write(output.join(name), b"previous output").unwrap();
    }
    for profile in ["development", "release"] {
        let ts = bounded(
            cli()
                .arg("build")
                .arg(input)
                .args(["--target", "process", "--profile", profile, "--out-dir"])
                .arg(&output),
            directory,
        );
        let native = bounded(
            cli()
                .args(["experimental", "build-native"])
                .arg(input)
                .args(["--entry", "main", "--profile", profile, "--out-dir"])
                .arg(&output),
            directory,
        );
        for result in [&ts, &native] {
            assert_eq!(result.status.code(), Some(2), "{profile}: {result:?}");
            assert!(result.stdout.is_empty(), "{result:?}");
            assert!(
                String::from_utf8_lossy(&result.stderr).contains(code),
                "{result:?}"
            );
        }
        assert_eq!(
            native.stderr,
            [b"seseragi: ".as_slice(), ts.stderr.as_slice(), b"\n"].concat(),
            "{profile}"
        );
        for name in previous {
            assert_eq!(fs::read(output.join(name)).unwrap(), b"previous output");
        }
    }
}

#[test]
fn regex_struct_errors_match_ts_and_native_without_replacing_outputs() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    let input = directory.join("main.ssrg");
    for (name, expression, code, key) in [
        (
            "RegexSpan",
            "RegexSpan { start: 0 }",
            "SES-T0101",
            "struct.field-missing",
        ),
        (
            "RegexSpan",
            "RegexSpan { start: \"zero\", end: 1 }",
            "SES-T0101",
            "struct.field-type-mismatch",
        ),
        (
            "RegexSpan",
            "RegexSpan { start: 0, end: 1, extra: 2 }",
            "SES-T0101",
            "struct.field-unresolved",
        ),
        (
            "RegexSpan",
            "RegexSpan { start: 0, start: 1, end: 1 }",
            "SES-T0101",
            "record.duplicate-field",
        ),
        (
            "RegexSpan",
            "RegexSpan<Int> { start: 0, end: 1 }",
            "SES-T0101",
            "struct.type-argument-arity-mismatch",
        ),
        (
            "RegexSpan",
            "RegexSpan { ...{ start: 0, end: 1 } }",
            "SES-T0101",
            "struct.spread-type-mismatch",
        ),
        (
            "RegexOptions",
            "RegexOptions { caseInsensitive: False }",
            "SES-T0101",
            "struct.field-missing",
        ),
        (
            "RegexOptions",
            "RegexOptions { caseInsensitive: 0, multiline: False, dotMatchesNewline: False }",
            "SES-T0101",
            "struct.field-type-mismatch",
        ),
        (
            "Regex",
            "Regex {}",
            "SES-T0101",
            "struct.construction-on-non-struct",
        ),
        (
            "MissingRegexType",
            "MissingRegexType {}",
            "SES-N0104",
            "module.export-unresolved",
        ),
    ] {
        for qualifier in ["", "regex."] {
            let import = if qualifier.is_empty() {
                format!("import {{ {name} }} from \"std/regex\"")
            } else {
                "import * as regex from \"std/regex\"".to_owned()
            };
            let source = format!("{import}\nlet invalid = {qualifier}{expression}\npub effect fn main = println \"unreachable\"\n");
            fs::write(&input, &source).unwrap();
            assert_rejected_construction(&input, directory, code, key);
            assert_eq!(fs::read_to_string(&input).unwrap(), source);
        }
    }
    for (import, name) in [
        ("import { RegexSpan } from \"std/regex\"", "RegexSpan"),
        ("import * as regex from \"std/regex\"", "regex.RegexSpan"),
    ] {
        fs::write(&input, format!("{import}\nlet invalid: {name} = {{ start: 0, end: 1 }}\npub effect fn main = println \"unreachable\"\n")).unwrap();
        assert_rejected_construction(
            &input,
            directory,
            "SES-T0101",
            "binding.annotation-type-mismatch",
        );
    }
}

#[test]
fn private_namespace_structs_remain_rejected_by_ts_and_native() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    fs::create_dir_all(directory.join("package/src")).unwrap();
    fs::write(
        directory.join("package/src/domain.ssrg"),
        "struct Hidden { value: Int }\npub opaque struct Secret { value: Int }\n",
    )
    .unwrap();
    let package = package(directory, "pub effect fn main = println \"unreachable\"\n");
    let input = package.join("src/main.ssrg");
    for (name, code, key) in [
        ("Hidden", "SES-N0102", "module.private-symbol"),
        ("Secret", "SES-T0101", "struct.representation-private"),
        ("Missing", "SES-N0104", "module.export-unresolved"),
    ] {
        fs::write(&input, format!("import * as domain from \"./domain\"\nlet invalid = domain.{name} {{ value: 1 }}\npub effect fn main = println \"unreachable\"\n")).unwrap();
        successful(bounded(
            cli().args(["lock", "update"]).arg(&package),
            directory,
        ));
        assert_rejected_construction(&package, directory, code, key);
    }
}
