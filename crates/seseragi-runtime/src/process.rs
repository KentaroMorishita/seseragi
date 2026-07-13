use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use seseragi_driver::CompiledModule;

use crate::{main_contract, FailureRenderer, HostService, MainContract};

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
    let cleanup = fs::remove_dir_all(&directory)
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
    fs::write(directory.join("entry.ts"), entry_source(contract))
        .map_err(|error| RunError::Host(format!("failed to stage runtime entry: {error}")))?;

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

fn prepare_directory() -> Result<PathBuf, String> {
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

fn entry_source(contract: &MainContract) -> String {
    let mut imports = vec!["import { run } from \"@seseragi/runtime/effect\";".to_owned()];
    let mut setup = Vec::new();
    let mut fields = Vec::new();
    let mut cleanup = Vec::new();
    let mut imports_console = false;
    let mut imports_stdin = false;
    for (index, binding) in contract.environment.iter().enumerate() {
        let field = format!("{:?}", binding.field);
        match binding.service {
            HostService::Console => {
                if !imports_console {
                    imports.push(
                        "import { liveConsole } from \"@seseragi/runtime/console\";".to_owned(),
                    );
                    imports_console = true;
                }
                fields.push(format!("{field}: liveConsole"));
            }
            HostService::Stdin => {
                if !imports_stdin {
                    imports.push(
                        "import { createProcessStdin } from \"@seseragi/runtime/stdin\";"
                            .to_owned(),
                    );
                    imports_stdin = true;
                }
                let local = format!("stdinAdapter{index}");
                setup.push(format!("const {local} = createProcessStdin();"));
                fields.push(format!("{field}: {local}"));
                cleanup.push(format!("{local}.close();"));
            }
        }
    }
    let failure = match &contract.failure_renderer {
        FailureRenderer::Never => {
            imports.push("import { main } from \"./main.ts\";".to_owned());
            "process.stderr.write(\"seseragi: unreachable typed failure\\n\");\n  process.exitCode = 1;".to_owned()
        }
        FailureRenderer::Show { module, export } => {
            if module == "./main.ts" {
                imports.push(format!(
                    "import {{ main, {export} as failureShow }} from \"./main.ts\";"
                ));
            } else {
                imports.push("import { main } from \"./main.ts\";".to_owned());
                imports.push(format!(
                    "import {{ {export} as failureShow }} from \"{module}\";"
                ));
            }
            "const message = failureShow.show(result.error);\n  if (typeof message !== \"string\") throw new TypeError(\"Show dictionary returned a non-string value\");\n  process.stderr.write(message.endsWith(\"\\n\") ? message : message + \"\\n\");\n  process.exitCode = 1;".to_owned()
        }
    };
    let cleanup_source = cleanup
        .iter()
        .map(|line| format!("    {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "{}\n{}\nconst environment = {{ {} }};\nlet result;\nlet hasRuntimeDefect = false;\ntry {{\n  result = await run(main(undefined), environment);\n}} catch (_runDefect) {{\n  hasRuntimeDefect = true;\n}}\ntry {{\n{}\n}} catch (_cleanupDefect) {{\n  hasRuntimeDefect = true;\n}}\nif (hasRuntimeDefect) {{\n  process.stderr.write(\"seseragi: runtime defect\\n\");\n  process.exitCode = 70;\n}} else if (result?.kind === \"failure\") {{\n  {}\n}}\n",
        imports.join("\n"),
        setup.join("\n"),
        fields.join(", "),
        cleanup_source,
        failure,
    )
}

#[cfg(test)]
mod tests {
    use super::entry_source;
    use crate::{EnvironmentBinding, FailureRenderer, HostService, MainContract};

    #[test]
    fn prepares_live_process_services_and_typed_failure_rendering() {
        let source = entry_source(&MainContract {
            environment: vec![
                EnvironmentBinding {
                    field: "console".to_owned(),
                    service: HostService::Console,
                },
                EnvironmentBinding {
                    field: "stdin".to_owned(),
                    service: HostService::Stdin,
                },
            ],
            failure_renderer: FailureRenderer::Show {
                module: "./main.ts".to_owned(),
                export: "__ssrg$instance$Show$0".to_owned(),
            },
        });

        assert!(source.contains("liveConsole"));
        assert!(source.contains("createProcessStdin"));
        assert!(source.contains("await run(main(undefined), environment)"));
        assert!(source.contains("failureShow.show(result.error)"));
        assert!(source.contains("stdinAdapter1.close()"));
        assert!(source.contains("catch (_cleanupDefect)"));
        assert!(
            source.find("stdinAdapter1.close()").unwrap()
                < source.find("failureShow.show(result.error)").unwrap()
        );
    }

    #[test]
    fn imports_each_host_adapter_once_for_multiple_service_fields() {
        let source = entry_source(&MainContract {
            environment: vec![
                EnvironmentBinding {
                    field: "first".to_owned(),
                    service: HostService::Console,
                },
                EnvironmentBinding {
                    field: "second".to_owned(),
                    service: HostService::Console,
                },
            ],
            failure_renderer: FailureRenderer::Never,
        });

        assert_eq!(source.matches("import { liveConsole }").count(), 1);
        assert!(source.contains("\"first\": liveConsole"));
        assert!(source.contains("\"second\": liveConsole"));
    }
}
