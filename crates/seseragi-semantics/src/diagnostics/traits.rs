use crate::{
    typed::{
        instances::analyze_derived_instances, instances::DerivedInstanceIssue, TypedResolution,
    },
    ResolvedModule,
};
use seseragi_syntax::{ByteRange, Diagnostic, DiagnosticSeverity, RelatedDiagnostic};

#[cfg(test)]
mod tests;

pub(super) fn collect_trait_diagnostics(
    resolved: &ResolvedModule,
    resolution: &TypedResolution<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    diagnostics.extend(
        analyze_derived_instances(resolved, resolution)
            .issues
            .iter()
            .map(diagnostic),
    );
}

fn diagnostic(issue: &DerivedInstanceIssue) -> Diagnostic {
    match issue {
        DerivedInstanceIssue::UnknownTrait {
            trait_name,
            primary,
            declaration,
        } => Diagnostic {
            id: String::new(),
            code: "SES-N0001".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "name.unresolved".to_owned(),
            primary: byte_range(*primary),
            related: vec![RelatedDiagnostic {
                message: format!("deriving trait {trait_name} is not defined"),
                primary: byte_range(*declaration),
            }],
            fixes: Vec::new(),
        },
        DerivedInstanceIssue::UnsupportedGenericShow {
            type_name,
            primary,
            declaration,
        } => Diagnostic {
            id: String::new(),
            code: "SES-T0201".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "trait.instance-missing".to_owned(),
            primary: byte_range(*primary),
            related: vec![RelatedDiagnostic {
                message: format!(
                    "derived Show<{type_name}> for generic ADTs is not implemented yet"
                ),
                primary: byte_range(*declaration),
            }],
            fixes: Vec::new(),
        },
        DerivedInstanceIssue::UnsupportedShowPayload {
            payload_name,
            primary,
            declaration,
        } => Diagnostic {
            id: String::new(),
            code: "SES-T0201".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "trait.instance-missing".to_owned(),
            primary: byte_range(*primary),
            related: vec![RelatedDiagnostic {
                message: format!("required Show<{payload_name}> instance is not available"),
                primary: byte_range(*declaration),
            }],
            fixes: Vec::new(),
        },
        DerivedInstanceIssue::AmbiguousInstance {
            trait_name,
            type_identity,
            provider_module,
            primary,
        } => Diagnostic {
            id: String::new(),
            code: "SES-T0202".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "trait.instance-ambiguous".to_owned(),
            primary: byte_range(*primary),
            related: vec![RelatedDiagnostic {
                message: format!(
                    "conflicting {trait_name}<{type_identity}> instance is imported from {provider_module}"
                ),
                primary: byte_range(*primary),
            }],
            fixes: Vec::new(),
        },
    }
}

fn byte_range(span: seseragi_syntax::ByteSpan) -> ByteRange {
    ByteRange {
        start: span.start,
        end: span.end,
    }
}
