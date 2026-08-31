use std::path::Path;

use seseragi_driver::{compile_module, render_terminal_diagnostics, CompileInput};
use seseragi_runtime::{DiagnosticFormat, ProcessRunOptions};

mod package;

pub(crate) fn run(arguments: &[String]) -> Result<i32, String> {
    let mut path = None;
    let mut diagnostic_format = DiagnosticFormat::Human;
    let mut index = 0;
    while index < arguments.len() {
        if arguments[index] == "--diagnostic-format" {
            let value = arguments
                .get(index + 1)
                .ok_or_else(|| "--diagnostic-format requires human or json".to_owned())?;
            diagnostic_format = match value.as_str() {
                "human" => DiagnosticFormat::Human,
                "json" => DiagnosticFormat::Json,
                _ => return Err("--diagnostic-format requires human or json".to_owned()),
            };
            index += 2;
            continue;
        }
        if arguments[index].starts_with('-') || path.is_some() {
            return Err("invalid run arguments; run `seseragi --help` for usage".to_owned());
        }
        path = Some(Path::new(&arguments[index]));
        index += 1;
    }
    let path = path.ok_or_else(|| "run requires a source file or package path".to_owned())?;
    run_path(path, diagnostic_format)
}

fn run_path(path: &Path, diagnostic_format: DiagnosticFormat) -> Result<i32, String> {
    if let Some(package) = crate::local_project::containing_package(path) {
        package::run_package(&package, diagnostic_format)
    } else if path.is_dir() {
        package::run_package(path, diagnostic_format)
    } else {
        run_file(path, diagnostic_format)
    }
}

fn run_file(path: &Path, diagnostic_format: DiagnosticFormat) -> Result<i32, String> {
    if path.extension().and_then(|extension| extension.to_str()) != Some("ssrg") {
        return Err("run expects a .ssrg source file".to_owned());
    }
    let source = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let source_name = path.to_string_lossy();
    let compiled =
        match compile_module(CompileInput::new(&source_name, "single-file/main", &source)) {
            Ok(compiled) => compiled,
            Err(diagnostics) => {
                eprint!("{}", render_terminal_diagnostics(&diagnostics, &source));
                return Ok(2);
            }
        };
    if !compiled.diagnostics.diagnostics.is_empty() {
        eprint!(
            "{}",
            render_terminal_diagnostics(&compiled.diagnostics, &source)
        );
    }
    seseragi_runtime::run_main_with_options(
        &compiled,
        ProcessRunOptions {
            diagnostic_format,
            ..ProcessRunOptions::default()
        },
    )
    .map(|outcome| outcome.exit_code)
    .map_err(|error| error.to_string())
}
