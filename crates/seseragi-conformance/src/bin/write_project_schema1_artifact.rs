use std::fs;
use std::path::{Path, PathBuf};

#[path = "../project_compile/compile.rs"]
mod compile;
#[path = "../project_compile/model.rs"]
mod model;

fn main() {
    let mut arguments = std::env::args_os().skip(1);
    let Some(case) = arguments.next().map(PathBuf::from) else {
        print_usage();
        std::process::exit(2);
    };
    if arguments.next().is_some() {
        print_usage();
        std::process::exit(2);
    }
    if let Err(error) = write_case(&case) {
        eprintln!("{}: {error}", case.display());
        std::process::exit(1);
    }
}

fn print_usage() {
    eprintln!("usage: write_project_schema1_artifact CASE_DIR");
}

fn write_case(case: &Path) -> Result<(), String> {
    let compiled_case = compile::compile_project_compile_case(case)?;
    for descriptor in &compiled_case.descriptor.modules {
        let compiled = compiled_case
            .compiled
            .modules
            .get(&descriptor.id)
            .expect("project compiler must produce every declared module");
        let artifacts = case.join(&descriptor.artifacts);
        fs::create_dir_all(&artifacts).map_err(|error| {
            format!(
                "failed to create project artifact directory {}: {error}",
                artifacts.display()
            )
        })?;
        write_json(&artifacts, "typed-hir.json", &compiled.typed_hir)?;
        write_json(
            &artifacts,
            "typed-interface.json",
            &compiled.typed_interface,
        )?;
        write_json(&artifacts, "core-ir.json", &compiled.core_ir)?;
        write_json(&artifacts, "typescript-ir.json", &compiled.typescript_ir)?;
        write_json(
            &artifacts,
            "generated-module.json",
            &compiled.generated.metadata,
        )?;
        write_json(&artifacts, "main.ts.map", &compiled.generated.source_map)?;
        fs::write(artifacts.join("main.ts"), &compiled.generated.typescript)
            .map_err(|error| format!("failed to write project TypeScript output: {error}"))?;
    }
    Ok(())
}

fn write_json<T: serde::Serialize>(
    directory: &Path,
    file_name: &str,
    value: &T,
) -> Result<(), String> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|error| format!("failed to encode {file_name}: {error}"))?;
    fs::write(directory.join(file_name), format!("{json}\n"))
        .map_err(|error| format!("failed to write {file_name}: {error}"))
}
