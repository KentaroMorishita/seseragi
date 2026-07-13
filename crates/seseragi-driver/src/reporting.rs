use seseragi_source::LineIndex;
use seseragi_syntax::{DiagnosticArtifact, DiagnosticSeverity};

/// Renders the driver's structured diagnostics for a terminal surface.
///
/// LSP and WASM callers should consume [`DiagnosticArtifact`] directly. This
/// adapter exists so command-line frontends do not invent a second location
/// or severity policy.
pub fn render_terminal_diagnostics(artifact: &DiagnosticArtifact, source: &str) -> String {
    let lines = LineIndex::new(source);
    let mut rendered = String::new();
    for diagnostic in &artifact.diagnostics {
        let location = lines.locate(diagnostic.primary.start.min(source.len()));
        let severity = match diagnostic.severity {
            DiagnosticSeverity::Error => "error",
            DiagnosticSeverity::Warning => "warning",
            DiagnosticSeverity::Information => "info",
            DiagnosticSeverity::Hint => "hint",
        };
        rendered.push_str(&format!(
            "{}:{}:{}: {severity}[{}]: {}\n",
            artifact.source,
            location.line,
            location.column,
            diagnostic.code,
            diagnostic.message_key,
        ));
    }
    rendered
}

#[cfg(test)]
mod tests {
    use super::render_terminal_diagnostics;
    use seseragi_syntax::parse_diagnostics;

    #[test]
    fn renders_source_ranges_through_the_shared_line_index() {
        let source = "pub let answer: Int =\n";
        let diagnostics = parse_diagnostics("app.ssrg", source);
        let rendered = render_terminal_diagnostics(&diagnostics, source);

        assert!(rendered.starts_with("app.ssrg:1:"));
        assert!(rendered.contains("error[SES-"));
    }
}
