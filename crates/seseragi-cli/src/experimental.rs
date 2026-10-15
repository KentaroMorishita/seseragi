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
    if !path.ends_with(".ssrg") || !std::path::Path::new(path).is_file() {
        return Err("experimental Rust target currently accepts one .ssrg file".into());
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
    let artifact =
        seseragi_backend_rust::emit_rust_module(&checked.core_ir, entry).map_err(|e| {
            match &e.origin {
                Some(origin) => format!("{path}:{}..{}: {e}", origin.start, origin.end),
                None => e.to_string(),
            }
        })?;
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
    let source_path = fs::canonicalize(path).map_err(|e| e.to_string())?;
    if [&rust_path, &metadata_path]
        .iter()
        .any(|output| fs::canonicalize(output).ok().as_ref() == Some(&source_path))
    {
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

fn usage() -> &'static str {
    "usage: seseragi experimental emit-rust source.ssrg --entry function --output program.rs | build-native source.ssrg --entry function --out-dir path"
}
