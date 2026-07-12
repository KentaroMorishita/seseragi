use std::path::Path;

pub(crate) fn artifact_module_id(case: &Path) -> Result<String, String> {
    let name = case
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "artifact case has no directory name".to_owned())?;
    Ok(format!("artifact/{name}"))
}

pub(crate) fn interface_source_name(case: &Path) -> Result<String, String> {
    Ok(format!("{}/main.ssrg", artifact_module_id(case)?))
}

pub(crate) fn compile_artifact_module(
    case: &Path,
    source: &str,
) -> Result<seseragi_driver::CompiledModule, String> {
    let source_name = interface_source_name(case)?;
    let module_id = artifact_module_id(case)?;
    seseragi_driver::compile_module(seseragi_driver::CompileInput::new(
        &source_name,
        &module_id,
        source,
    ))
    .map_err(|diagnostics| format_compiler_rejection(&diagnostics))
}

fn format_compiler_rejection(diagnostics: &seseragi_syntax::DiagnosticArtifact) -> String {
    let summary = diagnostics
        .diagnostics
        .iter()
        .map(|diagnostic| format!("{} {}", diagnostic.id, diagnostic.code))
        .collect::<Vec<_>>()
        .join(", ");
    format!("compiler rejected conformance case: {summary}")
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

pub(crate) fn parse_semantic_diagnostics_json(
    source_name: impl Into<String>,
    source: &str,
) -> Result<serde_json::Value, String> {
    let diagnostics = seseragi_semantics::semantic_diagnostics(source_name, source);
    serde_json::to_value(&diagnostics)
        .map_err(|error| format!("failed to encode SemanticDiagnostics: {error}"))
}

pub(crate) fn parse_resolved_ast_json(
    source_name: impl Into<String>,
    source: &str,
) -> Result<serde_json::Value, String> {
    let resolved_ast = seseragi_semantics::resolve_module(source_name, source);
    serde_json::to_value(&resolved_ast)
        .map_err(|error| format!("failed to encode ResolvedAst: {error}"))
}

pub(crate) fn parse_typed_interface_json(
    source_name: impl Into<String>,
    source: &str,
) -> Result<serde_json::Value, String> {
    let typed_interface = seseragi_semantics::type_module_public_interface(source_name, source);
    serde_json::to_value(&typed_interface)
        .map_err(|error| format!("failed to encode TypedModuleInterface: {error}"))
}
