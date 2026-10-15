//! Experimental analysis and measurement artifacts; no production compiler changes.
use seseragi_driver::{compile_module, BuildProfile, CompileInput};
use std::{env, fs, path::Path};

fn main() -> Result<(), String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 2 {
        return Err("Usage: cargo run -p seseragi-driver --example core_ir_performance -- source.ssrg output-directory".into());
    }
    let source = fs::read_to_string(&args[0]).map_err(|e| e.to_string())?;
    let output = Path::new(&args[1]);
    fs::create_dir_all(output).map_err(|e| e.to_string())?;
    for (name, profile) in [
        ("development", BuildProfile::Development),
        ("release", BuildProfile::Release),
    ] {
        let compiled = compile_module(
            CompileInput::new(&args[0], "performance/main", &source).with_profile(profile),
        )
        .map_err(|e| format!("{e:#?}"))?;
        for (suffix, value) in [
            (
                "core.json",
                serde_json::to_string_pretty(&compiled.core_ir).unwrap(),
            ),
            (
                "typescript-ir.json",
                serde_json::to_string_pretty(&compiled.typescript_ir).unwrap(),
            ),
            (
                "metadata.json",
                serde_json::to_string_pretty(&compiled.generated.metadata).unwrap(),
            ),
            ("ts", compiled.generated.typescript),
        ] {
            fs::write(output.join(format!("{name}.{suffix}")), value).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
