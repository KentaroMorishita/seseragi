use std::path::Path;

use crate::local_project::{compatible_targets, compile_path, LocalProjectCompilation};
use seseragi_driver::{compile_module, render_terminal_diagnostics, CompileInput};
use seseragi_project::{select_project_target, ProjectCommand, ProjectTarget};
use seseragi_runtime::{main_contract, BuildTarget};

pub(crate) fn build(arguments: &[String]) -> Result<i32, String> {
    let mut path = None;
    let mut output_directory = "dist".to_owned();
    let mut target = None;
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
                target = Some(match arguments.get(index).map(String::as_str) {
                    Some(value) => {
                        ProjectTarget::parse(value).map_err(|error| error.to_string())?
                    }
                    None => return Err("--target requires `process` or `web`".to_owned()),
                });
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
    target: Option<ProjectTarget>,
) -> Result<i32, String> {
    if path.is_dir() {
        build_package(path, output_directory, target)
    } else {
        build_file(path, output_directory, target)
    }
}

fn build_file(
    path: &Path,
    output_directory: &Path,
    target: Option<ProjectTarget>,
) -> Result<i32, String> {
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
    let contract =
        main_contract(&compiled).map_err(|error| format!("invalid entry point: {error}"))?;
    let compatible = compatible_targets(&contract);
    let selection = select_project_target(
        ProjectCommand::Build,
        target,
        None,
        target.is_none().then_some(compatible.as_slice()),
    )
    .map_err(|error| error.to_string())?;
    seseragi_runtime::build_main(&compiled, output_directory, build_target(selection.target))
        .map_err(|error| error.to_string())?;
    println!("Built {} -> {}", path.display(), output_directory.display());
    Ok(0)
}

fn build_package(
    path: &Path,
    output_directory: &Path,
    target: Option<ProjectTarget>,
) -> Result<i32, String> {
    let compiled = match compile_path(path, ProjectCommand::Build, target)? {
        LocalProjectCompilation::Compiled(compiled) => compiled,
        LocalProjectCompilation::Diagnostics => return Ok(2),
    };
    seseragi_runtime::build_local_project(
        &compiled.compiled,
        output_directory,
        build_target(compiled.target),
    )
    .map_err(|error| error.to_string())?;
    println!("Built {} -> {}", path.display(), output_directory.display());
    Ok(0)
}

fn build_target(target: ProjectTarget) -> BuildTarget {
    match target {
        ProjectTarget::Process => BuildTarget::Process,
        ProjectTarget::Web => BuildTarget::Web,
    }
}
