use std::path::Path;

use seseragi_driver::{compile_module, render_terminal_diagnostics, CompileInput};

pub(crate) fn run_file(path: &Path) -> Result<i32, String> {
    if path.is_dir() {
        return Err(
            "package directory execution is not available yet; pass one .ssrg file".to_owned(),
        );
    }
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
    seseragi_runtime::run_main(&compiled)
        .map(|outcome| outcome.exit_code)
        .map_err(|error| error.to_string())
}
