use std::path::Path;

use seseragi_driver::{compile_module, render_terminal_diagnostics, CompileInput};

fn main() {
    let exit = match run(std::env::args().skip(1)) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("seseragi: {error}");
            2
        }
    };
    std::process::exit(exit);
}

fn run(arguments: impl IntoIterator<Item = String>) -> Result<i32, String> {
    let arguments = arguments.into_iter().collect::<Vec<_>>();
    match arguments.as_slice() {
        [command, path] if command == "run" => run_file(Path::new(path)),
        [flag] if flag == "--help" || flag == "-h" => {
            print_usage();
            Ok(0)
        }
        _ => Err("usage: seseragi run path/to/app.ssrg".to_owned()),
    }
}

fn run_file(path: &Path) -> Result<i32, String> {
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

fn print_usage() {
    println!("Usage: seseragi run path/to/app.ssrg");
}
