use crate::{
    typed::{
        instances::analyze_instance_contracts, instances::analyze_instances,
        instances::DerivedInstanceIssue, TypedResolution,
    },
    ResolvedModule,
};
use seseragi_syntax::{ByteRange, Diagnostic, DiagnosticSeverity, RelatedDiagnostic};

mod contracts;
#[cfg(test)]
mod tests;

pub(super) fn collect_trait_diagnostics(
    resolved: &ResolvedModule,
    resolution: &TypedResolution<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    diagnostics.extend(
        analyze_instances(resolved, resolution)
            .issues
            .iter()
            .map(diagnostic),
    );
    diagnostics.extend(
        analyze_instance_contracts(resolved, resolution)
            .iter()
            .map(contracts::diagnostic),
    );
}

fn diagnostic(issue: &DerivedInstanceIssue) -> Diagnostic {
    match issue {
        DerivedInstanceIssue::OrphanInstance { identity, primary } => Diagnostic {
            type_difference: None,
            id: String::new(),
            code: "SES-T0202".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "trait.instance-orphan".to_owned(),
            primary: byte_range(*primary),
            related: vec![RelatedDiagnostic {
                message: format!("{identity} must be defined by the trait or an outermost nominal head type's module; an alias does not create an owner"),
                primary: byte_range(*primary),
            }],
            fixes: Vec::new(),
        },
        DerivedInstanceIssue::UnresolvedInstanceHead { primary } => Diagnostic {
            type_difference: None,
            id: String::new(),
            code: "SES-T0101".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "trait.instance-head-unresolved".to_owned(),
            primary: byte_range(*primary),
            related: vec![RelatedDiagnostic {
                message: "the expanded instance head must preserve its resolved constructor and all declared type parameters; an unrepresented alias constructor cannot yet be used here".to_owned(),
                primary: byte_range(*primary),
            }],
            fixes: Vec::new(),
        },
        DerivedInstanceIssue::UnprovedInstanceCoherence { trait_name, primary, first, provider_module } => Diagnostic {
            type_difference: None, id: String::new(), code: "SES-T0101".to_owned(),
            severity: DiagnosticSeverity::Error, message_key: "trait.instance-coherence-unproved".to_owned(),
            primary: byte_range(*primary),
            related: vec![RelatedDiagnostic {
                message: match provider_module {
                    Some(provider) => format!("cannot yet prove this {trait_name} instance disjoint from the instance provided by {provider}; higher-kinded constructor equality is not fully supported"),
                    None => format!("cannot yet prove this {trait_name} instance disjoint from the instance defined here; higher-kinded constructor equality is not fully supported"),
                },
                primary: byte_range(first.unwrap_or(*primary)),
            }],
            fixes: Vec::new(),
        },
        DerivedInstanceIssue::UnknownTrait {
            trait_name,
            primary,
            declaration,
        } => Diagnostic {
            type_difference: None,
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
        DerivedInstanceIssue::UnsupportedDerivedMember {
            trait_name,
            member_type,
            primary,
            declaration,
        } => {
            let requirement = crate::TypeDocument::Named {
                name: trait_name.clone(),
                canonical: None,
                arguments: vec![crate::TypeDocument::from_typed_type(member_type)],
            }
            .render(crate::TypeRenderOptions::default());
            Diagnostic {
                type_difference: None,
                id: String::new(),
                code: "SES-T0201".to_owned(),
                severity: DiagnosticSeverity::Error,
                message_key: "trait.instance-missing".to_owned(),
                primary: byte_range(*primary),
                related: vec![RelatedDiagnostic {
                    message: format!("required {requirement} instance is not available"),
                    primary: byte_range(*declaration),
                }],
                fixes: Vec::new(),
            }
        }
        DerivedInstanceIssue::AmbiguousInstance {
            trait_name,
            type_identity,
            provider_module,
            primary,
        } => Diagnostic {
            type_difference: None,
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
        DerivedInstanceIssue::DuplicateLocalInstance {
            trait_name,
            identity,
            primary,
            first,
        } => Diagnostic {
            type_difference: None,
            id: String::new(),
            code: "SES-T0202".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "trait.instance-duplicate".to_owned(),
            primary: byte_range(*primary),
            related: vec![RelatedDiagnostic {
                message: format!("{trait_name} instance {identity} was first defined here"),
                primary: byte_range(*first),
            }],
            fixes: Vec::new(),
        },
        DerivedInstanceIssue::OverlappingLocalInstance { trait_name, identity, primary, first } => Diagnostic {
            type_difference: None, id: String::new(), code: "SES-T0202".to_owned(),
            severity: DiagnosticSeverity::Error, message_key: "trait.instance-duplicate".to_owned(),
            primary: byte_range(*primary),
            related: vec![RelatedDiagnostic { message: format!("this {trait_name} instance {identity} overlaps the instance defined here"), primary: byte_range(*first) }],
            fixes: Vec::new(),
        },
        DerivedInstanceIssue::OverlappingStandardInstance {
            trait_name,
            standard_identity,
            primary,
        } => Diagnostic {
            type_difference: None,
            id: String::new(),
            code: "SES-T0202".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "trait.instance-duplicate".to_owned(),
            primary: byte_range(*primary),
            related: vec![RelatedDiagnostic {
                message: format!(
                    "{trait_name} instance overlaps sealed standard instance {standard_identity}"
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
