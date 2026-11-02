#[path = "../../test-support/native_cache.rs"]
mod native_cache;
#[path = "support/generic_impl_process.rs"]
mod process;

use process::{bounded, runtime, successful};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}
fn cli() -> Command {
    native_cache::command(env!("CARGO_BIN_EXE_seseragi"))
}
fn package(directory: &Path) -> PathBuf {
    let package = native_cache::copy_input(
        &root().join("examples/spec/fixtures/rust-backend-projects/std-test"),
        directory,
    );
    successful(bounded(
        cli().args(["lock", "update"]).arg(&package),
        directory,
    ));
    package
}
fn run_test(package: &Path, directory: &Path, target: &str, arguments: &[&str]) -> Output {
    bounded(
        cli()
            .arg("test")
            .arg(package)
            .args(["--target", target])
            .args(arguments),
        directory,
    )
}
fn compare(package: &Path, directory: &Path, arguments: &[&str], code: i32) -> Output {
    let ts = run_test(package, directory, "node", arguments);
    let native = run_test(package, directory, "native", arguments);
    assert_eq!(ts.status.code(), Some(code), "{ts:?}");
    assert_eq!(native.status.code(), Some(code), "{native:?}");
    assert_eq!(native.stdout, ts.stdout);
    assert_eq!(native.stderr, ts.stderr);
    ts
}

#[test]
fn std_test_apis_saved_generic_and_selected_dictionaries_match_strict_ts_bun_node_and_native() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    let input = native_cache::copy_input(
        &root().join("examples/spec/fixtures/rust-backend/std-test.ssrg"),
        directory,
    );
    let expected = include_bytes!("../../../examples/spec/fixtures/rust-backend/std-test.stdout");
    let native = directory.join("native");
    successful(bounded(
        cli()
            .args(["experimental", "build-native"])
            .arg(&input)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native),
        directory,
    ));
    let output = successful(bounded(
        Command::new(native.join(format!("program{}", std::env::consts::EXE_SUFFIX)))
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1"),
        directory,
    ));
    assert_eq!(output.stdout, expected);
    assert!(
        String::from_utf8_lossy(&output.stderr).starts_with("native managed-live=0 "),
        "{output:?}"
    );
    for (profile, build_profile) in [
        ("development", seseragi_driver::BuildProfile::Development),
        ("release", seseragi_driver::BuildProfile::Release),
    ] {
        let dist = directory.join(profile);
        successful(bounded(
            cli()
                .arg("build")
                .arg(&input)
                .args(["--target", "process", "--profile", profile, "--out-dir"])
                .arg(&dist),
            directory,
        ));
        let raw = seseragi_driver::compile_module(
            seseragi_driver::CompileInput::new(
                "main.ssrg",
                "native/test",
                &fs::read_to_string(&input).unwrap(),
            )
            .with_profile(build_profile),
        )
        .unwrap();
        let raw_path = dist.join("raw-original.ts");
        fs::write(&raw_path, raw.generated.typescript).unwrap();
        let config = serde_json::json!({"compilerOptions":{
            "strict":true,"noEmit":true,"skipLibCheck":false,"target":"ES2022",
            "module":"ESNext","moduleResolution":"Bundler","allowImportingTsExtensions":true,
            "types":["bun","node"],"typeRoots":[root().join("node_modules/@types")],
            "baseUrl":root(),"paths":{"@seseragi/runtime/*":["runtime/ts/src/*"]},
            "lib":["ES2023","DOM","DOM.Iterable"]},"files":[raw_path]});
        let config_path = dist.join("raw-tsconfig.json");
        fs::write(&config_path, serde_json::to_vec(&config).unwrap()).unwrap();
        successful(bounded(
            Command::new(runtime("node", directory))
                .arg(root().join("node_modules/typescript/bin/tsc"))
                .arg("--project")
                .arg(config_path),
            directory,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(dist.join("artifact-manifest.json")).unwrap())
                .unwrap();
        let entry = dist.join(manifest["entry"].as_str().unwrap());
        for engine in ["bun", "node"] {
            let output = successful(bounded(
                Command::new(runtime(engine, directory))
                    .arg(&entry)
                    .env("PATH", ""),
                directory,
            ));
            assert_eq!(output.stdout, expected, "{profile}/{engine}");
            assert!(output.stderr.is_empty(), "{output:?}");
        }
    }
}

#[test]
fn native_test_runner_matches_results_output_timeout_skip_and_seed_in_discovery_order() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    let package = package(directory);
    let output = compare(&package, directory, &[], 1);
    let expected = concat!(
        "PASS basic::suite::pass\nPASS basic::suite::false\nPASS basic::suite::unequal\n",
        "FAIL basic::suite::equal-failure\nFAIL basic::suite::not-equal-failure\n",
        "FAIL basic::suite::true-failure\nFAIL basic::suite::false-failure\nFAIL basic::suite::explicit\n",
        "PASS basic::suite::expected\nFAIL basic::suite::mismatch\nFAIL basic::suite::success\nFAIL basic::suite::defect\n",
        "SKIP basic::suite::skipped::one -- outer\nSKIP basic::suite::skipped::two -- inner\n",
        "FAIL basic::suite::timeout\nPASS basic::suite::zero\nPASS basic::suite::output\nPASS basic::suite::environment\n",
        "PASS second::pass\n8 passed; 9 failed; 2 skipped\n"
    );
    assert_eq!(output.stdout, expected.as_bytes());
    assert_eq!(output.stderr, concat!(
        "basic::suite::equal-failure: values are not equal; expected 1; actual 2\n",
        "basic::suite::not-equal-failure: values are equal; actual \"雪\"\n",
        "basic::suite::true-failure: expected true; expected true; actual false\n",
        "basic::suite::false-failure: expected false; expected false; actual true\n",
        "basic::suite::explicit: boom\nbasic::suite::mismatch: typed failure did not match: \"typed\"\n",
        "basic::suite::success: expected a typed failure\nbasic::suite::defect: defect: kaboom\n",
        "basic::suite::timeout: timed out after 2 ms\n",
        "basic::suite::output stdout:\nout雪\nbasic::suite::output stderr:\nerr🙂\n",
        "{\"level\":\"info\",\"message\":\"message\",\"fields\":[]}\n",
        "basic::suite::environment stdout:\nseseragi-test-xorshift64\n1195535348,-1368485341\n"
    ).as_bytes());
    let replay = compare(&package, directory, &["--jobs", "1"], 1);
    assert_eq!(replay.stdout, output.stdout);
    assert_eq!(replay.stderr, output.stderr);
    let selected = compare(
        &package,
        directory,
        &["--exact", "basic::suite::environment"],
        0,
    );
    assert_eq!(
        selected.stdout,
        b"PASS basic::suite::environment\n1 passed; 0 failed; 0 skipped\n"
    );
    assert_eq!(
        selected.stderr,
        b"basic::suite::environment stdout:\nseseragi-test-xorshift64\n1195535348,-1368485341\n"
    );
    let skipped = compare(&package, directory, &["--filter", "skipped"], 0);
    assert_eq!(skipped.stdout, b"SKIP basic::suite::skipped::one -- outer\nSKIP basic::suite::skipped::two -- inner\n0 passed; 0 failed; 2 skipped\n");
    let empty = compare(&package, directory, &["--filter", "missing"], 2);
    assert!(empty.stdout.is_empty());
    assert_eq!(
        empty.stderr,
        b"seseragi: test selection matched zero cases\n"
    );
    for seed in ["-9007199254740991", "9223372036854775807"] {
        compare(
            &package,
            directory,
            &["--exact", "basic::suite::environment", "--seed", seed],
            0,
        );
    }
    let native = bounded(
        cli()
            .arg("test")
            .arg(&package)
            .args(["--target", "native"])
            .env("SESERAGI_NATIVE_METRICS", "1"),
        directory,
    );
    assert_eq!(native.status.code(), Some(1));
    assert_eq!(native.stdout, output.stdout);
    let stderr = String::from_utf8(native.stderr).unwrap();
    assert!(stderr
        .strip_prefix(std::str::from_utf8(&output.stderr).unwrap())
        .unwrap()
        .starts_with("native managed-live=0 "));
}

#[test]
fn native_discovery_rejects_invalid_names_duplicates_and_empty_trees_like_ts() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    let package = package(directory);
    fs::remove_file(package.join("tests/second.ssrg")).unwrap();
    for (expression, detail) in [
        ("t.test \"\" (t.isTrue True)", "case name is not canonical: \"\""),
        ("t.suite \"a::b\" [t.test \"ok\" (t.isTrue True)]", "suite name is not canonical: \"a::b\""),
        ("t.skip \"later\\nnow\" (t.test \"ok\" (t.isTrue True))", "skip reason name is not canonical: \"later\\nnow\""),
        ("t.test \"bad\\rname\" (t.isTrue True)", "case name is not canonical: \"bad\\rname\""),
        ("t.suite \"s\" [t.test \"same\" (t.isTrue True),t.skip \"later\" (t.test \"same\" (t.isTrue True))]", "duplicate test name basic::s::same"),
        ("t.suite \"empty\" []", "test selection matched zero cases"),
    ] {
        fs::write(package.join("tests/basic.ssrg"), format!("import * as t from \"std/test\"\npub let tests:t.Test = {expression}\n")).unwrap();
        let output = compare(&package, directory, &["--filter", "missing"], 2);
        assert!(output.stdout.is_empty());
        assert_eq!(output.stderr, format!("seseragi: {detail}\n").as_bytes());
    }
}

#[test]
fn native_timeouts_cancel_owned_effects_run_finalizers_and_report_cleanup_leaks_like_ts() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    let package = package(directory);
    fs::remove_file(package.join("tests/second.ssrg")).unwrap();
    fs::write(package.join("tests/basic.ssrg"), r#"
import * as t from "std/test"
import * as e from "std/effect"
import * as d from "std/deferred"
import * as time from "std/time"
effect fn wait -> Unit = do { gate <- d.make<Never,Unit> (); d.await gate }
effect fn ignore _error:ConsoleError -> Unit with Console = e.succeed ()
effect fn acquired -> Unit with Console = e.succeed ()
fn release _resource:Unit -> Effect<{},Never,Unit> = wait ()
effect fn released _resource:Unit -> Unit with Console = println "released" |> e.recover ignore
effect fn clean = do { e.acquireRelease (acquired ()) released; wait () }
effect fn timeoutLeak = do { e.acquireRelease (e.succeed ()) release; wait () }
effect fn defectLeak -> Unit = do { e.acquireRelease (e.succeed ()) release; e.die "defect" }
effect fn successLeak -> Unit = do { e.acquireRelease (e.succeed ()) release; e.succeed () }
let short = match time.nanoseconds 999999 { Left _ -> time.zeroDuration (); Right duration -> duration }
pub let tests:t.Test = t.suite "cleanup" [
  t.timeout short (t.test "timeout" (clean ())),
  t.timeout short (t.test "timeout-leak" (timeoutLeak ())),
  t.test "defect-leak" (defectLeak ()),
  t.test "success-leak" (successLeak ())
]
"#).unwrap();
    let output = compare(&package, directory, &[], 1);
    assert_eq!(output.stdout, b"FAIL basic::cleanup::timeout\nFAIL basic::cleanup::timeout-leak\nFAIL basic::cleanup::defect-leak\nFAIL basic::cleanup::success-leak\n0 passed; 4 failed; 0 skipped\n");
    assert_eq!(output.stderr, b"basic::cleanup::timeout: timed out after 0 ms\nbasic::cleanup::timeout stdout:\nreleased\nbasic::cleanup::timeout-leak: resource leak after timeout\nbasic::cleanup::defect-leak: resource leak after defect\nbasic::cleanup::success-leak: resource leak during cleanup\n");
    let manifest = package.join("seseragi.toml");
    fs::write(
        &manifest,
        fs::read_to_string(&manifest)
            .unwrap()
            .replace("cleanup_grace_ms = 20", "cleanup_grace_ms = 0"),
    )
    .unwrap();
    successful(bounded(
        cli().args(["lock", "update"]).arg(&package),
        directory,
    ));
    let output = compare(
        &package,
        directory,
        &["--exact", "basic::cleanup::timeout"],
        1,
    );
    assert_eq!(
        output.stderr,
        b"basic::cleanup::timeout: resource leak after timeout\n"
    );
}
