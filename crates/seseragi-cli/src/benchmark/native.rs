use serde_json::{json, Value};
use seseragi_driver::CompiledLocalBenchmarks;
use seseragi_runtime::BenchmarkRunOptions;
use std::{fs, io::Write, path::Path, process::Command};

pub(super) fn run(
    compiled: &CompiledLocalBenchmarks,
    root: &Path,
    options: &BenchmarkRunOptions,
) -> Result<i32, String> {
    let metadata = metadata(&options.version);
    let mut report = json!({"schema": 1, "kind": "benchmark-report", "metadata": metadata, "config": options.config, "cases": []});
    report["config"]["regressionThresholdPercent"] =
        super::baseline::json_number(&report["config"]["regressionThresholdPercent"]);
    let baseline = match &options.baseline {
        Some(path) => match fs::read(path)
            .map_err(|error| error.to_string())
            .and_then(|bytes| {
                serde_json::from_slice::<Value>(&bytes).map_err(|error| error.to_string())
            })
            .and_then(|value| {
                super::baseline::compare(&report, &value)?;
                Ok(value)
            }) {
            Ok(value) => Some(value),
            Err(error) => {
                eprintln!("benchmark baseline: Error: {error}");
                return Ok(1);
            }
        },
        None => None,
    };
    let modules = compiled
        .compiled
        .order
        .iter()
        .map(|id| &compiled.compiled.modules[id].core_ir)
        .collect::<Vec<_>>();
    let benchmarks = compiled
        .benchmark_modules
        .iter()
        .map(|module| {
            let export = compiled.compiled.modules[&module.module_id]
                .typed_interface
                .exports
                .iter()
                .find(|export| export.namespace == "value" && export.name == "benchmarks")
                .expect("discovered Benchmark export");
            (module.name.clone(), export.symbol.clone())
        })
        .collect::<Vec<_>>();
    let artifact =
        seseragi_backend_rust::emit_rust_benchmarks(&modules, &benchmarks).map_err(|error| {
            match &error.origin {
                Some(origin) => format!(
                    "{}:{}..{}: {error}",
                    origin.source, origin.start, origin.end
                ),
                None => error.to_string(),
            }
        })?;
    let scratch = tempfile::tempdir().map_err(|error| error.to_string())?;
    let executable = scratch.path().join(crate::experimental::output::EXECUTABLE);
    crate::experimental::native_cargo::build(
        root,
        "benchmarks",
        seseragi_project::BuildProfile::Release,
        &artifact.source,
        &executable,
    )?;
    let config = &options.config;
    let mut command = Command::new(executable);
    command.current_dir(root).args([
        "--warmup",
        &config.warmup.to_string(),
        "--samples",
        &config.samples.to_string(),
        "--minimum-sample-ms",
        &config.minimum_sample_ms.to_string(),
        "--timeout-ms",
        &config.timeout_ms.to_string(),
        "--cleanup-grace-ms",
        &config.cleanup_grace_ms.to_string(),
        "--seed",
        &config.seed,
    ]);
    if let Some(filter) = &options.filter {
        command.args(["--filter", filter]);
    }
    if let Some(exact) = &options.exact {
        command.args(["--exact", exact]);
    }
    let output = command.output().map_err(|error| error.to_string())?;
    std::io::stderr()
        .write_all(&output.stderr)
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Ok(output.status.code().unwrap_or(1));
    }
    report["cases"] = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("invalid native benchmark report: {error}"))?;
    let mut failed = report["cases"]
        .as_array()
        .unwrap()
        .iter()
        .any(|case| case["status"] == "failed");
    if let Some(baseline) = baseline {
        let comparison = super::baseline::compare(&report, &baseline)?;
        failed |= comparison.iter().any(|entry| {
            matches!(
                entry["status"].as_str(),
                Some("regression" | "incomparable" | "failure")
            )
        });
        report["comparison"] = comparison.into();
    }
    if let Some(path) = &options.save_baseline {
        if report["cases"]
            .as_array()
            .unwrap()
            .iter()
            .all(|case| case["status"] == "passed")
        {
            if let Err(error) = save_baseline(Path::new(path), &report) {
                eprintln!("benchmark baseline save: Error: {error}");
                return Ok(1);
            }
        }
    }
    if options.json {
        println!("{report}");
    } else {
        print_report(&report);
    }
    Ok(i32::from(failed))
}

fn metadata(version: &str) -> Value {
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        "windows" => "win32",
        os => os,
    };
    let architecture = match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        "x86" => "ia32",
        arch => arch,
    };
    json!({
        "languageVersion": version, "compilerVersion": version, "runtimeVersion": version,
        "profile": "release", "target": {"identity": "seseragi/native", "version": version},
        "host": {"os": os, "architecture": architecture, "cpu": cpu(), "logicalCores": std::thread::available_parallelism().map(usize::from).unwrap_or(1), "gcAvailable": true},
        "timerResolutionNs": 1,
    })
}

fn cpu() -> String {
    #[cfg(target_os = "macos")]
    if let Ok(output) = Command::new("/usr/sbin/sysctl")
        .args(["-n", "machdep.cpu.brand_string"])
        .output()
    {
        if output.status.success() {
            let value = String::from_utf8_lossy(&output.stdout).trim().to_owned();
            if !value.is_empty() {
                return value;
            }
        }
    }
    #[cfg(target_os = "linux")]
    if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
        if let Some(value) = cpuinfo
            .lines()
            .filter_map(|line| line.split_once(':'))
            .find_map(|(key, value)| {
                matches!(key.trim(), "model name" | "Hardware").then(|| value.trim().to_owned())
            })
        {
            if !value.is_empty() {
                return value;
            }
        }
    }
    #[cfg(windows)]
    if let Ok(value) = std::env::var("PROCESSOR_IDENTIFIER") {
        if !value.is_empty() {
            return value;
        }
    }
    "unknown".into()
}

fn save_baseline(path: &Path, report: &Value) -> Result<(), String> {
    super::baseline::validate(report)?;
    let parent = path.parent().ok_or("baseline destination has no parent")?;
    let mut temporary =
        tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    writeln!(
        temporary,
        "{}",
        serde_json::to_string_pretty(report).map_err(|error| error.to_string())?
    )
    .map_err(|error| error.to_string())?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|error| error.to_string())?;
    temporary.persist(path).map_err(|error| error.to_string())?;
    Ok(())
}

fn number(value: &Value) -> String {
    let text = value.to_string();
    text.strip_suffix(".0").unwrap_or(&text).to_owned()
}

fn print_report(report: &Value) {
    for case in report["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        if case["status"] == "failed" {
            println!(
                "FAIL {name}: {}: {}",
                case["failure"]["kind"].as_str().unwrap(),
                case["failure"]["detail"].as_str().unwrap()
            );
        } else {
            let summary = &case["summary"];
            let input_size = if case["inputSize"].is_null() {
                "unspecified".into()
            } else {
                number(&case["inputSize"])
            };
            println!(
                "{name}: median {} ns; MAD {}; min {}; max {}; {} samples x {}; input {input_size}",
                number(&summary["median"]),
                number(&summary["medianAbsoluteDeviation"]),
                number(&summary["minimum"]),
                number(&summary["maximum"]),
                number(&summary["sampleCount"]),
                number(&case["iterations"])
            );
        }
    }
    for entry in report["comparison"].as_array().into_iter().flatten() {
        println!(
            "{} {}",
            entry["status"].as_str().unwrap().to_uppercase(),
            entry["name"].as_str().unwrap()
        );
    }
}
