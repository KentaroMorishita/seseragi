use seseragi_syntax::{ByteRange, Diagnostic, DiagnosticArtifact, DiagnosticSeverity};

use seseragi_project::LinkError;

pub(super) fn append_link_diagnostics(errors: Vec<LinkError>, artifact: &mut DiagnosticArtifact) {
    let next_id = artifact.diagnostics.len() + 1;
    artifact
        .diagnostics
        .extend(errors.into_iter().enumerate().map(|(index, error)| {
            let (code, message_key) = match error {
                LinkError::UnresolvedSpecifier { .. } => {
                    ("SES-N0104", "module.specifier-unresolved")
                }
                LinkError::MissingExport { .. } => ("SES-N0104", "module.export-unresolved"),
                LinkError::PrivateExport { .. } => ("SES-N0102", "module.private-symbol"),
                LinkError::DuplicateImport { .. } => ("SES-N0101", "module.import-ambiguous"),
                LinkError::MissingNamespaceAlias { .. }
                | LinkError::UnsupportedImportNamespace { .. } => {
                    ("SES-N0104", "module.import-unsupported")
                }
            };
            let origin = error.origin();
            Diagnostic {
                id: format!("d{}", next_id + index),
                code: code.to_owned(),
                severity: DiagnosticSeverity::Error,
                message_key: message_key.to_owned(),
                primary: ByteRange {
                    start: origin.start,
                    end: origin.end,
                },
                related: Vec::new(),
                fixes: Vec::new(),
            }
        }));
}
