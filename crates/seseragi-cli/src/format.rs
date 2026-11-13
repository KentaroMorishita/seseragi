use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use seseragi_driver::{format_module, render_terminal_diagnostics};

mod ignore;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum FormatMode {
    Write,
    Check,
}

pub(crate) fn format(arguments: &[String]) -> Result<i32, String> {
    let mut mode = FormatMode::Write;
    let mut paths = Vec::new();
    for argument in arguments {
        match argument.as_str() {
            "--check" => mode = FormatMode::Check,
            "-" => paths.push(argument.clone()),
            flag if flag.starts_with('-') => {
                return Err(format!("unknown format option `{flag}`"));
            }
            _ => paths.push(argument.clone()),
        }
    }
    if paths.iter().any(|path| path == "-") {
        if paths.len() != 1 {
            return Err("format reads standard input only when `-` is the only path".to_owned());
        }
        return format_stdin(mode);
    }
    let roots = if paths.is_empty() {
        let current = std::env::current_dir()
            .map_err(|error| format!("failed to read the current directory: {error}"))?;
        let package = crate::local_project::containing_package(&current).ok_or_else(|| {
            "format needs a path when the current directory is not inside a package".to_owned()
        })?;
        vec![package]
    } else {
        paths.into_iter().map(PathBuf::from).collect()
    };
    let files = collect_sources(&roots)?;
    let mut status = 0;
    for file in &files {
        status = status.max(format_file(file, mode));
    }
    Ok(status)
}

/// Collects `.ssrg` files from explicit files, directories and packages.
/// Directories skip generated and dependency folders and `.gitignore` entries.
fn collect_sources(roots: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for root in roots {
        let metadata = std::fs::metadata(root)
            .map_err(|error| format!("failed to read {}: {error}", root.display()))?;
        if metadata.is_dir() {
            let mut found = Vec::new();
            ignore::walk(root, &mut found)?;
            found.sort();
            files.extend(found);
        } else if root.extension().and_then(|extension| extension.to_str()) == Some("ssrg") {
            files.push(root.clone());
        } else {
            return Err(format!(
                "format expects .ssrg source files, directories or packages: {}",
                root.display()
            ));
        }
    }
    let mut seen = std::collections::BTreeSet::new();
    files.retain(|file| seen.insert(file.clone()));
    Ok(files)
}

/// Formats one file and returns its exit status: 0 done, 1 needs formatting
/// in check mode, 2 failed. Failures are reported and do not stop the run.
fn format_file(path: &Path, mode: FormatMode) -> i32 {
    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("seseragi: failed to read {}: {error}", path.display());
            return 2;
        }
    };
    let source_name = path.to_string_lossy();
    let formatted = match format_module(&source_name, &source) {
        Ok(formatted) => formatted,
        Err(diagnostics) => {
            eprint!("{}", render_terminal_diagnostics(&diagnostics, &source));
            return 2;
        }
    };
    match mode {
        FormatMode::Check if formatted.changed => {
            eprintln!("{} is not canonically formatted", path.display());
            1
        }
        FormatMode::Write if formatted.changed => match std::fs::write(path, formatted.text) {
            Ok(()) => 0,
            Err(error) => {
                eprintln!("seseragi: failed to write {}: {error}", path.display());
                2
            }
        },
        _ => 0,
    }
}

fn format_stdin(mode: FormatMode) -> Result<i32, String> {
    let mut source = String::new();
    std::io::stdin()
        .read_to_string(&mut source)
        .map_err(|error| format!("failed to read standard input: {error}"))?;
    let formatted = match format_module("<stdin>", &source) {
        Ok(formatted) => formatted,
        Err(diagnostics) => {
            eprint!("{}", render_terminal_diagnostics(&diagnostics, &source));
            return Ok(2);
        }
    };
    if mode == FormatMode::Check {
        if formatted.changed {
            eprintln!("<stdin> is not canonically formatted");
            return Ok(1);
        }
        return Ok(0);
    }
    let mut stdout = std::io::stdout().lock();
    stdout
        .write_all(formatted.text.as_bytes())
        .and_then(|()| stdout.flush())
        .map_err(|error| format!("failed to write standard output: {error}"))?;
    Ok(0)
}
