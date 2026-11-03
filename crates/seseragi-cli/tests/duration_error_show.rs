use std::{fs, path::Path, process::Command};

#[path = "support/generic_impl_process.rs"]
mod process;
use process::{bounded, runtime, successful};

fn compare(source: &str, stdout: &str, stderr: &str, exit: i32, directory: &Path) {
    let input = directory.join("main.ssrg");
    fs::write(&input, source).unwrap();
    for profile in ["development", "release"] {
        let native = directory.join(format!("native-{profile}"));
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .args(["experimental", "build-native"])
                .arg(&input)
                .args(["--entry", "main", "--profile", profile, "--out-dir"])
                .arg(&native),
            directory,
        ));
        let native_output = bounded(
            Command::new(native.join(format!("program{}", std::env::consts::EXE_SUFFIX)))
                .current_dir(directory)
                .env("PATH", ""),
            directory,
        );
        assert_eq!(
            native_output.status.code(),
            Some(exit),
            "{profile}: {native_output:?}"
        );
        assert_eq!(native_output.stdout, stdout.as_bytes(), "native {profile}");
        let native_message = if exit == 1 {
            let location = format!("{}:3:55: ", input.file_name().unwrap().to_string_lossy());
            native_output
                .stderr
                .strip_prefix(location.as_bytes())
                .unwrap_or_else(|| panic!("native {profile}: {native_output:?}"))
        } else {
            native_output.stderr.as_slice()
        };
        assert_eq!(native_message, stderr.as_bytes(), "native {profile}");

        let ts = directory.join(format!("ts-{profile}"));
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
                .arg(&ts),
            directory,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(ts.join("artifact-manifest.json")).unwrap()).unwrap();
        for engine in ["node", "bun"] {
            let output = bounded(
                Command::new(runtime(engine, directory))
                    .arg(ts.join(manifest["entry"].as_str().unwrap()))
                    .current_dir(directory)
                    .env("PATH", ""),
                directory,
            );
            assert_eq!(
                output.status.code(),
                native_output.status.code(),
                "{engine} {profile}: {output:?}"
            );
            assert_eq!(output.stdout, native_output.stdout, "{engine} {profile}");
            assert_eq!(output.stderr, native_message, "{engine} {profile}");
        }
    }
    assert_eq!(fs::read_to_string(input).unwrap(), source);
}

#[test]
fn duration_errors_show_matches_ts_and_native_at_int_boundaries() {
    let scratch = tempfile::tempdir().unwrap();
    compare(
        r#"import * as time from "std/time"
pub effect fn main = do {
  println (show (time.NegativeDuration (-9007199254740991)))
  println (show (time.NegativeDuration (-1)))
  println (show (time.NegativeDuration 0))
  println (show (time.NegativeDuration 9007199254740991))
  println (show time.DurationOutsideRange)
}
"#,
        "NegativeDuration -9007199254740991\nNegativeDuration -1\nNegativeDuration 0\nNegativeDuration 9007199254740991\nDurationOutsideRange\n",
        "",
        0,
        scratch.path(),
    );
}

#[test]
fn duration_error_main_failures_match_ts_and_native_at_int_boundaries() {
    let scratch = tempfile::tempdir().unwrap();
    for (expression, expected) in [
        ("time.DurationOutsideRange", "DurationOutsideRange\n"),
        ("(time.NegativeDuration (-1))", "NegativeDuration -1\n"),
        (
            "(time.NegativeDuration (-9007199254740991))",
            "NegativeDuration -9007199254740991\n",
        ),
        ("(time.NegativeDuration 0)", "NegativeDuration 0\n"),
        (
            "(time.NegativeDuration 9007199254740991)",
            "NegativeDuration 9007199254740991\n",
        ),
    ] {
        let source = format!(
            "import * as effects from \"std/effect\"\nimport * as time from \"std/time\"\npub effect fn main -> Unit fails time.DurationError = effects.fail {expression}\n"
        );
        compare(&source, "", expected, 1, scratch.path());
    }
}
