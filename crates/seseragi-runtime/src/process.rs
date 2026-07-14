use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use seseragi_driver::CompiledModule;

use crate::{main_contract, MainContract};

mod entry;
mod local_package;

use entry::entry_source;
pub use local_package::{run_local_package, run_local_project};

static NEXT_RUN: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RunOutcome {
    pub exit_code: i32,
}

#[derive(Debug)]
pub enum RunError {
    InvalidEntry(String),
    Host(String),
}

impl std::fmt::Display for RunError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidEntry(message) => write!(formatter, "invalid entry point: {message}"),
            Self::Host(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for RunError {}

/// Runs a compiled single-file `main` with the process Console and Stdin.
/// Child standard streams are inherited, so this is interactive as well as
/// suitable for subprocess integration tests.
pub fn run_main(compiled: &CompiledModule) -> Result<RunOutcome, RunError> {
    let contract = main_contract(compiled).map_err(RunError::InvalidEntry)?;
    let directory = prepare_directory().map_err(RunError::Host)?;
    let result = run_in_directory(compiled, &contract, &directory);
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
) -> Result<RunOutcome, RunError> {
    fs::write(directory.join("main.ts"), &compiled.generated.typescript).map_err(|error| {
        RunError::Host(format!("failed to stage generated TypeScript: {error}"))
    })?;
    crate::stage_typescript_package(directory).map_err(RunError::Host)?;
    fs::write(
        directory.join("entry.ts"),
        entry_source(contract, "./main.ts"),
    )
    .map_err(|error| RunError::Host(format!("failed to stage runtime entry: {error}")))?;

    run_target(directory)
}

pub(super) fn run_target(directory: &Path) -> Result<RunOutcome, RunError> {
    let status = Command::new("bun")
        .arg("run")
        .arg("entry.ts")
        .current_dir(directory)
        .status()
        .map_err(|error| RunError::Host(format!("failed to launch Bun target adapter: {error}")))?;
    let exit_code = status.code().ok_or_else(|| {
        RunError::Host("Bun target adapter terminated without an exit code".to_owned())
    })?;
    Ok(RunOutcome { exit_code })
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
