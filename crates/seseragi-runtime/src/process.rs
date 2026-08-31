use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use seseragi_driver::CompiledModule;

use crate::{main_contract, validate_target, ExecutionTarget, MainContract, TargetMismatch};

mod entry;
mod local_package;
mod web_entry;

pub use build::{
    build_local_project, build_local_project_with_options, build_main, build_main_with_options,
    BuildError, BuildTarget,
};
use entry::entry_source;
pub use local_package::{
    run_document_entry_in_directory, run_local_package, run_local_package_with_options,
    run_local_project, run_local_project_in_directory_with_options, run_local_project_with_options,
    run_local_tests_in_directory, TestRunOptions,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessSignalMode {
    Cancel,
    Forward,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RandomSeed {
    Entropy,
    Fixed(i64),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticFormat {
    Human,
    Json,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProcessRunOptions {
    pub signal_mode: ProcessSignalMode,
    pub shutdown_grace_ms: u64,
    pub random_seed: RandomSeed,
    pub diagnostic_format: DiagnosticFormat,
}

impl Default for ProcessRunOptions {
    fn default() -> Self {
        Self {
            signal_mode: ProcessSignalMode::Cancel,
            shutdown_grace_ms: 10_000,
            random_seed: RandomSeed::Entropy,
            diagnostic_format: DiagnosticFormat::Human,
        }
    }
}

mod build;
static NEXT_RUN: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RunOutcome {
    pub exit_code: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapturedRunOutcome {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug)]
pub enum RunError {
    InvalidEntry(String),
    TargetMismatch(TargetMismatch),
    Host(String),
}

impl std::fmt::Display for RunError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidEntry(message) => write!(formatter, "invalid entry point: {message}"),
            Self::TargetMismatch(mismatch) => mismatch.fmt(formatter),
            Self::Host(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for RunError {}

/// Runs a compiled single-file `main` with the process Console and Stdin.
/// Child standard streams are inherited, so this is interactive as well as
/// suitable for subprocess integration tests.
pub fn run_main(compiled: &CompiledModule) -> Result<RunOutcome, RunError> {
    run_main_with_options(compiled, ProcessRunOptions::default())
}

pub fn run_main_with_options(
    compiled: &CompiledModule,
    options: ProcessRunOptions,
) -> Result<RunOutcome, RunError> {
    let contract = main_contract(compiled).map_err(RunError::InvalidEntry)?;
    validate_target(&contract, ExecutionTarget::Process).map_err(RunError::TargetMismatch)?;
    let directory = prepare_directory().map_err(RunError::Host)?;
    let result = run_in_directory(compiled, &contract, &directory, options);
    finish_run(result, &directory)
}

pub(super) fn finish_run(
    result: Result<RunOutcome, RunError>,
    directory: &Path,
) -> Result<RunOutcome, RunError> {
    let cleanup = fs::remove_dir_all(directory)
        .map_err(|error| RunError::Host(format!("failed to clean execution directory: {error}")));
    match (result, cleanup) {
        (Ok(outcome), Ok(())) => Ok(outcome),
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
    }
}

fn run_in_directory(
    compiled: &CompiledModule,
    contract: &MainContract,
    directory: &Path,
    options: ProcessRunOptions,
) -> Result<RunOutcome, RunError> {
    stage_main_program(compiled, contract, directory, options).map_err(RunError::Host)?;

    run_target(directory, None)
}

fn stage_main_program(
    compiled: &CompiledModule,
    contract: &MainContract,
    directory: &Path,
    options: ProcessRunOptions,
) -> Result<(), String> {
    stage_main_module(compiled, directory)?;
    fs::write(
        directory.join("entry.ts"),
        entry_source(contract, "./main.ts", None, options),
    )
    .map_err(|error| format!("failed to stage runtime entry: {error}"))
}

fn stage_main_module(compiled: &CompiledModule, directory: &Path) -> Result<(), String> {
    fs::write(directory.join("main.ts"), &compiled.generated.typescript)
        .map_err(|error| format!("failed to stage generated TypeScript: {error}"))?;
    crate::stage_typescript_package(directory)
}

pub(super) fn run_target(
    directory: &Path,
    application_directory: Option<&Path>,
) -> Result<RunOutcome, RunError> {
    let entry = directory.join("entry.ts");
    let working_directory = application_directory.unwrap_or(directory);
    let mut command = Command::new("bun");
    command.arg("run").arg(entry).current_dir(working_directory);
    if let Some(application_directory) = application_directory {
        command.env("SESERAGI_APPLICATION_ROOT", application_directory);
    }
    let status = command
        .status()
        .map_err(|error| RunError::Host(format!("failed to launch Bun target adapter: {error}")))?;
    let exit_code = status.code().ok_or_else(|| {
        RunError::Host("Bun target adapter terminated without an exit code".to_owned())
    })?;
    Ok(RunOutcome { exit_code })
}

pub(super) fn run_target_captured(
    directory: &Path,
    application_directory: &Path,
) -> Result<CapturedRunOutcome, RunError> {
    let output = Command::new("bun")
        .arg("run")
        .arg(directory.join("entry.ts"))
        .current_dir(application_directory)
        .env("SESERAGI_APPLICATION_ROOT", application_directory)
        .output()
        .map_err(|error| RunError::Host(format!("failed to launch Bun target adapter: {error}")))?;
    let exit_code = output.status.code().ok_or_else(|| {
        RunError::Host("Bun target adapter terminated without an exit code".to_owned())
    })?;
    Ok(CapturedRunOutcome {
        exit_code,
        stdout: String::from_utf8(output.stdout)
            .map_err(|_| RunError::Host("documentation stdout was not UTF-8".to_owned()))?,
        stderr: String::from_utf8(output.stderr)
            .map_err(|_| RunError::Host("documentation stderr was not UTF-8".to_owned()))?,
    })
}

pub(super) fn prepare_directory() -> Result<PathBuf, String> {
    let run = NEXT_RUN.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir()
        .join("seseragi-run")
        .join(format!("{}-{run}", std::process::id()));
    if directory.exists() {
        fs::remove_dir_all(&directory)
            .map_err(|error| format!("failed to reset execution directory: {error}"))?;
    }
    fs::create_dir_all(&directory)
        .map_err(|error| format!("failed to create execution directory: {error}"))?;
    Ok(directory)
}
