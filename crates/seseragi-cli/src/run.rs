use std::path::{Path, PathBuf};

use seseragi_driver::{compile_module, CompileInput};
use seseragi_project::{select_project_target, ProjectCommand, ProjectTarget};
use seseragi_runtime::{DiagnosticFormat, ProcessRunOptions, ProcessSignalMode, RandomSeed};

use crate::diagnostics::{render_diagnostics, DiagnosticDocument};

mod package;

pub(crate) fn run(arguments: &[String]) -> Result<i32, String> {
    let invocation = Invocation::parse(arguments)?;
    run_path(&invocation)
}

fn run_path(invocation: &Invocation) -> Result<i32, String> {
    if let Some(package) = crate::local_project::containing_package(&invocation.path) {
        package::run_package(&package, invocation)
    } else if invocation.path.is_dir() {
        package::run_package(&invocation.path, invocation)
    } else {
        run_file(invocation)
    }
}

fn run_file(invocation: &Invocation) -> Result<i32, String> {
    let path = &invocation.path;
    if path.extension().and_then(|extension| extension.to_str()) != Some("ssrg") {
        return Err("run expects a .ssrg source file".to_owned());
    }
    select_project_target(ProjectCommand::Run, invocation.target, None, None)
        .map_err(|error| error.to_string())?;
    let source = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let source_name = path.to_string_lossy();
    let format = invocation.diagnostic_format();
    let compiled =
        match compile_module(CompileInput::new(&source_name, "single-file/main", &source)) {
            Ok(compiled) => compiled,
            Err(diagnostics) => {
                eprint!(
                    "{}",
                    render_diagnostics(
                        format,
                        &[DiagnosticDocument {
                            path: &source_name,
                            source: &source,
                            artifact: &diagnostics,
                        }],
                    )?
                );
                return Ok(2);
            }
        };
    if !compiled.diagnostics.diagnostics.is_empty() {
        eprint!(
            "{}",
            render_diagnostics(
                format,
                &[DiagnosticDocument {
                    path: &source_name,
                    source: &source,
                    artifact: &compiled.diagnostics,
                }],
            )?
        );
    }
    let options = invocation.apply(ProcessRunOptions::default())?;
    seseragi_runtime::run_main_with_options(&compiled, options)
        .map(|outcome| outcome.exit_code)
        .map_err(|error| error.to_string())
}

pub(super) struct Invocation {
    path: PathBuf,
    target: Option<ProjectTarget>,
    diagnostic_format: Option<DiagnosticFormat>,
    signal_mode: Option<ProcessSignalMode>,
    shutdown_grace_ms: Option<u64>,
    hash_seed: Option<RandomSeed>,
    random_seed: Option<RandomSeed>,
}

impl Invocation {
    fn parse(arguments: &[String]) -> Result<Self, String> {
        let mut invocation = Self {
            path: PathBuf::new(),
            target: None,
            diagnostic_format: None,
            signal_mode: None,
            shutdown_grace_ms: None,
            hash_seed: None,
            random_seed: None,
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
                "--target" => {
                    set_once(
                        &mut invocation.target,
                        ProjectTarget::parse(value("--target")?)
                            .map_err(|error| error.to_string())?,
                        "--target",
                    )?;
                    2
                }
                "--diagnostic-format" => {
                    let format = match value("--diagnostic-format")? {
                        "text" => DiagnosticFormat::Text,
                        "json" => DiagnosticFormat::Json,
                        _ => return Err("--diagnostic-format expects `text` or `json`".to_owned()),
                    };
                    set_once(
                        &mut invocation.diagnostic_format,
                        format,
                        "--diagnostic-format",
                    )?;
                    2
                }
                "--signal-mode" => {
                    let mode = match value("--signal-mode")? {
                        "cancel" => ProcessSignalMode::Cancel,
                        "forward" => ProcessSignalMode::Forward,
                        _ => return Err("--signal-mode expects `cancel` or `forward`".to_owned()),
                    };
                    set_once(&mut invocation.signal_mode, mode, "--signal-mode")?;
                    2
                }
                "--shutdown-grace-ms" => {
                    let grace = value("--shutdown-grace-ms")?.parse::<u64>().map_err(|_| {
                        "--shutdown-grace-ms expects a non-negative integer".to_owned()
                    })?;
                    set_once(
                        &mut invocation.shutdown_grace_ms,
                        grace,
                        "--shutdown-grace-ms",
                    )?;
                    2
                }
                "--hash-seed" => {
                    let seed = parse_seed(value("--hash-seed")?, "--hash-seed")?;
                    set_once(&mut invocation.hash_seed, seed, "--hash-seed")?;
                    2
                }
                "--random-seed" => {
                    let seed = parse_seed(value("--random-seed")?, "--random-seed")?;
                    set_once(&mut invocation.random_seed, seed, "--random-seed")?;
                    2
                }
                value if value.starts_with('-') => {
                    return Err(format!("unknown run option `{value}`"));
                }
                value if !saw_path => {
                    invocation.path = Path::new(value).to_owned();
                    saw_path = true;
                    1
                }
                value => return Err(format!("unexpected run argument `{value}`")),
            };
            index += consumed;
        }
        if !saw_path {
            return Err("run requires a source file or package path".to_owned());
        }
        if invocation.signal_mode == Some(ProcessSignalMode::Forward)
            && invocation.shutdown_grace_ms.is_some()
        {
            return Err("--shutdown-grace-ms is only valid with --signal-mode cancel".to_owned());
        }
        Ok(invocation)
    }

    pub(super) fn target(&self) -> Option<ProjectTarget> {
        self.target
    }

    pub(super) fn diagnostic_format(&self) -> DiagnosticFormat {
        self.diagnostic_format.unwrap_or(DiagnosticFormat::Text)
    }

    pub(super) fn apply(
        &self,
        mut options: ProcessRunOptions,
    ) -> Result<ProcessRunOptions, String> {
        if let Some(mode) = self.signal_mode {
            options.signal_mode = mode;
        }
        if let Some(grace) = self.shutdown_grace_ms {
            if options.signal_mode == ProcessSignalMode::Forward {
                return Err(
                    "--shutdown-grace-ms is only valid when the effective signal mode is cancel"
                        .to_owned(),
                );
            }
            options.shutdown_grace_ms = grace;
        }
        if let Some(seed) = self.hash_seed {
            options.hash_seed = seed;
        }
        if let Some(seed) = self.random_seed {
            options.random_seed = seed;
        }
        options.diagnostic_format = self.diagnostic_format();
        Ok(options)
    }
}

fn parse_seed(value: &str, flag: &str) -> Result<RandomSeed, String> {
    if value == "entropy" {
        Ok(RandomSeed::Entropy)
    } else {
        value
            .parse::<i64>()
            .map(RandomSeed::Fixed)
            .map_err(|_| format!("{flag} expects `entropy` or a signed 64-bit integer"))
    }
}

fn set_once<T>(slot: &mut Option<T>, value: T, flag: &str) -> Result<(), String> {
    if slot.is_some() {
        return Err(format!("{flag} may only be specified once"));
    }
    *slot = Some(value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_options_on_both_sides_of_the_path() {
        let invocation = Invocation::parse(&[
            "--target".to_owned(),
            "process".to_owned(),
            "app.ssrg".to_owned(),
            "--diagnostic-format".to_owned(),
            "json".to_owned(),
            "--signal-mode".to_owned(),
            "cancel".to_owned(),
            "--shutdown-grace-ms".to_owned(),
            "0".to_owned(),
            "--hash-seed".to_owned(),
            "-7".to_owned(),
            "--random-seed".to_owned(),
            "entropy".to_owned(),
        ])
        .unwrap();
        let options = invocation.apply(ProcessRunOptions::default()).unwrap();

        assert_eq!(invocation.target(), Some(ProjectTarget::Process));
        assert_eq!(options.diagnostic_format, DiagnosticFormat::Json);
        assert_eq!(options.signal_mode, ProcessSignalMode::Cancel);
        assert_eq!(options.shutdown_grace_ms, 0);
        assert_eq!(options.hash_seed, RandomSeed::Fixed(-7));
        assert_eq!(options.random_seed, RandomSeed::Entropy);
    }

    #[test]
    fn rejects_unknown_duplicate_and_incompatible_options() {
        for arguments in [
            vec!["app.ssrg", "--unknown"],
            vec!["app.ssrg", "--target", "process", "--target", "process"],
            vec!["app.ssrg", "--diagnostic-format", "human"],
            vec![
                "app.ssrg",
                "--signal-mode",
                "forward",
                "--shutdown-grace-ms",
                "1",
            ],
        ] {
            let arguments = arguments.into_iter().map(str::to_owned).collect::<Vec<_>>();
            assert!(Invocation::parse(&arguments).is_err());
        }
    }
}
