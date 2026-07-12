use seseragi_syntax::{
    ByteRange, Diagnostic, DiagnosticArtifact, DiagnosticSeverity, ImportOccurrence,
};

pub(super) fn append_unlinked_dependency_diagnostics(
    imports: Vec<ImportOccurrence>,
    artifact: &mut DiagnosticArtifact,
) {
    let next_id = artifact.diagnostics.len() + 1;
    artifact
        .diagnostics
        .extend(
            imports
                .into_iter()
                .enumerate()
                .map(|(index, import)| Diagnostic {
                    id: format!("d{}", next_id + index),
                    code: "SES-N0104".to_owned(),
                    severity: DiagnosticSeverity::Error,
                    message_key: "module.specifier-unresolved".to_owned(),
                    primary: ByteRange {
                        start: import.origin.start,
                        end: import.origin.end,
                    },
                    related: Vec::new(),
                    fixes: Vec::new(),
                }),
        );
}
