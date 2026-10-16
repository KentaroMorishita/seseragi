use seseragi_driver::{compile_core_module, CompileInput};
use std::{fs, path::PathBuf, process::Command};

pub(crate) fn run(arguments: &[String]) -> Result<i32, String> {
    let [operation, path, flags @ ..] = arguments else {
        return Err(usage().into());
    };
    let native = match operation.as_str() {
        "emit-rust" => false,
        "build-native" => true,
        _ => return Err(usage().into()),
    };
    let mut entry = None;
    let mut destination = None;
    for pair in flags.chunks(2) {
        let [flag, value] = pair else {
            return Err(usage().into());
        };
        match flag.as_str() {
            "--entry" if entry.is_none() => entry = Some(value.as_str()),
            "--output" if !native && destination.is_none() => {
                destination = Some(PathBuf::from(value))
            }
            "--out-dir" if native && destination.is_none() => {
                destination = Some(PathBuf::from(value))
            }
            _ => return Err(usage().into()),
        }
    }
    let entry = entry.ok_or_else(|| usage().to_owned())?;
    let destination = destination.ok_or_else(|| usage().to_owned())?;
    let (artifact, source_paths) = if std::path::Path::new(path).is_dir() {
        compile_project(path, entry)?
    } else {
        if !path.ends_with(".ssrg") || !std::path::Path::new(path).is_file() {
            return Err("experimental Rust target accepts a .ssrg file or package root".into());
        }
        let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let checked = compile_core_module(CompileInput::new(path, "experimental/main", &source))
            .map_err(|diagnostics| {
                seseragi_driver::render_terminal_diagnostics(&diagnostics, &source)
            })?;
        if !checked.diagnostics.diagnostics.is_empty() {
            eprint!(
                "{}",
                seseragi_driver::render_terminal_diagnostics(&checked.diagnostics, &source)
            );
        }
        (
            seseragi_backend_rust::emit_rust_module(&checked.core_ir, entry)
                .map_err(|e| target_error(e, Some(path)))?,
            vec![fs::canonicalize(path).map_err(|e| e.to_string())?],
        )
    };
    // Check the complete target capability before writing any output.
    // Keep generated source, manifest, and compiler artifacts together.
    let (rust_path, metadata_path) = if native {
        fs::create_dir_all(&destination).map_err(|e| e.to_string())?;
        (
            destination.join("program.rs"),
            destination.join("rust-backend.json"),
        )
    } else {
        (
            destination.clone(),
            destination.with_extension("rust-backend.json"),
        )
    };
    if [&rust_path, &metadata_path].iter().any(|output| {
        fs::canonicalize(output)
            .ok()
            .is_some_and(|p| source_paths.contains(&p))
    }) {
        return Err("output must differ from the source file".into());
    }
    fs::write(&rust_path, &artifact.source).map_err(|e| format!("{}: {e}", rust_path.display()))?;
    fs::write(
        &metadata_path,
        serde_json::to_string_pretty(&artifact.metadata).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    if native {
        let executable = destination.join(if cfg!(windows) {
            "program.exe"
        } else {
            "program"
        });
        let output = Command::new("rustc")
            .args(["--edition=2021", "--crate-name", "seseragi_program", "-O"])
            .arg(&rust_path)
            .arg("-o")
            .arg(&executable)
            .output()
            .map_err(|e| format!("could not start rustc: {e}; install a Rust toolchain"))?;
        if !output.status.success() {
            return Err(format!(
                "Rust target compilation failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        println!("{}", executable.display());
    } else {
        println!("{}", rust_path.display());
    }
    Ok(0)
}

fn target_error(
    error: seseragi_backend_rust::RustBackendError,
    source_path: Option<&str>,
) -> String {
    match &error.origin {
        Some(origin) => format!(
            "{}:{}..{}: {error}",
            source_path.unwrap_or(&origin.source),
            origin.start,
            origin.end
        ),
        None => error.to_string(),
    }
}

fn compile_project(
    path: &str,
    entry: &str,
) -> Result<(seseragi_backend_rust::RustArtifact, Vec<PathBuf>), String> {
    seseragi_project::read_and_validate_lockfile(path)
        .map_err(|error| format!("{}: {error}", error.code()))?;
    let project = seseragi_project::load_local_project(path)
        .map_err(|error| format!("{}: {error}", error.code()))?;
    let checked = seseragi_driver::compile_core_local_project(&project).map_err(|error| {
        if let seseragi_driver::ProjectCompileError::Diagnostics { modules } = error.error() {
            modules
                .iter()
                .map(|diagnostics| {
                    let source = project
                        .modules()
                        .find(|(identity, _)| {
                            seseragi_project::logical_module_id(identity) == diagnostics.module
                        })
                        .expect("diagnostics belong to a loaded module")
                        .1
                        .source();
                    seseragi_driver::render_terminal_diagnostics(&diagnostics.diagnostics, source)
                })
                .collect::<String>()
        } else {
            format!("project compiler rejected package: {:?}", error.error())
        }
    })?;
    for (id, module) in &checked.compiled.modules {
        if !module.diagnostics.diagnostics.is_empty() {
            let source = project
                .modules()
                .find(|(identity, _)| seseragi_project::logical_module_id(identity) == *id)
                .expect("checked module was loaded")
                .1
                .source();
            eprint!(
                "{}",
                seseragi_driver::render_terminal_diagnostics(&module.diagnostics, source)
            );
        }
    }
    let modules = checked
        .compiled
        .order
        .iter()
        .map(|id| &checked.compiled.modules[id].core_ir)
        .collect::<Vec<_>>();
    let artifact = seseragi_backend_rust::emit_rust_project(&modules, &checked.entry_module, entry)
        .map_err(|error| {
            let source = error.module.as_deref().and_then(|id| {
                project
                    .modules()
                    .find(|(identity, _)| seseragi_project::logical_module_id(identity) == id)
                    .map(|(_, module)| module.source_path().to_string_lossy())
            });
            target_error(error, source.as_deref())
        })?;
    let source_paths = project
        .modules()
        .map(|(_, module)| fs::canonicalize(module.source_path()).map_err(|e| e.to_string()))
        .collect::<Result<_, _>>()?;
    Ok((artifact, source_paths))
}

fn usage() -> &'static str {
    "usage: seseragi experimental emit-rust source.ssrg|package-root --entry function --output program.rs | build-native source.ssrg|package-root --entry function --out-dir path"
}
