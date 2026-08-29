use seseragi_driver::{
    compile_local_tests, render_terminal_diagnostics, LinkedCompileError, LocalTestCompileError,
    ProjectCompileError,
};
use seseragi_runtime::{run_local_tests_in_directory, TestRunOptions};
use std::path::{Path, PathBuf};

pub(crate) fn test(arguments: &[String]) -> Result<i32, String> {
    let invocation = Invocation::parse(arguments)?;
    let root = crate::local_project::containing_package(&invocation.path)
        .unwrap_or_else(|| invocation.path.clone());
    if !root.join("seseragi.toml").is_file() {
        return Err(format!(
            "test expects a package containing seseragi.toml: {}",
            root.display()
        ));
    }
    seseragi_project::read_and_validate_lockfile(&root)
        .map_err(|error| format!("{}: {error}", error.code()))?;
    let project = seseragi_project::load_local_tests(&root)
        .map_err(|error| format!("{}: {error}", error.code()))?;
    let manifest = project
        .packages()
        .package(project.packages().root())
        .expect("local test project contains root manifest")
        .manifest();
    let settings = manifest.test.as_ref();
    let target = invocation
        .target
        .as_deref()
        .or_else(|| {
            settings
                .and_then(|test| test.target.as_ref())
                .map(|value| value.as_str())
        })
        .or_else(|| {
            manifest
                .run
                .as_ref()
                .and_then(|run| run.target.as_ref())
                .map(|value| value.as_str())
        })
        .ok_or_else(|| {
            "test target is required; pass `--target node` or set test.target/run.target".to_owned()
        })?;
    if !matches!(target, "node" | "process" | "test-js") {
        return Err(format!(
            "test target `{target}` is unsupported; expected `node`"
        ));
    }
    let compiled = match compile_local_tests(&project) {
        Ok(compiled) => compiled,
        Err(error) => return render_compile_error(&project, error),
    };
    if compiled.test_modules.is_empty() {
        return Err("test discovery found no .ssrg modules under the test root".to_owned());
    }
    let options = TestRunOptions {
        filter: invocation.filter,
        exact: invocation.exact,
        jobs: invocation
            .jobs
            .or_else(|| settings.map(|test| test.jobs))
            .unwrap_or(1),
        timeout_ms: invocation
            .timeout_ms
            .or_else(|| settings.map(|test| test.timeout_ms))
            .unwrap_or(30_000),
        cleanup_grace_ms: settings.map(|test| test.cleanup_grace_ms).unwrap_or(5_000),
        seed: invocation
            .seed
            .or_else(|| settings.map(|test| test.seed))
            .unwrap_or(0),
    };
    run_local_tests_in_directory(&compiled, &root, &options)
        .map(|outcome| outcome.exit_code)
        .map_err(|error| error.to_string())
}

fn render_compile_error(
    project: &seseragi_project::LoadedLocalTests,
    error: LocalTestCompileError,
) -> Result<i32, String> {
    match error {
        LocalTestCompileError::Discovery { module, reason } => {
            Err(format!("test discovery failed for {module}: {reason}"))
        }
        LocalTestCompileError::Compile(error) => {
            let diagnostics = match error.error() {
                ProjectCompileError::Diagnostics { modules } => {
                    modules.first().map(|entry| &entry.diagnostics)
                }
                ProjectCompileError::Compile {
                    error: LinkedCompileError::Diagnostics(diagnostics),
                    ..
                } => Some(diagnostics),
                _ => None,
            };
            if let (Some(identity), Some(diagnostics)) = (error.module(), diagnostics) {
                let module = project
                    .module(identity)
                    .expect("diagnostic identity belongs to test project");
                eprint!(
                    "{}",
                    render_terminal_diagnostics(diagnostics, module.source())
                );
                return Ok(2);
            }
            Err(format!(
                "test compiler rejected package: {:?}",
                error.error()
            ))
        }
    }
}

struct Invocation {
    path: PathBuf,
    filter: Option<String>,
    exact: Option<String>,
    jobs: Option<usize>,
    timeout_ms: Option<u64>,
    seed: Option<i64>,
    target: Option<String>,
}

impl Invocation {
    fn parse(arguments: &[String]) -> Result<Self, String> {
        let mut invocation = Self {
            path: Path::new(".").to_owned(),
            filter: None,
            exact: None,
            jobs: None,
            timeout_ms: None,
            seed: None,
            target: None,
        };
        let mut saw_path = false;
        let mut index = 0;
        while index < arguments.len() {
            let argument = &arguments[index];
            let value = |flag: &str| -> Result<&str, String> {
                arguments
                    .get(index + 1)
                    .map(String::as_str)
                    .ok_or_else(|| format!("{flag} requires a value"))
            };
            let consumed = match argument.as_str() {
                "--filter" => {
                    invocation.filter = Some(value("--filter")?.to_owned());
                    2
                }
                "--exact" => {
                    invocation.exact = Some(value("--exact")?.to_owned());
                    2
                }
                "--jobs" => {
                    invocation.jobs = Some(parse_positive(value("--jobs")?, "--jobs")?);
                    2
                }
                "--timeout" | "--timeout-ms" => {
                    invocation.timeout_ms =
                        Some(parse_positive(value(argument)?, argument)? as u64);
                    2
                }
                "--seed" => {
                    invocation.seed = Some(
                        value("--seed")?
                            .parse::<i64>()
                            .map_err(|_| "--seed expects an Int".to_owned())?,
                    );
                    2
                }
                "--target" => {
                    invocation.target = Some(value("--target")?.to_owned());
                    2
                }
                value if value.starts_with('-') => {
                    return Err(format!("unknown test option `{value}`"));
                }
                value if !saw_path => {
                    invocation.path = Path::new(value).to_owned();
                    saw_path = true;
                    1
                }
                value => return Err(format!("unexpected test argument `{value}`")),
            };
            index += consumed;
        }
        if invocation.filter.is_some() && invocation.exact.is_some() {
            return Err("--filter and --exact are mutually exclusive".to_owned());
        }
        Ok(invocation)
    }
}

fn parse_positive(value: &str, flag: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| format!("{flag} expects a positive integer"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_the_test_discovery_product_fixture() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("examples/spec/fixtures/projects/test-discovery")
            .canonicalize()
            .unwrap();
        let arguments = vec![root.to_string_lossy().into_owned()];

        assert_eq!(test(&arguments).unwrap(), 0);
    }

    #[test]
    fn rejects_ambiguous_selection_and_non_positive_jobs() {
        assert!(Invocation::parse(&[
            "--filter".to_owned(),
            "one".to_owned(),
            "--exact".to_owned(),
            "two".to_owned(),
        ])
        .is_err());
        assert!(Invocation::parse(&["--jobs".to_owned(), "0".to_owned()]).is_err());
    }
}
