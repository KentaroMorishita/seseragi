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
    let surface = serde_json::to_string_pretty(&seseragi_semantics::standard_prelude_surface())
        .unwrap_or_else(|error| {
            eprintln!("failed to encode standard Prelude surface: {error}");
            std::process::exit(1);
        });
    fs::write(case.join("module.json"), format!("{surface}\n")).unwrap_or_else(|error| {
        eprintln!("failed to write {}: {error}", case.display());
        std::process::exit(1);
    });
}
