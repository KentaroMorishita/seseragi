use std::path::Path;

use crate::local_project::{compile_path, LocalProjectCompilation};
use seseragi_driver::{compile_module, render_terminal_diagnostics, CompileInput};
use seseragi_runtime::BuildTarget;

pub(crate) fn build(arguments: &[String]) -> Result<i32, String> {
    let mut path = None;
    let mut output_directory = "dist".to_owned();
    let mut target = BuildTarget::Process;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--out-dir" => {
                index += 1;
                output_directory = arguments
                    .get(index)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| "--out-dir requires a directory".to_owned())?
                    .clone();
            }
            "--target" => {
                index += 1;
                target = match arguments.get(index).map(String::as_str) {
                    Some("process") => BuildTarget::Process,
                    Some("web") => BuildTarget::Web,
                    Some(value) => {
                        return Err(format!(
                            "unsupported build target `{value}`; expected `process` or `web`"
                        ));
                    }
                    None => return Err("--target requires `process` or `web`".to_owned()),
                };
            }
            argument if argument.starts_with('-') => {
                return Err(format!("unknown build option `{argument}`"));
            }
            argument if path.is_none() => path = Some(argument.to_owned()),
            argument => return Err(format!("unexpected build argument `{argument}`")),
        }
        index += 1;
    }
    let path = path.ok_or_else(|| "build requires a source file or package path".to_owned())?;
    build_path(Path::new(&path), Path::new(&output_directory), target)
}

pub(crate) fn build_path(
    path: &Path,
    output_directory: &Path,
    target: BuildTarget,
) -> Result<i32, String> {
    if path.is_dir() {
        build_package(path, output_directory, target)
    } else {
        build_file(path, output_directory, target)
    }
}

fn build_file(path: &Path, output_directory: &Path, target: BuildTarget) -> Result<i32, String> {
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
    seseragi_runtime::build_main(&compiled, output_directory, target)
        .map_err(|error| error.to_string())?;
    println!("Built {} -> {}", path.display(), output_directory.display());
    Ok(0)
}

fn build_package(path: &Path, output_directory: &Path, target: BuildTarget) -> Result<i32, String> {
    let compiled = match compile_path(path)? {
        LocalProjectCompilation::Compiled(compiled) => compiled,
        LocalProjectCompilation::Diagnostics => return Ok(2),
    };
    seseragi_runtime::build_local_project(&compiled, output_directory, target)
        .map_err(|error| error.to_string())?;
    println!("Built {} -> {}", path.display(), output_directory.display());
    Ok(0)
}
