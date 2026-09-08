use seseragi_runtime::{run_local_benchmarks_in_directory, BenchmarkConfig, BenchmarkRunOptions};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(crate) fn benchmark(arguments: &[String]) -> Result<i32, String> {
    let invocation = Invocation::parse(arguments)?;
    let root = crate::local_project::containing_package(&invocation.path)
        .unwrap_or(invocation.path.clone());
    if !root.join("seseragi.toml").is_file() {
        return Err("benchmark expects a package containing seseragi.toml".to_owned());
    }
    seseragi_project::read_and_validate_lockfile(&root)
        .map_err(|error| format!("{}: {error}", error.code()))?;
    let project = seseragi_project::load_local_benchmarks(&root)
        .map_err(|error| format!("{}: {error}", error.code()))?;
    let manifest = project
        .packages()
        .package(project.packages().root())
        .expect("root package")
        .manifest();
    let settings = manifest.benchmark.as_ref();
    let target = invocation
        .target
        .as_deref()
        .or_else(|| {
            settings
                .and_then(|value| value.target.as_ref())
                .map(|value| value.as_str())
        })
        .or_else(|| {
            manifest
                .run
                .as_ref()
                .and_then(|value| value.target.as_ref())
                .map(|value| value.as_str())
        })
        .ok_or(
            "benchmark target is required; pass --target node or set benchmark.target/run.target",
        )?;
    if !matches!(target, "node" | "process" | "test-js") {
        return Err(format!(
            "benchmark target `{target}` is unsupported by the CLI process adapter; expected node"
        ));
    }
    let compiled = match seseragi_driver::compile_local_benchmarks(
        &project,
        seseragi_project::BuildProfile::Release,
    ) {
        Ok(value) => value,
        Err(error) => return crate::test::render_tool_compile_error("benchmark", &project, error),
    };
    if compiled.benchmark_modules.is_empty() {
        return Err(
            "benchmark discovery found no exported `pub let benchmarks: benchmark.Benchmark`"
                .to_owned(),
        );
    }
    let config = BenchmarkConfig {
        warmup: invocation
            .warmup
            .or_else(|| settings.map(|value| value.warmup))
            .unwrap_or(10),
        samples: invocation
            .samples
            .or_else(|| settings.map(|value| value.samples))
            .unwrap_or(50),
        minimum_sample_ms: invocation
            .minimum_sample_ms
            .or_else(|| settings.map(|value| value.minimum_sample_ms))
            .unwrap_or(100),
        regression_threshold_percent: invocation
            .threshold
            .or_else(|| settings.map(|value| value.regression_threshold_percent))
            .unwrap_or(5.0),
        seed: invocation.seed.unwrap_or(0).to_string(),
        timeout_ms: invocation.timeout_ms.unwrap_or(30_000),
        cleanup_grace_ms: invocation.cleanup_grace_ms.unwrap_or(5_000),
    };
    // Baseline paths are command-cwd paths, independent of the package cwd used
    // by benchmark bodies and their foreign adapters.
    let absolute = |value: Option<String>| -> Result<Option<String>, String> {
        value
            .map(|value| {
                let path = PathBuf::from(value);
                let path = if path.is_absolute() {
                    path
                } else {
                    std::env::current_dir()
                        .map_err(|error| error.to_string())?
                        .join(path)
                };
                Ok(path.to_string_lossy().into_owned())
            })
            .transpose()
    };
    let options = BenchmarkRunOptions {
        config,
        filter: invocation.filter,
        exact: invocation.exact,
        baseline: absolute(invocation.baseline)?,
        save_baseline: absolute(invocation.save_baseline)?,
        json: invocation.json,
        version: seseragi_release::TOOLCHAIN_VERSION.to_owned(),
    };
    run_local_benchmarks_in_directory(&compiled, &root, &options)
        .map(|result| result.exit_code)
        .map_err(|error| error.to_string())
}

#[derive(Default)]
struct Invocation {
    path: PathBuf,
    target: Option<String>,
    filter: Option<String>,
    exact: Option<String>,
    warmup: Option<u64>,
    samples: Option<u64>,
    minimum_sample_ms: Option<u64>,
    threshold: Option<f64>,
    seed: Option<i64>,
    timeout_ms: Option<u64>,
    cleanup_grace_ms: Option<u64>,
    baseline: Option<String>,
    save_baseline: Option<String>,
    json: bool,
}
impl Invocation {
    fn parse(arguments: &[String]) -> Result<Self, String> {
        let mut value = Self {
            path: Path::new(".").to_owned(),
            ..Self::default()
        };
        let mut seen = BTreeSet::new();
        let mut saw_path = false;
        let mut index = 0;
        while index < arguments.len() {
            let flag = arguments[index].as_str();
            if flag.starts_with('-')
                && !seen.insert(if flag == "--timeout" {
                    "--timeout-ms"
                } else {
                    flag
                })
            {
                return Err(format!("{flag} may only be specified once"));
            }
            if flag == "--json" {
                value.json = true;
                index += 1;
                continue;
            }
            if !flag.starts_with('-') {
                if saw_path {
                    return Err(format!("unexpected benchmark argument `{flag}`"));
                }
                saw_path = true;
                value.path = PathBuf::from(flag);
                index += 1;
                continue;
            }
            let argument = arguments
                .get(index + 1)
                .ok_or_else(|| format!("{flag} requires a value"))?;
            let number = |minimum: u64| -> Result<u64, String> {
                argument
                    .parse::<u64>()
                    .ok()
                    .filter(|n| *n >= minimum && *n <= 9_007_199_254_740_991)
                    .ok_or_else(|| format!("{flag} expects an integer >= {minimum}"))
            };
            match flag {
                "--target" => value.target = Some(argument.clone()),
                "--filter" => value.filter = Some(argument.clone()),
                "--exact" => value.exact = Some(argument.clone()),
                "--warmup" => value.warmup = Some(number(0)?),
                "--samples" => value.samples = Some(number(3)?),
                "--minimum-sample-ms" => value.minimum_sample_ms = Some(number(1)?),
                "--timeout" | "--timeout-ms" => value.timeout_ms = Some(number(1)?),
                "--cleanup-grace-ms" => value.cleanup_grace_ms = Some(number(0)?),
                "--seed" => {
                    value.seed = Some(argument.parse().map_err(|_| "--seed expects an Int64")?)
                }
                "--regression-threshold-percent" => {
                    value.threshold = Some(
                        argument
                            .parse::<f64>()
                            .ok()
                            .filter(|n| n.is_finite() && *n >= 0.0)
                            .ok_or("regression threshold must be finite and nonnegative")?,
                    )
                }
                "--baseline" => value.baseline = Some(argument.clone()),
                "--save-baseline" => value.save_baseline = Some(argument.clone()),
                _ => return Err(format!("unknown benchmark option `{flag}`")),
            }
            index += 2;
        }
        if value.filter.is_some() && value.exact.is_some() {
            return Err("--filter and --exact are mutually exclusive".to_owned());
        }
        Ok(value)
    }
}
