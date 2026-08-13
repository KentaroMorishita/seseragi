use std::{fs, path::PathBuf};

fn main() {
    let mut arguments = std::env::args_os().skip(1);
    let Some(case) = arguments.next().map(PathBuf::from) else {
        eprintln!("usage: write_stdlib_schema1_artifact CASE_DIR");
        std::process::exit(2);
    };
    if arguments.next().is_some() {
        eprintln!("usage: write_stdlib_schema1_artifact CASE_DIR");
        std::process::exit(2);
    }

    fs::create_dir_all(&case).unwrap_or_else(|error| {
        eprintln!("failed to create {}: {error}", case.display());
        std::process::exit(1);
    });
    let case_name = case.file_name().and_then(|name| name.to_str());
    let surface = match case_name {
        Some("prelude") => {
            serde_json::to_string_pretty(&seseragi_semantics::standard_prelude_surface())
        }
        Some("registry") => {
            serde_json::to_string_pretty(&seseragi_project::standard_module_registry_surface())
        }
        Some("parity") => seseragi_conformance::standard_module_parity_surface()
            .map_err(|error| serde_json::Error::io(std::io::Error::other(error)))
            .and_then(|surface| serde_json::to_string_pretty(&surface)),
        _ => {
            eprintln!("standard library case must be named `prelude`, `registry`, or `parity`");
            std::process::exit(2);
        }
    }
    .unwrap_or_else(|error| {
        eprintln!("failed to encode standard library surface: {error}");
        std::process::exit(1);
    });
    fs::write(case.join("module.json"), format!("{surface}\n")).unwrap_or_else(|error| {
        eprintln!("failed to write {}: {error}", case.display());
        std::process::exit(1);
    });
}
