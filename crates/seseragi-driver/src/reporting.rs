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
            diagnostic.message(),
        ));
        if let Some(line) = source.split('\n').nth(location.line.saturating_sub(1)) {
            rendered.push_str(&format!("  | {line}\n  | "));
            rendered.push_str(&" ".repeat(location.column.saturating_sub(1)));
            let end = lines.locate(diagnostic.primary.end.min(source.len()));
            let width = if end.line == location.line {
                end.column.saturating_sub(location.column).max(1)
            } else {
                1
            };
            rendered.push_str(&"^".repeat(width));
            if let Some(label) = diagnostic.labels().first() {
                rendered.push_str(&format!(" {}", label.message));
            }
            rendered.push('\n');
        }
        let (expected, actual) = diagnostic.expected_actual_types();
        if let Some(expected) = expected {
            rendered.push_str(&format!("  = expected: {expected}\n"));
        }
        if let Some(actual) = actual {
            rendered.push_str(&format!("  = actual: {actual}\n"));
        }
        for note in diagnostic.notes() {
            rendered.push_str(&format!("  = note: {note}\n"));
        }
        for help in diagnostic.helps() {
            rendered.push_str(&format!("  = help: {help}\n"));
        }
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
        assert!(rendered.contains("Expected an expression here"));
        assert!(!rendered.contains("parser.expected-expression"));
        assert!(rendered.contains("pub let answer: Int ="));
        assert!(rendered.contains("= help:"));
    }
}
