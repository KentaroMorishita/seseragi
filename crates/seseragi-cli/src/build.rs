use std::path::Path;

use crate::local_project::{compile_path, LocalProjectCompilation};
use seseragi_driver::{compile_module, render_terminal_diagnostics, CompileInput};

pub(crate) fn build_path(path: &Path, output_directory: &Path) -> Result<i32, String> {
    if path.is_dir() {
        build_package(path, output_directory)
    } else {
        build_file(path, output_directory)
    }
}

fn build_file(path: &Path, output_directory: &Path) -> Result<i32, String> {
    if path.extension().and_then(|extension| extension.to_str()) != Some("ssrg") {
        return Err("build expects a .ssrg source file".to_owned());
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
    seseragi_runtime::build_main(&compiled, output_directory).map_err(|error| error.to_string())?;
    println!("Built {} -> {}", path.display(), output_directory.display());
    Ok(0)
}

fn build_package(path: &Path, output_directory: &Path) -> Result<i32, String> {
    let compiled = match compile_path(path)? {
        LocalProjectCompilation::Compiled(compiled) => compiled,
        LocalProjectCompilation::Diagnostics => return Ok(2),
    };
    seseragi_runtime::build_local_project(&compiled, output_directory)
        .map_err(|error| error.to_string())?;
    println!("Built {} -> {}", path.display(), output_directory.display());
    Ok(0)
}
