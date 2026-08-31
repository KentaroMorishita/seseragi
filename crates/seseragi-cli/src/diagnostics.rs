use serde_json::json;
use seseragi_runtime::DiagnosticFormat;
use seseragi_syntax::DiagnosticArtifact;

pub(crate) struct DiagnosticDocument<'a> {
    pub path: &'a str,
    pub source: &'a str,
    pub artifact: &'a DiagnosticArtifact,
}

pub(crate) fn render_diagnostics(
    format: DiagnosticFormat,
    documents: &[DiagnosticDocument<'_>],
) -> Result<String, String> {
    match format {
        DiagnosticFormat::Text => Ok(documents
            .iter()
            .map(|document| {
                seseragi_driver::render_terminal_diagnostics(document.artifact, document.source)
            })
            .collect()),
        DiagnosticFormat::Json => {
            let value = json!({
                "schema": 1,
                "toolVersion": seseragi_release::TOOLCHAIN_VERSION,
                "languageVersion": seseragi_project::IMPLEMENTED_LANGUAGE_VERSION,
                "unicodeVersion": seseragi_project::UNICODE_VERSION,
                "diagnostics": documents
                    .iter()
                    .map(|document| json!({
                        "path": document.path,
                        "diagnostics": document.artifact,
                    }))
                    .collect::<Vec<_>>(),
            });
            serde_json::to_string(&value)
                .map(|encoded| format!("{encoded}\n"))
                .map_err(|error| format!("failed to encode diagnostics: {error}"))
        }
    }
}
