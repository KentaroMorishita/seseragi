use std::fs;
use std::path::{Path, PathBuf};

#[path = "write_schema1_artifact/selection.rs"]
mod selection;

use selection::{Selection, Stage};

fn main() {
    let mut arguments = std::env::args_os().skip(1);
    let Some(case) = arguments.next().map(PathBuf::from) else {
        print_usage();
        std::process::exit(2);
    };
    let argument = match arguments.next() {
        Some(argument) => match argument.into_string() {
            Ok(argument) => Some(argument),
            Err(_) => {
                eprintln!("option must be valid UTF-8");
                std::process::exit(2);
            }
        },
        None => None,
    };
    if arguments.next().is_some() {
        print_usage();
        std::process::exit(2);
    }
    let selection = Selection::resolve(&case, argument.as_deref()).unwrap_or_else(|error| {
        eprintln!("{}: {error}", case.display());
        std::process::exit(2);
    });
    if let Err(error) = write_case(&case, &selection) {
        eprintln!("{}: {error}", case.display());
        std::process::exit(1);
    }
}

fn print_usage() {
    eprintln!(
        "usage: write_schema1_artifact CASE_DIR [--only=STAGE[,STAGE...]]\n\
         without --only, only artifacts already present in CASE_DIR are refreshed"
    );
}

fn write_case(case: &Path, selection: &Selection) -> Result<(), String> {
    let source_path = case.join("main.ssrg");
    let source = fs::read_to_string(&source_path)
        .map_err(|error| format!("failed to read main.ssrg: {error}"))?;
    let source_name = source_name(case)?;
    let compiled = selection
        .requires_driver()
        .then(|| compile_case(case, &source_name, &source))
        .transpose()?;

    if selection.contains(Stage::Tokens) {
        write_json(
            case,
            "tokens.json",
            &seseragi_syntax::lex("main.ssrg", &source),
        )?;
    }
    if selection.contains(Stage::Cst) {
        write_json(
            case,
            "cst.json",
            &seseragi_syntax::parse_cst("main.ssrg", &source),
        )?;
    }
    if selection.contains(Stage::Diagnostics) {
        write_json(
            case,
            "diagnostics.json",
            &seseragi_syntax::parse_diagnostics("main.ssrg", &source),
        )?;
    }
    if selection.contains(Stage::SemanticDiagnostics) {
        write_json(
            case,
            "semantic-diagnostics.json",
            &seseragi_semantics::semantic_diagnostics("main.ssrg", &source),
        )?;
    }
    if selection.contains(Stage::Interface) {
        write_json(
            case,
            "interface.json",
            &seseragi_syntax::parse_module_interface(&source_name, &source),
        )?;
    }
    if selection.contains(Stage::SurfaceAst) {
        write_json(
            case,
            "surface-ast.json",
            &seseragi_syntax::parse_surface_ast("main.ssrg", &source),
        )?;
    }

    if selection.contains(Stage::ResolvedAst) {
        write_json(
            case,
            "resolved-ast.json",
            &seseragi_semantics::resolve_module(&source_name, &source),
        )?;
    }

    if selection.contains(Stage::TypedHir) {
        write_json(
            case,
            "typed-hir.json",
            &compiled
                .as_ref()
                .expect("driver output exists for typed HIR")
                .typed_hir,
        )?;
    }
    if selection.contains(Stage::TypedInterface) {
        if let Some(compiled) = &compiled {
            write_json(case, "typed-interface.json", &compiled.typed_interface)?;
        } else {
            write_json(
                case,
                "typed-interface.json",
                &seseragi_semantics::type_module_public_interface(&source_name, &source),
            )?;
        }
    }

    if selection.contains(Stage::CoreIr) {
        write_json(
            case,
            "core-ir.json",
            &compiled
                .as_ref()
                .expect("driver output exists for Core IR")
                .core_ir,
        )?;
    }

    if selection.contains(Stage::TypeScriptIr) {
        write_json(
            case,
            "typescript-ir.json",
            &compiled
                .as_ref()
                .expect("driver output exists for TypeScript IR")
                .typescript_ir,
        )?;
    }

    if selection.contains(Stage::GeneratedModule) {
        let bundle = &compiled
            .as_ref()
            .expect("driver output exists for generated module")
            .generated;
        write_json(case, "generated-module.json", &bundle.metadata)?;
        write_json(case, "main.ts.map", &bundle.source_map)?;
        fs::write(case.join("main.ts"), &bundle.typescript)
            .map_err(|error| format!("failed to write main.ts: {error}"))?;
    }

    Ok(())
}

fn compile_case(
    case: &Path,
    source_name: &str,
    source: &str,
) -> Result<seseragi_driver::CompiledModule, String> {
    let module_id = module_id(case)?;
    seseragi_driver::compile_module(seseragi_driver::CompileInput::new(
        source_name,
        &module_id,
        source,
    ))
    .map_err(|diagnostics| {
        let diagnostics = serde_json::to_string_pretty(&diagnostics)
            .unwrap_or_else(|_| "<failed to encode diagnostics>".to_owned());
        format!("compiler rejected backend artifact generation:\n{diagnostics}")
    })
}

fn write_json<T: serde::Serialize>(case: &Path, file_name: &str, value: &T) -> Result<(), String> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|error| format!("failed to encode {file_name}: {error}"))?;
    fs::write(case.join(file_name), format!("{json}\n"))
        .map_err(|error| format!("failed to write {file_name}: {error}"))
}

fn source_name(case: &Path) -> Result<String, String> {
    Ok(format!("{}/main.ssrg", module_id(case)?))
}

fn module_id(case: &Path) -> Result<String, String> {
    let name = case
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "case directory has no valid name".to_owned())?;
    Ok(format!("artifact/{name}"))
}
