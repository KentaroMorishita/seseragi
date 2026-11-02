#[allow(dead_code)]
#[path = "support/generic_impl_process.rs"]
mod process;

use process::{bounded, runtime, successful};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const SOURCE: &str = include_str!("../../../examples/spec/fixtures/rust-backend/random.ssrg");
const EXPECTED: &[u8] =
    include_bytes!("../../../examples/spec/fixtures/rust-backend/random.stdout");

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_seseragi"))
}

fn package(directory: &Path, source: &str) -> PathBuf {
    let package = directory.join("package");
    fs::create_dir_all(package.join("src")).unwrap();
    fs::write(package.join("src/main.ssrg"), source).unwrap();
    fs::write(
        package.join("seseragi.toml"),
        include_str!("../../../examples/spec/fixtures/projects/random-seed/seseragi.toml"),
    )
    .unwrap();
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

fn native(binary: &Path, seed: i64, directory: &Path, expected: &[u8]) {
    let output = successful(bounded(
        Command::new(binary)
            .env("PATH", "")
            .env("SESERAGI_RANDOM_SEED", seed.to_string())
            .env("SESERAGI_NATIVE_METRICS", "1"),
        directory,
    ));
    assert_eq!(output.stdout, expected, "seed {seed}");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("managed-live=0"),
        "{output:?}"
    );
}

#[test]
fn random_matches_ts_bun_node_and_native_for_errors_and_boundaries() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    let package = package(directory, SOURCE);
    let binary = build_native(&package.join("src/main.ssrg"), directory);
    for profile in ["development", "release"] {
        let dist = directory.join(profile);
        successful(bounded(
            cli()
                .arg("build")
                .arg(&package)
                .args(["--profile", profile, "--out-dir"])
                .arg(&dist),
            directory,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(dist.join("artifact-manifest.json")).unwrap())
                .unwrap();
        for engine in ["bun", "node"] {
            let output = successful(bounded(
                Command::new(runtime(engine, directory))
                    .arg(dist.join(manifest["entry"].as_str().unwrap()))
                    .env("PATH", ""),
                directory,
            ));
            assert_eq!(output.stdout, EXPECTED, "{profile}/{engine}");
            assert!(output.stderr.is_empty(), "{profile}/{engine}: {output:?}");
        }
    }
    for seed in [42, 0, -1, 17, i64::MIN, i64::MAX] {
        let ts = successful(bounded(
            cli()
                .arg("run")
                .arg(&package)
                .arg("--random-seed")
                .arg(seed.to_string()),
            directory,
        ));
        assert!(ts.stderr.is_empty(), "{ts:?}");
        if seed == 42 {
            assert_eq!(ts.stdout, EXPECTED);
        }
        native(&binary, seed, directory, &ts.stdout);
        native(&binary, seed, directory, &ts.stdout);
    }
}

#[test]
fn native_random_seed_overrides_manifest_and_inherited_environment_for_one_run() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    let package = package(
        directory,
        include_str!("../../../examples/spec/fixtures/projects/random-seed/src/main.ssrg"),
    );
    let original = fs::read(package.join("seseragi.toml")).unwrap();
    let source = package.join("src/main.ssrg");
    for input in [&package, &source] {
        for seed in [None, Some("17"), Some("-9223372036854775808"), None] {
            let mut process = cli();
            process.arg("run").arg(&package);
            let mut native = cli();
            native
                .arg("run")
                .arg(input)
                .args(["--target", "native"])
                .env("SESERAGI_RANDOM_SEED", "999");
            if let Some(seed) = seed {
                process.args(["--random-seed", seed]);
                native.args(["--random-seed", seed]);
            }
            let expected = successful(bounded(&mut process, directory));
            let actual = successful(bounded(&mut native, directory));
            assert_eq!(actual.stdout, expected.stdout);
            assert!(actual.stderr.is_empty(), "{actual:?}");
        }
    }
    assert_eq!(fs::read(package.join("seseragi.toml")).unwrap(), original);
    for seed in ["entropy", "9223372036854775808", "bad"] {
        let output = bounded(
            cli()
                .arg("run")
                .arg(&package)
                .env("SESERAGI_RANDOM_SEED", "invalid inherited seed")
                .args(["--target", "native", "--random-seed", seed]),
            directory,
        );
        if seed == "entropy" {
            successful(output);
        } else {
            assert_eq!(output.status.code(), Some(2));
            assert!(String::from_utf8_lossy(&output.stderr).contains("signed 64-bit integer"));
        }
    }
    let standalone = directory.join("standalone.ssrg");
    fs::write(
        &standalone,
        include_str!("../../../examples/spec/fixtures/projects/random-seed/src/main.ssrg"),
    )
    .unwrap();
    successful(bounded(
        cli()
            .arg("run")
            .arg(standalone)
            .args(["--target", "native"])
            .env("SESERAGI_RANDOM_SEED", "invalid inherited seed"),
        directory,
    ));
}

#[test]
fn random_rejection_sampling_matches_ts_and_advances_past_the_rejected_word() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    let package = package(
        directory,
        r#"
import * as random from "std/random"
import * as effects from "std/effect"
pub effect fn main = do {
  value <- random.intBetween (-2) 9007199254740991 |> effects.mapError show
  next <- random.nextInt ()
  println (show value) |> effects.mapError show
  println (show next) |> effects.mapError show
}
"#,
    );
    let ts = successful(bounded(
        cli()
            .arg("run")
            .arg(&package)
            .args(["--random-seed", "3071"]),
        directory,
    ));
    assert_eq!(ts.stdout, b"5391084846646747\n-2786565656275751\n");
    assert!(ts.stderr.is_empty(), "{ts:?}");
    let binary = build_native(&package.join("src/main.ssrg"), directory);
    native(&binary, 3071, directory, &ts.stdout);
}

#[test]
fn random_cold_failures_and_argument_defects_keep_scope_cleanup_and_source_order() {
    for (body, expected, code) in [
        (
            "let cold = random.intBetween 3 3; say \"created\"; cold |> effects.mapError show",
            "acquired\ncreated\nreleased\nEmptyRandomIntRange { lower: 3, upperExclusive: 3 }\n",
            0,
        ),
        (
            "random.intBetween (explode ()) 10 |> effects.mapError show",
            "acquired\nreleased\n",
            70,
        ),
    ] {
        let scratch = tempfile::tempdir().unwrap();
        let directory = scratch.path();
        let source = format!(
            r#"
import * as random from "std/random"
import * as effects from "std/effect"
fn explode -> Int = 1 / 0
effect fn ignore _error:ConsoleError -> Unit with Console, Random = effects.succeed ()
effect fn say text:String -> Unit with Console, Random = println text |> effects.recover ignore
pub effect fn main = effects.scoped (do {{
  effects.acquireRelease (say "acquired") (\_:Unit -> say "released")
  {body}
  effects.succeed ()
}}) |> effects.recover (\message:String -> say message)
"#
        );
        let package = package(directory, &source);
        let binary = build_native(&package.join("src/main.ssrg"), directory);
        for native in [false, true] {
            let mut command = if native { Command::new(&binary) } else { cli() };
            if native {
                command.env("PATH", "").env("SESERAGI_NATIVE_METRICS", "1");
            } else {
                command
                    .arg("run")
                    .arg(&package)
                    .args(["--random-seed", "42"]);
                if code == 70 {
                    command.args(["--diagnostic-format", "json"]);
                }
            }
            let output = bounded(&mut command, directory);
            assert_eq!(output.status.code(), Some(code), "{native}: {output:?}");
            assert_eq!(output.stdout, expected.as_bytes(), "{native}");
            let stderr = String::from_utf8_lossy(&output.stderr);
            if code == 70 {
                assert!(stderr.contains("Int division by zero"), "{stderr}");
            }
            if native {
                assert!(stderr.contains("managed-live=0"), "{stderr}");
            } else if code == 0 {
                assert!(stderr.is_empty(), "{stderr}");
            }
        }
    }
}

#[test]
fn native_random_rejects_package_provider_pins_without_publishing_outputs() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    let package = package(
        directory,
        include_str!("../../../examples/spec/fixtures/projects/random-seed/src/main.ssrg"),
    );
    let manifest = package.join("seseragi.toml");
    let mut source = fs::read_to_string(&manifest).unwrap();
    source.push_str("\n[providers]\n\"std/random::Random\" = \"custom/random#random\"\n");
    fs::write(manifest, source).unwrap();
    successful(bounded(
        cli().args(["lock", "update"]).arg(&package),
        directory,
    ));
    let output = directory.join("native");
    let result = bounded(
        cli()
            .args(["experimental", "build-native"])
            .arg(&package)
            .args(["--entry", "main", "--out-dir"])
            .arg(&output),
        directory,
    );
    assert_eq!(result.status.code(), Some(2), "{result:?}");
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("unsupported native Random provider"),
        "{result:?}"
    );
    assert!(!output.exists());
}
