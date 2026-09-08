use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Temporary(PathBuf);
impl Temporary {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "seseragi-benchmark-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(path.join("src")).unwrap();
        fs::create_dir(path.join("benchmarks")).unwrap();
        fs::write(path.join("src/library.ssrg"), "pub let answer = 42\n").unwrap();
        fs::write(path.join("seseragi.toml"), "[package]\nname = \"fixture/benchmark\"\nversion = \"0.0.0\"\nlanguage = \">=0.1.0\"\n[benchmark]\ntarget = \"node\"\nwarmup = 0\nsamples = 3\nminimum_sample_ms = 1\n").unwrap();
        Self(path)
    }
    fn lock(&self) {
        success(cli(&self.0, &["lock", "update", "."]));
    }
}
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn cli(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(args)
        .current_dir(root)
        .output()
        .unwrap()
}
fn success(output: Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}
fn write_case(root: &Path, body: &str) {
    fs::write(root.join("benchmarks/basic.ssrg"), format!("import * as benchmark from \"std/benchmark\"\nimport * as effects from \"std/effect\"\npub let benchmarks: benchmark.Benchmark = benchmark.benchmark \"case\" ({body})\n")).unwrap();
}
fn set_samples(report: &mut serde_json::Value, value: f64) {
    let count = report["config"]["samples"].as_u64().unwrap() as usize;
    for case in report["cases"].as_array_mut().unwrap() {
        case["samples"] = serde_json::json!(vec![value; count]);
        case["summary"] = serde_json::json!({"median":value,"medianAbsoluteDeviation":0,"minimum":value,"maximum":value,"sampleCount":count});
    }
}

#[test]
fn discovers_canonical_fixture_and_reports_measurement_and_baselines() {
    let temporary = Temporary::new();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/spec/fixtures/projects/benchmark-discovery");
    fs::copy(
        fixture.join("src/library.ssrg"),
        temporary.0.join("src/library.ssrg"),
    )
    .unwrap();
    fs::copy(
        fixture.join("benchmarks/basic.ssrg"),
        temporary.0.join("benchmarks/basic.ssrg"),
    )
    .unwrap();
    fs::write(
        temporary.0.join("seseragi.toml"),
        fs::read_to_string(fixture.join("seseragi.toml"))
            .unwrap()
            .replace(">=0.1.0 <0.2.0", ">=0.1.0")
            .replace("test-js", "node"),
    )
    .unwrap();
    temporary.lock();
    let lock = fs::read(temporary.0.join("seseragi.lock")).unwrap();
    let stdout = success(cli(
        &temporary.0,
        &[
            "benchmark",
            ".",
            "--json",
            "--save-baseline",
            "baseline.json",
        ],
    ));
    let mut report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(report["schema"], 1);
    assert_eq!(report["metadata"]["profile"], "release");
    assert_eq!(
        report["metadata"]["target"]["identity"],
        "seseragi/bun-process"
    );
    assert_eq!(report["cases"][0]["name"], "basic::array::sum");
    assert_eq!(report["cases"][0]["inputSize"], 5);
    assert_eq!(report["cases"][0]["samples"].as_array().unwrap().len(), 5);
    let iterations = report["cases"][0]["iterations"].as_u64().unwrap() as f64;
    assert!(report["cases"][0]["samples"]
        .as_array()
        .unwrap()
        .iter()
        .all(|value| value.as_f64().unwrap() * iterations >= 10_000_000.0));
    assert_eq!(fs::read(temporary.0.join("seseragi.lock")).unwrap(), lock);
    assert!(temporary.0.join("baseline.json").is_file());
    set_samples(&mut report, 1e30);
    fs::write(temporary.0.join("baseline.json"), report.to_string()).unwrap();
    success(cli(
        &temporary.0,
        &[
            "benchmark",
            ".",
            "--baseline",
            "baseline.json",
            "--exact",
            "basic::array::sum",
        ],
    ));
    set_samples(&mut report, 1e-9);
    fs::write(temporary.0.join("baseline.json"), report.to_string()).unwrap();
    let regression = cli(
        &temporary.0,
        &["benchmark", ".", "--baseline", "baseline.json", "--json"],
    );
    assert_eq!(regression.status.code(), Some(1));
    let current: serde_json::Value = serde_json::from_slice(&regression.stdout).unwrap();
    assert_eq!(current["comparison"][0]["status"], "regression");
    report["metadata"]["host"]["cpu"] = "incompatible".into();
    fs::write(temporary.0.join("baseline.json"), report.to_string()).unwrap();
    assert_eq!(
        cli(
            &temporary.0,
            &["benchmark", ".", "--baseline", "baseline.json"]
        )
        .status
        .code(),
        Some(1)
    );
}

#[test]
fn compile_discovery_and_option_errors_are_two_and_case_failures_are_one() {
    let temporary = Temporary::new();
    write_case(&temporary.0, "benchmark.fail \"expected\"");
    temporary.lock();
    let failure = cli(&temporary.0, &["benchmark", ".", "--json"]);
    assert_eq!(
        failure.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&failure.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&failure.stdout).unwrap();
    assert_eq!(report["cases"][0]["failure"]["kind"], "typed-failure");
    assert!(report["cases"][0].get("samples").is_none());
    for options in [
        vec!["--filter", "missing"],
        vec!["--samples", "2"],
        vec!["--regression-threshold-percent", "NaN"],
        vec!["--filter", "a", "--exact", "a"],
        vec!["--target", "web"],
    ] {
        let mut args = vec!["benchmark", "."];
        args.extend(options);
        assert_eq!(cli(&temporary.0, &args).status.code(), Some(2));
    }
    fs::write(
        temporary.0.join("benchmarks/excluded.ssrg"),
        "pub let invalid: Int = \"wrong\"\n",
    )
    .unwrap();
    assert_eq!(
        cli(&temporary.0, &["benchmark", ".", "--exact", "basic::case"])
            .status
            .code(),
        Some(2)
    );
    fs::remove_file(temporary.0.join("benchmarks/excluded.ssrg")).unwrap();
    fs::write(
        temporary.0.join("benchmarks/basic.ssrg"),
        "pub let benchmarks: Int = 42\n",
    )
    .unwrap();
    assert_eq!(
        cli(&temporary.0, &["benchmark", "."]).status.code(),
        Some(2)
    );
}

#[test]
fn benchmark_environment_rejects_unprovided_clock() {
    let temporary = Temporary::new();
    fs::write(temporary.0.join("benchmarks/basic.ssrg"), "import * as benchmark from \"std/benchmark\"\nimport * as clock from \"std/clock\"\npub let benchmarks: benchmark.Benchmark = benchmark.benchmark \"clock\" ((\\_ -> ()) <$> clock.now ())\n").unwrap();
    temporary.lock();
    let output = cli(&temporary.0, &["benchmark", "."]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("clock"));
}

#[test]
fn never_failure_widens_only_in_effect_failure_position_and_benchmark_failure_has_instances() {
    let temporary = Temporary::new();
    write_case(&temporary.0, "effects.defer (\\_ -> { let same = benchmark.ExplicitBenchmarkFailure \"x\" == benchmark.ExplicitBenchmarkFailure \"x\"; let text = show (benchmark.ExplicitBenchmarkFailure \"x\"); effects.succeed () })");
    temporary.lock();
    success(cli(&temporary.0, &["benchmark", ".", "--json"]));
    write_case(&temporary.0, "effects.fail \"wrong failure type\"");
    assert_eq!(
        cli(&temporary.0, &["benchmark", "."]).status.code(),
        Some(2)
    );
    fs::write(temporary.0.join("benchmarks/basic.ssrg"), "import * as benchmark from \"std/benchmark\"\nimport * as effects from \"std/effect\"\ntype Never = | Reachable\npub let benchmarks: benchmark.Benchmark = benchmark.benchmark \"custom-never\" (effects.fail Reachable)\n").unwrap();
    assert_eq!(
        cli(&temporary.0, &["benchmark", "."]).status.code(),
        Some(2)
    );
    write_case(&temporary.0, "effects.succeed 42");
    assert_eq!(
        cli(&temporary.0, &["benchmark", "."]).status.code(),
        Some(2)
    );
}

#[test]
fn release_quality_suite_executes_tool_root_imports_and_foreign_copy() {
    let temporary = Temporary::new();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/spec/fixtures/projects/benchmark-discovery");
    fs::copy(
        fixture.join("benchmarks/quality.ssrg"),
        temporary.0.join("benchmarks/quality.ssrg"),
    )
    .unwrap();
    fs::create_dir(temporary.0.join("host")).unwrap();
    fs::copy(
        fixture.join("host/copy.mjs"),
        temporary.0.join("host/copy.mjs"),
    )
    .unwrap();
    temporary.lock();
    let report: serde_json::Value =
        serde_json::from_str(&success(cli(&temporary.0, &["benchmark", ".", "--json"]))).unwrap();
    assert_eq!(report["cases"].as_array().unwrap().len(), 9);
    assert!(report["cases"]
        .as_array()
        .unwrap()
        .iter()
        .all(|case| case["status"] == "passed"));
    // Same module path in src/ and benchmarks/ must retain distinct identities;
    // a relative benchmark import stays in the benchmark root.
    fs::write(temporary.0.join("src/helper.ssrg"), "pub let number = 0\n").unwrap();
    fs::write(
        temporary.0.join("benchmarks/helper.ssrg"),
        "pub let number = 42\n",
    )
    .unwrap();
    fs::write(temporary.0.join("benchmarks/basic.ssrg"), "import * as benchmark from \"std/benchmark\"\nimport * as effects from \"std/effect\"\nimport { number } from \"./helper\"\npub let benchmarks: benchmark.Benchmark = benchmark.benchmark \"identity\" (if number == 42 then effects.succeed () else benchmark.fail \"wrong root\")\n").unwrap();
    temporary.lock();
    success(cli(
        &temporary.0,
        &["benchmark", ".", "--exact", "basic::identity"],
    ));
}

#[test]
fn benchmark_discovery_requires_a_local_public_let() {
    let temporary = Temporary::new();
    write_case(&temporary.0, "effects.succeed ()");
    fs::rename(
        temporary.0.join("benchmarks/basic.ssrg"),
        temporary.0.join("benchmarks/helper.ssrg"),
    )
    .unwrap();
    fs::write(
        temporary.0.join("benchmarks/basic.ssrg"),
        "pub import { benchmarks } from \"./helper\"\n",
    )
    .unwrap();
    temporary.lock();
    let output = cli(&temporary.0, &["benchmark", "."]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("benchmarks"));
}
