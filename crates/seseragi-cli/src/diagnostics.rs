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

#[cfg(test)]
mod tests {
    use super::*;
    use seseragi_syntax::{parse_diagnostics, ByteRange, Diagnostic, DiagnosticSeverity};

    #[test]
    fn renders_internal_compiler_error_message_and_note_in_text_and_json() {
        let source = "pub fn answer -> Int = 42\n";
        let mut artifact = parse_diagnostics("compiler.ssrg", source);
        assert!(artifact.diagnostics.is_empty());
        artifact.diagnostics.push(Diagnostic {
            id: "d1".to_owned(),
            code: "SES-I0001".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "internal.compiler-error".to_owned(),
            primary: ByteRange { start: 0, end: 0 },
            related: Vec::new(),
            fixes: Vec::new(),
            type_difference: None,
        });
        let documents = [DiagnosticDocument {
            path: "compiler.ssrg",
            source,
            artifact: &artifact,
        }];
        let note = "This is a bug in the compiler, not an error in your source. Please report it with the source that triggers it.";
        let text = render_diagnostics(DiagnosticFormat::Text, &documents).unwrap();
        assert!(text.starts_with("compiler.ssrg:1:1: error[SES-I0001]: Internal compiler error\n"));
        assert!(text.contains(&format!("  = note: {note}\n")));
        assert!(!text.contains("internal.compiler-error"));

        let encoded = render_diagnostics(DiagnosticFormat::Json, &documents).unwrap();
        let json: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        let diagnostics = json["diagnostics"][0]["diagnostics"]["diagnostics"]
            .as_array()
            .unwrap();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0]["code"], "SES-I0001");
        assert_eq!(diagnostics[0]["message"], "Internal compiler error");
        assert_eq!(diagnostics[0]["notes"], json!([note]));
    }
}
