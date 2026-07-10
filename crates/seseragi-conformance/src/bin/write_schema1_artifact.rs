use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let mut arguments = std::env::args_os().skip(1);
    let Some(case) = arguments.next().map(PathBuf::from) else {
        eprintln!("usage: write_schema1_artifact CASE_DIR");
        std::process::exit(2);
    };
    let selection = arguments
        .next()
        .and_then(|argument| argument.into_string().ok())
        .and_then(|argument| argument.strip_prefix("--only=").map(str::to_owned));
    if let Err(error) = write_case(&case, selection.as_deref()) {
        eprintln!("{}: {error}", case.display());
        std::process::exit(1);
    }
}

fn write_case(case: &Path, selection: Option<&str>) -> Result<(), String> {
    let source_path = case.join("main.ssrg");
    let source = fs::read_to_string(&source_path)
        .map_err(|error| format!("failed to read main.ssrg: {error}"))?;
    let source_name = source_name(case)?;

    if should_write(selection, "tokens") {
        write_json(
            case,
            "tokens.json",
            &seseragi_syntax::lex("main.ssrg", &source),
        )?;
    }
    if should_write(selection, "cst") {
        write_json(
            case,
            "cst.json",
            &seseragi_syntax::parse_cst("main.ssrg", &source),
        )?;
    }
    if should_write(selection, "diagnostics") {
        write_json(
            case,
            "diagnostics.json",
            &seseragi_syntax::parse_diagnostics("main.ssrg", &source),
        )?;
    }
    if should_write(selection, "semantic-diagnostics") {
        write_json(
            case,
            "semantic-diagnostics.json",
            &seseragi_semantics::semantic_diagnostics("main.ssrg", &source),
        )?;
    }
    if should_write(selection, "interface") {
        write_json(
            case,
            "interface.json",
            &seseragi_syntax::parse_module_interface(&source_name, &source),
        )?;
    }
    if should_write(selection, "surface-ast") {
        write_json(
            case,
            "surface-ast.json",
            &seseragi_syntax::parse_surface_ast("main.ssrg", &source),
        )?;
    }

    let interface = seseragi_syntax::parse_module_interface(&source_name, &source);
    if should_write(selection, "resolved-ast") {
        write_json(
            case,
            "resolved-ast.json",
            &seseragi_semantics::resolve_module_interface(interface),
        )?;
    }

    let typed = seseragi_semantics::type_module(&source_name, &source);
    if should_write(selection, "typed-hir") {
        write_json(case, "typed-hir.json", &typed)?;
    }
    if should_write(selection, "typed-interface") {
        write_json(
            case,
            "typed-interface.json",
            &seseragi_semantics::type_module_public_interface(&source_name, &source),
        )?;
    }

    let core = seseragi_lowering::lower_typed_module(typed);
    if should_write(selection, "core-ir") {
        write_json(case, "core-ir.json", &core)?;
    }

    let typescript_ir = seseragi_lowering::lower_core_module_to_typescript_ir(core);
    if should_write(selection, "typescript-ir") {
        write_json(case, "typescript-ir.json", &typescript_ir)?;
    }

    let bundle = seseragi_lowering::emit_typescript_module(typescript_ir, &source);
    if should_write(selection, "generated-module") {
        write_json(case, "generated-module.json", &bundle.metadata)?;
        write_json(case, "main.ts.map", &bundle.source_map)?;
        fs::write(case.join("main.ts"), bundle.typescript)
            .map_err(|error| format!("failed to write main.ts: {error}"))?;
    }

    Ok(())
}

fn should_write(selection: Option<&str>, stage: &str) -> bool {
    selection.is_none_or(|selection| selection.split(',').any(|item| item == stage))
}

fn write_json<T: serde::Serialize>(case: &Path, file_name: &str, value: &T) -> Result<(), String> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|error| format!("failed to encode {file_name}: {error}"))?;
    fs::write(case.join(file_name), format!("{json}\n"))
        .map_err(|error| format!("failed to write {file_name}: {error}"))
}

fn source_name(case: &Path) -> Result<String, String> {
    let name = case
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "case directory has no valid name".to_owned())?;
    Ok(format!("artifact/{name}/main.ssrg"))
}
