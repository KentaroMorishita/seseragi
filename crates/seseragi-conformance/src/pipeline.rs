use std::path::Path;

pub(crate) fn interface_source_name(case: &Path) -> Result<String, String> {
    let name = case
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "interface case has no directory name".to_owned())?;
    Ok(format!("artifact/{name}/main.ssrg"))
}

pub(crate) fn parse_module_interface_json(
    source_name: impl Into<String>,
    source: &str,
) -> Result<serde_json::Value, String> {
    let interface = seseragi_syntax::parse_module_interface(source_name, source);
    serde_json::to_value(&interface)
        .map_err(|error| format!("failed to encode ModuleInterface: {error}"))
}

pub(crate) fn parse_diagnostics_json(
    source_name: impl Into<String>,
    source: &str,
) -> Result<serde_json::Value, String> {
    let diagnostics = seseragi_syntax::parse_diagnostics(source_name, source);
    serde_json::to_value(&diagnostics)
        .map_err(|error| format!("failed to encode Diagnostics: {error}"))
}

pub(crate) fn parse_resolved_ast_json(
    source_name: impl Into<String>,
    source: &str,
) -> Result<serde_json::Value, String> {
    let interface = seseragi_syntax::parse_module_interface(source_name, source);
    let resolved_ast = seseragi_semantics::resolve_module_interface(interface);
    serde_json::to_value(&resolved_ast)
        .map_err(|error| format!("failed to encode ResolvedAst: {error}"))
}

pub(crate) fn parse_typed_hir_json(
    source_name: impl Into<String>,
    source: &str,
) -> Result<serde_json::Value, String> {
    let typed_hir = seseragi_semantics::type_module(source_name, source);
    serde_json::to_value(&typed_hir).map_err(|error| format!("failed to encode TypedHir: {error}"))
}

pub(crate) fn parse_core_ir_json(
    source_name: impl Into<String>,
    source: &str,
) -> Result<serde_json::Value, String> {
    let typed_hir = seseragi_semantics::type_module(source_name, source);
    let core_ir = seseragi_lowering::lower_typed_module(typed_hir);
    serde_json::to_value(&core_ir).map_err(|error| format!("failed to encode CoreIr: {error}"))
}

pub(crate) fn parse_typescript_ir_json(
    source_name: impl Into<String>,
    source: &str,
) -> Result<serde_json::Value, String> {
    let typed_hir = seseragi_semantics::type_module(source_name, source);
    let core_ir = seseragi_lowering::lower_typed_module(typed_hir);
    let typescript_ir = seseragi_lowering::lower_core_module_to_typescript_ir(core_ir);
    serde_json::to_value(&typescript_ir)
        .map_err(|error| format!("failed to encode TypeScriptIr: {error}"))
}

pub(crate) fn emit_generated_module(
    source_name: impl Into<String>,
    source: &str,
) -> Result<seseragi_lowering::GeneratedBundle, String> {
    let typed_hir = seseragi_semantics::type_module(source_name, source);
    let core_ir = seseragi_lowering::lower_typed_module(typed_hir);
    let typescript_ir = seseragi_lowering::lower_core_module_to_typescript_ir(core_ir);
    Ok(seseragi_lowering::emit_typescript_module(
        typescript_ir,
        source,
    ))
}
