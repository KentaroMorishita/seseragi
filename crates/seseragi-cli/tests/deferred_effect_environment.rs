use seseragi_driver::{BuildProfile, CompileInput};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[path = "support/generic_impl_process.rs"]
mod process;
use process::{bounded, runtime, successful};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

fn check(source: &str, expected: &str, code: i32) {
    let scratch = tempfile::Builder::new()
        .prefix("seseragi-deferred-environment-")
        .tempdir()
        .unwrap();
    let directory = scratch.path();
    let source_path = directory.join("main.ssrg");
    fs::write(&source_path, source).unwrap();
    let native = directory.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(&source_path)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native),
        directory,
    ));
    let output = bounded(
        Command::new(native.join(format!("program{}", std::env::consts::EXE_SUFFIX)))
            .current_dir(directory)
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1"),
        directory,
    );
    assert_eq!(output.status.code(), Some(code), "{output:?}");
    assert_eq!(output.stdout, expected.as_bytes());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("managed-live=0"), "{stderr}");
    if code == 70 {
        assert!(stderr.contains("body defect"), "{stderr}");
    }

    for (label, profile) in [
        ("development", BuildProfile::Development),
        ("release", BuildProfile::Release),
    ] {
        let destination = directory.join(label);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(&source_path)
                .args(["--target", "process", "--profile", label, "--out-dir"])
                .arg(&destination),
            directory,
        ));
        let compiled = seseragi_driver::compile_module(
            CompileInput::new("main.ssrg", "fixture/deferred-environment", source)
                .with_profile(profile),
        )
        .unwrap();
        // Check the driver's original emitted bytes as well as the process entry.
        let raw = destination.join("original.ts");
        fs::write(&raw, compiled.generated.typescript).unwrap();
        let mut files = vec![raw];
        if label == "development" {
            files.push(destination.join("entry.ts"));
        }
        let config = serde_json::json!({"compilerOptions": {
            "strict":true, "noEmit":true, "skipLibCheck":false,
            "target":"ES2022", "module":"ESNext", "moduleResolution":"Bundler",
            "types":["bun", "node"], "typeRoots":[root().join("node_modules/@types")],
            "baseUrl":root(), "paths":{"@seseragi/runtime/*":["runtime/ts/src/*"]},
            "lib":["ES2023", "DOM", "DOM.Iterable"], "allowImportingTsExtensions":true
        },"files":files});
        let config_path = destination.join("tsconfig.json");
        fs::write(&config_path, serde_json::to_vec(&config).unwrap()).unwrap();
        successful(bounded(
            Command::new(runtime("node", directory))
                .arg(root().join("node_modules/typescript/bin/tsc"))
                .arg("--project")
                .arg(&config_path),
            directory,
        ));
        for engine in ["bun", "node"] {
            let output = bounded(
                Command::new(runtime(engine, directory))
                    .arg(destination.join("entry.js"))
                    .current_dir(&destination)
                    .env("PATH", ""),
                directory,
            );
            assert_eq!(
                output.status.code(),
                Some(code),
                "{label}/{engine}: {output:?}"
            );
            assert_eq!(output.stdout, expected.as_bytes(), "{label}/{engine}");
            if code == 0 {
                assert!(output.stderr.is_empty(), "{output:?}");
            } else {
                assert_eq!(output.stderr, b"seseragi: runtime defect\n");
            }
        }
    }
}

#[test]
fn anonymous_defer_matches_named_cold_control_and_releases_ref_cycles_on_all_exits() {
    let source = include_str!(
        "../../../examples/spec/fixtures/rust-backend/deferred-branch-environment.ssrg"
    );
    let expected = include_str!(
        "../../../examples/spec/fixtures/rust-backend/deferred-branch-environment.stdout"
    );
    // The explicit instance in this fixture reproduced the same failure on107,
    // independently of generic impl sugar. Retain both source forms as controls.
    check(source, expected, 0);
    check(
        include_str!("../../../examples/spec/fixtures/rust-backend/generic-impl-effects.ssrg"),
        expected,
        0,
    );
    assert_eq!(source.matches("body \"ready\" released").count(), 1);
    check(
        &source.replace("body \"ready\" released", "body \"fail\" released"),
        "acquired\ncreated\ncold\nreleased\nFailure: body failure\nreleased: 1\n",
        0,
    );
    check(
        &source.replace("body \"ready\" released", "body \"defect\" released"),
        "acquired\ncreated\ncold\nreleased\n",
        70,
    );
}

#[test]
fn match_tail_executes_the_prefix_once_on_each_selected_branch() {
    let source = r#"
import * as effects from "std/effect"
effect fn ignore _error: ConsoleError -> Unit with Console = effects.succeed ()
effect fn say message: String -> Unit with Console = println message |> effects.recover ignore
effect fn choose flag: Bool -> Bool with Console = do {
  let cold = effects.defer $ \_: Unit -> do {
    say "cold"
    match flag { True -> effects.succeed True; False -> effects.succeed False }
  }
  say "created"
  cold
}
pub effect fn main = do {
  first <- choose True
  println (show first)
  second <- choose False
  println (show second)
}
"#;
    check(source, "created\ncold\nTrue\ncreated\ncold\nFalse\n", 0);
}

#[test]
fn returning_a_cold_effect_does_not_execute_or_flatten_it() {
    check(
        include_str!("../../../examples/spec/fixtures/rust-backend/deferred-nested-cold.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/deferred-nested-cold.stdout"),
        0,
    );
}
