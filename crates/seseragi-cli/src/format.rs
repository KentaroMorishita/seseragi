use std::path::Path;

use seseragi_driver::{format_module, render_terminal_diagnostics};

#[derive(Clone, Copy)]
pub(crate) enum FormatMode {
    Write,
    Check,
}

pub(crate) fn format_file(path: &Path, mode: FormatMode) -> Result<i32, String> {
    if path.is_dir() {
        return Err("format expects one .ssrg source file".to_owned());
    }
    if path.extension().and_then(|extension| extension.to_str()) != Some("ssrg") {
        return Err("format expects a .ssrg source file".to_owned());
    }
    let source = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let source_name = path.to_string_lossy();
    let formatted = match format_module(&source_name, &source) {
        Ok(formatted) => formatted,
        Err(diagnostics) => {
            eprint!("{}", render_terminal_diagnostics(&diagnostics, &source));
            return Ok(2);
        }
    };

    match mode {
        FormatMode::Check if formatted.changed => {
            eprintln!("{} is not canonically formatted", path.display());
            Ok(1)
        }
        FormatMode::Check => Ok(0),
        FormatMode::Write if formatted.changed => {
            std::fs::write(path, formatted.text)
                .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
            Ok(0)
        }
        FormatMode::Write => Ok(0),
    }
}
