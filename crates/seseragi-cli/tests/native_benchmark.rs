#[path = "../../test-support/native_cache.rs"]
mod native_cache;
#[allow(dead_code)]
#[path = "support/generic_impl_process.rs"]
mod process;

use process::{bounded, runtime, successful};
use serde_json::{json, Value};
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
    let package = directory.join("package");
    fs::create_dir_all(package.join("src")).unwrap();
    fs::create_dir(package.join("benchmarks")).unwrap();
    fs::write(
        package.join("src/library.ssrg"),
        "pub let answer:Int = 42\n",
    )
    .unwrap();
    fs::write(package.join("seseragi.toml"), "[package]\nname=\"fixture/native-benchmark\"\nversion=\"0.0.0\"\nlanguage=\">=0.1.0\"\n[benchmark]\ntarget=\"native\"\nwarmup=0\nsamples=3\nminimum_sample_ms=1\n").unwrap();
    successful(bounded(
        cli().args(["lock", "update"]).arg(&package),
        directory,
    ));
    package
}
fn write_case(package: &Path, expression: &str) {
    fs::write(package.join("benchmarks/basic.ssrg"), format!("import * as b from \"std/benchmark\"\nimport * as e from \"std/effect\"\neffect fn mustNotRun -> Unit = e.die \"must not execute\"\npub let benchmarks:b.Benchmark = {expression}\n")).unwrap();
}
fn run(package: &Path, directory: &Path, target: &str, arguments: &[&str]) -> Output {
    let output = bounded(
        cli()
            .arg("benchmark")
            .arg(package)
            .current_dir(directory)
            .args(["--target", target])
            .args(arguments),
        directory,
    );
    output
}
fn report(output: &Output, code: i32) -> Value {
    assert_eq!(output.status.code(), Some(code), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn benchmark_values_match_strict_ts_bun_node_and_cargo_native() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    let input = native_cache::copy_input(
        &root().join("examples/spec/fixtures/rust-backend/std-benchmark.ssrg"),
        directory,
    );
    let expected =
        include_bytes!("../../../examples/spec/fixtures/rust-backend/std-benchmark.stdout");
    let out = directory.join("native");
    successful(bounded(
        cli()
            .args(["experimental", "build-native"])
            .arg(&input)
            .args(["--entry", "main", "--out-dir"])
            .arg(&out),
        directory,
    ));
    let native = successful(bounded(
        Command::new(out.join(format!("program{}", std::env::consts::EXE_SUFFIX)))
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1"),
        directory,
    ));
    assert_eq!(native.stdout, expected);
    assert!(
        String::from_utf8_lossy(&native.stderr).contains("managed-live=0"),
        "{native:?}"
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
                "native/benchmark",
                &fs::read_to_string(&input).unwrap(),
            )
            .with_profile(build_profile),
        )
        .unwrap();
        let raw_path = dist.join("raw-original.ts");
        fs::write(&raw_path, raw.generated.typescript).unwrap();
        let config = json!({"compilerOptions":{
            "strict":true,"noEmit":true,"skipLibCheck":false,"target":"ES2022",
            "module":"ESNext","moduleResolution":"Bundler","allowImportingTsExtensions":true,
            "types":["bun","node"],"typeRoots":[root().join("node_modules/@types")],
            "baseUrl":root(),"paths":{"@seseragi/runtime/*":["runtime/ts/src/*"]},
            "lib":["ES2023","DOM","DOM.Iterable"]},"files":[raw_path]});
        let config_path = dist.join("tsconfig.json");
        fs::write(&config_path, serde_json::to_vec(&config).unwrap()).unwrap();
        successful(bounded(
            Command::new(runtime("node", directory))
                .arg(root().join("node_modules/typescript/bin/tsc"))
                .arg("--project")
                .arg(config_path),
            directory,
        ));
        let manifest: Value =
            serde_json::from_slice(&fs::read(dist.join("artifact-manifest.json")).unwrap())
                .unwrap();
        for engine in ["bun", "node"] {
            let output = successful(bounded(
                Command::new(runtime(engine, directory))
                    .arg(dist.join(manifest["entry"].as_str().unwrap()))
                    .env("PATH", ""),
                directory,
            ));
            assert_eq!(output.stdout, expected, "{profile}/{engine}");
            assert!(output.stderr.is_empty(), "{output:?}");
        }
    }
}

#[test]
fn benchmark_failure_show_and_main_failures_match_bun_node_and_cargo_native() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    let input = directory.join("main.ssrg");
    for (literal, message) in [
        (r#""x""#, "x"),
        (r#""""#, ""),
        (r#""雪🙂\n\t\"\\\u{0}""#, "雪🙂\n\t\"\\\0"),
        (r#""x\n""#, "x\n"),
    ] {
        let displayed = format!("ExplicitBenchmarkFailure {message}");
        for (expression, code) in [
            (
                format!("println (show (b.ExplicitBenchmarkFailure {literal}))"),
                0,
            ),
            (format!("b.fail {literal}"), 1),
        ] {
            fs::write(
                &input,
                format!(
                    "import * as b from \"std/benchmark\"\npub effect fn main = {expression}\n"
                ),
            )
            .unwrap();
            let stdout = if code == 0 {
                format!("{displayed}\n")
            } else {
                String::new()
            };
            let stderr = if code == 0 {
                String::new()
            } else if displayed.ends_with('\n') {
                displayed.clone()
            } else {
                format!("{displayed}\n")
            };
            let out = directory.join("native");
            successful(bounded(
                cli()
                    .args(["experimental", "build-native"])
                    .arg(&input)
                    .args(["--entry", "main", "--out-dir"])
                    .arg(&out),
                directory,
            ));
            let native = bounded(
                Command::new(out.join(format!("program{}", std::env::consts::EXE_SUFFIX)))
                    .env("PATH", ""),
                directory,
            );
            assert_eq!(native.status.code(), Some(code), "{native:?}");
            assert_eq!(native.stdout, stdout.as_bytes(), "{expression}");
            let native_stderr = if code == 0 {
                String::new()
            } else {
                format!("{}:2:22: {stderr}", input.display())
            };
            assert_eq!(native.stderr, native_stderr.as_bytes(), "{expression}");
            for profile in ["development", "release"] {
                let dist = directory.join(profile);
                successful(bounded(
                    cli()
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
                        .arg(&dist),
                    directory,
                ));
                let manifest: Value =
                    serde_json::from_slice(&fs::read(dist.join("artifact-manifest.json")).unwrap())
                        .unwrap();
                for engine in ["bun", "node"] {
                    let output = bounded(
                        Command::new(runtime(engine, directory))
                            .arg(dist.join(manifest["entry"].as_str().unwrap()))
                            .env("PATH", ""),
                        directory,
                    );
                    assert_eq!(output.status.code(), native.status.code(), "{output:?}");
                    assert_eq!(
                        output.stdout, native.stdout,
                        "{profile}/{engine}/{expression}"
                    );
                    assert_eq!(
                        output.stderr,
                        stderr.as_bytes(),
                        "{profile}/{engine}/{expression}"
                    );
                }
            }
        }
    }
}

fn verify_measurement(case: &Value) {
    assert_eq!(case["status"], "passed");
    assert!(case.get("failure").is_none());
    let iterations = case["iterations"].as_u64().unwrap();
    assert!(iterations > 0 && iterations <= 1_000_000_000);
    let samples = case["samples"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_f64().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(samples.len(), 3);
    assert!(samples
        .iter()
        .all(|sample| sample.is_finite() && sample * iterations as f64 >= 1_000_000.0));
    let mut sorted = samples.clone();
    sorted.sort_by(f64::total_cmp);
    let median = sorted[1];
    let mut deviations = samples
        .iter()
        .map(|value| (value - median).abs())
        .collect::<Vec<_>>();
    deviations.sort_by(f64::total_cmp);
    for (key, value) in [
        ("median", median),
        ("medianAbsoluteDeviation", deviations[1]),
        ("minimum", sorted[0]),
        ("maximum", sorted[2]),
        ("sampleCount", 3.0),
    ] {
        assert_eq!(case["summary"][key].as_f64(), Some(value), "{case}");
    }
    assert_eq!(case["stdout"], "");
    assert_eq!(case["stderr"], "");
}

#[test]
fn native_benchmark_discovers_sizes_selection_measurements_and_baselines_like_ts() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    let package = package(directory);
    write_case(&package, "b.inputSize 1 (b.suite \"雪\" [b.inputSize 9007199254740991 (b.benchmark \"inner\" (e.succeed ())), b.benchmark \"inherited\" (e.succeed ()), b.inputSize 2 (b.inputSize 3 (b.benchmark \"nested\" (e.succeed ())))])");
    fs::write(package.join("benchmarks/z.ssrg"), "import * as b from \"std/benchmark\"\nimport * as e from \"std/effect\"\npub let benchmarks:b.Benchmark = b.benchmark \"last\" (e.succeed ())\n").unwrap();
    let lock = fs::read(package.join("seseragi.lock")).unwrap();
    for target in ["node", "native"] {
        let baseline = format!("{target}-baseline.json");
        let output = run(
            &package,
            directory,
            target,
            &["--json", "--warmup", "2", "--save-baseline", &baseline],
        );
        let measured = report(&output, 0);
        assert_eq!(measured["schema"], 1);
        assert_eq!(measured["kind"], "benchmark-report");
        assert_eq!(measured["metadata"]["profile"], "release");
        assert_eq!(measured["config"]["warmup"], 2);
        let cases = measured["cases"].as_array().unwrap();
        assert_eq!(
            cases
                .iter()
                .map(|case| (case["name"].as_str().unwrap(), case["inputSize"].as_i64()))
                .collect::<Vec<_>>(),
            [
                ("basic::雪::inner", Some(9007199254740991)),
                ("basic::雪::inherited", Some(1)),
                ("basic::雪::nested", Some(3)),
                ("z::last", None)
            ]
        );
        for case in cases {
            verify_measurement(case);
        }
        let baseline_path = directory.join(&baseline);
        assert!(baseline_path.is_file());
        assert!(!package.join(&baseline).exists());
        let mut saved: Value = serde_json::from_slice(&fs::read(&baseline_path).unwrap()).unwrap();
        for case in saved["cases"].as_array_mut().unwrap() {
            case["samples"] = json!([1e30, 1e30, 1e30]);
            case["summary"] = json!({"median":1e30,"medianAbsoluteDeviation":0,"minimum":1e30,"maximum":1e30,"sampleCount":3});
        }
        fs::write(&baseline_path, saved.to_string()).unwrap();
        let selected = report(
            &run(
                &package,
                directory,
                target,
                &[
                    "--json",
                    "--baseline",
                    &baseline,
                    "--exact",
                    "basic::雪::inner",
                ],
            ),
            0,
        );
        assert_eq!(selected["cases"].as_array().unwrap().len(), 1);
        assert_eq!(selected["comparison"][0]["status"], "unchanged");
        assert_eq!(selected["comparison"][1]["status"], "missing");
        saved["cases"][0]["inputSize"] = json!(2);
        fs::write(&baseline_path, saved.to_string()).unwrap();
        let changed = report(
            &run(
                &package,
                directory,
                target,
                &[
                    "--json",
                    "--baseline",
                    &baseline,
                    "--exact",
                    "basic::雪::inner",
                ],
            ),
            1,
        );
        assert_eq!(changed["comparison"][0]["status"], "incomparable");
        saved["metadata"]["host"]["cpu"] = json!("incompatible");
        fs::write(&baseline_path, saved.to_string()).unwrap();
        let incompatible = run(&package, directory, target, &["--baseline", &baseline]);
        assert_eq!(incompatible.status.code(), Some(1));
        assert!(incompatible.stdout.is_empty());
        assert_eq!(incompatible.stderr, b"benchmark baseline: Error: incompatible benchmark baseline host/profile/target/timer\n");
        let text = successful(run(&package, directory, target, &["--filter", "nested"]));
        let line = String::from_utf8(text.stdout).unwrap();
        assert!(line.starts_with("basic::雪::nested: median "), "{line}");
        assert!(
            line.contains(" ns; MAD ")
                && line.contains("; min ")
                && line.contains("; max ")
                && line.contains("; 3 samples x ")
        );
        assert!(line.ends_with("; input 3\n"), "{line}");
    }
    assert_eq!(fs::read(package.join("seseragi.lock")).unwrap(), lock);
}

#[test]
fn native_benchmark_discovery_errors_match_ts_before_filter_or_body_execution() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    let package = package(directory);
    for (expression, detail) in [
        (
            "b.benchmark \"\" (mustNotRun ())",
            "Error: case name is not canonical: \"\"",
        ),
        (
            "b.suite \"a::b\" []",
            "Error: suite name is not canonical: \"a::b\"",
        ),
        (
            "b.benchmark \"a\\nb\" (e.succeed ())",
            "Error: case name is not canonical: \"a\\nb\"",
        ),
        (
            "b.benchmark \"a\\rb\" (e.succeed ())",
            "Error: case name is not canonical: \"a\\rb\"",
        ),
        (
            "b.inputSize 0 (b.benchmark \"a\" (e.succeed ()))",
            "inputSize must be a positive Int",
        ),
        (
            "b.inputSize (-1) (b.suite \"empty\" [])",
            "inputSize must be a positive Int",
        ),
        (
            "b.suite \"s\" [b.benchmark \"a\" (mustNotRun ()), b.benchmark \"a\" (e.succeed ())]",
            "duplicate benchmark name basic::s::a",
        ),
        (
            "b.suite \"empty\" []",
            "benchmark discovery selected zero cases",
        ),
        (
            "b.benchmark \"valid\" (mustNotRun ())",
            "benchmark discovery selected zero cases",
        ),
    ] {
        write_case(&package, expression);
        let ts = run(&package, directory, "node", &["--filter", "missing"]);
        let native = run(&package, directory, "native", &["--filter", "missing"]);
        assert_eq!(ts.status.code(), Some(2), "{ts:?}");
        assert_eq!(native.status.code(), Some(2), "{native:?}");
        assert!(ts.stdout.is_empty());
        assert_eq!(native.stdout, ts.stdout);
        assert_eq!(
            ts.stderr,
            format!("benchmark: Error: {detail}\n").as_bytes()
        );
        assert_eq!(native.stderr, ts.stderr);
    }
}

#[test]
fn native_benchmark_failures_captures_seeds_timeouts_and_cleanup_match_ts() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    let package = package(directory);
    fs::write(
        package.join("benchmarks/basic.ssrg"),
        r#"
import * as b from "std/benchmark"
import * as e from "std/effect"
import * as d from "std/deferred"
import * as random from "std/random"
import { Random } from "std/random"
effect fn ignore _error:ConsoleError -> Unit with Console = e.succeed ()
effect fn say message:String -> Unit with Console = println message |> e.recover ignore
effect fn wait -> Unit = do { gate <- d.make<Never,Unit> (); d.await gate }
fn release _resource:Unit -> Effect<{},Never,Unit> = wait ()
effect fn released _resource:Unit -> Unit with Console = say "released"
effect fn acquired -> Unit with Console = e.succeed ()
effect fn broken -> Unit = e.die "broken"
effect fn cleanupBroken _resource:Unit -> Unit = e.die "finalizer"
effect fn typed -> Unit with Console fails b.BenchmarkFailure = do {
  e.acquireRelease (acquired ()) released
  say "body"
  b.fail "雪\n\""
}
effect fn randomFailure -> Unit with Random fails b.BenchmarkFailure = do {
  algorithm <- random.algorithmId ()
  first <- random.nextInt ()
  second <- random.nextInt ()
  b.fail (algorithm + ":" + show first + ":" + show second)
}
effect fn leak -> Unit = do { e.acquireRelease (e.succeed ()) release; e.succeed () }
effect fn timeoutLeak -> Unit = do { e.acquireRelease (e.succeed ()) release; wait () }
effect fn defectLeak -> Unit = do { e.acquireRelease (e.succeed ()) release; e.die "broken" }
effect fn cleanupDefect -> Unit fails b.BenchmarkFailure = do {
  e.acquireRelease (e.succeed ()) cleanupBroken
  b.fail "original"
}
pub let benchmarks:b.Benchmark = b.suite "failures" [
  b.benchmark "typed" (typed ()),
  b.benchmark "empty" (b.fail ""),
  b.benchmark "defect" (broken ()),
  b.benchmark "cleanup-defect" (cleanupDefect ()),
  b.benchmark "random" (randomFailure ()),
  b.benchmark "timeout" (wait ()),
  b.benchmark "leak" (leak ()),
  b.benchmark "timeout-leak" (timeoutLeak ()),
  b.benchmark "defect-leak" (defectLeak ())
]
"#,
    )
    .unwrap();
    for seed in ["0", "-9223372036854775808", "9223372036854775807"] {
        let args = [
            "--json",
            "--timeout-ms",
            "40",
            "--cleanup-grace-ms",
            "40",
            "--seed",
            seed,
        ];
        let ts = report(&run(&package, directory, "node", &args), 1);
        let native = report(&run(&package, directory, "native", &args), 1);
        assert_eq!(native["config"], ts["config"]);
        assert_eq!(native["cases"], ts["cases"]);
        let cases = ts["cases"].as_array().unwrap();
        assert_eq!(
            cases
                .iter()
                .map(|case| case["failure"]["kind"].as_str().unwrap())
                .collect::<Vec<_>>(),
            [
                "typed-failure",
                "typed-failure",
                "defect",
                "defect",
                "typed-failure",
                "timeout",
                "resource-leak",
                "resource-leak",
                "resource-leak"
            ]
        );
        assert_eq!(cases[0]["stdout"], "body\nreleased\n");
        assert_eq!(
            cases[0]["failure"]["detail"],
            "{\"tag\":\"ExplicitBenchmarkFailure\",\"value\":\"雪\\n\\\"\"}"
        );
        assert_eq!(cases[3]["failure"]["detail"], "finalizer");
        for case in cases {
            assert!(
                case.get("samples").is_none()
                    && case.get("summary").is_none()
                    && case.get("iterations").is_none()
            );
        }
    }
    let args = ["--exact", "basic::failures::typed"];
    let ts = run(&package, directory, "node", &args);
    let native = run(&package, directory, "native", &args);
    assert_eq!(ts.status.code(), Some(1));
    assert_eq!(native.status.code(), Some(1));
    assert_eq!(native.stdout, ts.stdout);
    assert_eq!(native.stderr, ts.stderr);
}
