use crate::{
    ResolvedModule, SymbolKind, SymbolNamespace, TypedConstraint, TypedInstance,
    TypedInstanceImplementation, TypedRecordField, TypedShowPayloadEvidence, TypedType,
};
use seseragi_syntax::{ByteSpan, SurfaceDecl, TypeParameter, TypeRef};
use std::collections::{BTreeMap, BTreeSet};

use super::{canonical_instance_identity, DerivedInstanceIssue, TypedResolution};

const DISPLAY_TRAITS: [(&str, &str); 2] = [
    ("Show", "std/prelude::Show"),
    ("Debug", "std/prelude::Debug"),
];

pub(super) struct DerivedShowAnalysis {
    pub(super) instances: Vec<TypedInstance>,
    pub(super) issues: Vec<DerivedInstanceIssue>,
}

struct ShowCandidate {
    trait_name: String,
    trait_identity: String,
    name: String,
    symbol: String,
    origin: ByteSpan,
    type_parameters: Vec<TypeParameter>,
    members: Vec<ShowMember>,
}

#[derive(Clone)]
struct ShowMember {
    key: String,
    type_ref: TypeRef,
}

pub(super) fn analyze_derived_show(
    resolved: &ResolvedModule,
    resolution: &TypedResolution<'_>,
) -> DerivedShowAnalysis {
    let candidates = collect_candidates(resolved, resolution);
    let all_identities = candidates
        .iter()
        .map(candidate_identity)
        .collect::<BTreeSet<_>>();
    let valid = valid_candidate_identities(&candidates, resolution, &all_identities);
    let issues = collect_member_issues(&candidates, resolution, &all_identities, &valid);
    let instances = candidates
        .into_iter()
        .filter(|candidate| valid.contains(&candidate_identity(candidate)))
        .map(|candidate| typed_instance(candidate, resolution, &all_identities, &valid))
        .collect();
    DerivedShowAnalysis { instances, issues }
}

fn collect_candidates(
    resolved: &ResolvedModule,
    resolution: &TypedResolution<'_>,
) -> Vec<ShowCandidate> {
    let mut candidates = Vec::new();
    for declaration in &resolved.declarations {
        let Some((name, name_span, type_parameters, deriving, members, origin)) =
            display_declaration(declaration, resolution)
        else {
            continue;
        };
        let Some(symbol) = resolution
            .declaration_symbol(name_span, SymbolKind::Type)
            .and_then(|symbol| symbol.canonical.clone())
        else {
            continue;
        };
        for (trait_name, trait_identity) in DISPLAY_TRAITS {
            if deriving.iter().any(|derived| derived == trait_name) {
                candidates.push(ShowCandidate {
                    trait_name: trait_name.to_owned(),
                    trait_identity: trait_identity.to_owned(),
                    name: name.clone(),
                    symbol: symbol.clone(),
                    origin,
                    type_parameters: type_parameters.clone(),
                    members: members.clone(),
                });
            }
        }
    }
    candidates
}

type DisplayDeclaration = (
    String,
    ByteSpan,
    Vec<TypeParameter>,
    Vec<String>,
    Vec<ShowMember>,
    ByteSpan,
);

fn display_declaration(
    declaration: &SurfaceDecl,
    resolution: &TypedResolution<'_>,
) -> Option<DisplayDeclaration> {
    match declaration {
        SurfaceDecl::Type {
            name,
            name_span,
            type_parameters,
            deriving,
            variants,
            span,
            ..
        } => Some((
            name.clone(),
            *name_span,
            type_parameters.clone(),
            deriving.clone(),
            variants
                .iter()
                .filter_map(|variant| {
                    Some(ShowMember {
                        key: resolution
                            .declaration_symbol(variant.name_span, SymbolKind::Constructor)?
                            .canonical
                            .clone()?,
                        type_ref: variant.payload.clone()?,
                    })
                })
                .collect(),
            *span,
        )),
        SurfaceDecl::Newtype {
            name,
            name_span,
            type_parameters,
            deriving,
            representation,
            span,
            ..
        } => Some((
            name.clone(),
            *name_span,
            type_parameters.clone(),
            deriving.clone(),
            vec![ShowMember {
                key: resolution
                    .declaration_symbol(*name_span, SymbolKind::Constructor)?
                    .canonical
                    .clone()?,
                type_ref: representation.clone(),
            }],
            *span,
        )),
        SurfaceDecl::Struct {
            name,
            name_span,
            type_parameters,
            deriving,
            fields,
            span,
            ..
        } => Some((
            name.clone(),
            *name_span,
            type_parameters.clone(),
            deriving.clone(),
            fields
                .iter()
                .map(|field| ShowMember {
                    key: field.name.clone(),
                    type_ref: field.type_ref.clone(),
                })
                .collect(),
            *span,
        )),
        _ => None,
    }
}

fn valid_candidate_identities(
    candidates: &[ShowCandidate],
    resolution: &TypedResolution<'_>,
    all_identities: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut valid = all_identities.clone();
    loop {
        let invalid = candidates
            .iter()
            .filter(|candidate| valid.contains(&candidate_identity(candidate)))
            .filter(|candidate| {
                candidate.members.iter().any(|member| {
                    member_evidence(candidate, member, resolution).is_none_or(|evidence| {
                        references_invalid_derived(&evidence.evidence, all_identities, &valid)
                    })
                })
            })
            .map(candidate_identity)
            .collect::<Vec<_>>();
        if invalid.is_empty() {
            return valid;
        }
        for identity in invalid {
            valid.remove(&identity);
        }
    }
}

fn collect_member_issues(
    candidates: &[ShowCandidate],
    resolution: &TypedResolution<'_>,
    all_identities: &BTreeSet<String>,
    valid: &BTreeSet<String>,
) -> Vec<DerivedInstanceIssue> {
    candidates
        .iter()
        .filter(|candidate| !valid.contains(&candidate_identity(candidate)))
        .filter_map(|candidate| {
            candidate.members.iter().find_map(|member| {
                let evidence = member_evidence(candidate, member, resolution);
                (evidence.as_ref().is_none_or(|evidence| {
                    references_invalid_derived(&evidence.evidence, all_identities, valid)
                }))
                .then(|| DerivedInstanceIssue::UnsupportedDerivedMember {
                    trait_name: candidate.trait_name.clone(),
                    member_type: derived_member_type(resolution, &member.type_ref),
                    primary: type_ref_span(&member.type_ref),
                    declaration: candidate.origin,
                })
            })
        })
        .collect()
}

fn typed_instance(
    candidate: ShowCandidate,
    resolution: &TypedResolution<'_>,
    all_identities: &BTreeSet<String>,
    valid: &BTreeSet<String>,
) -> TypedInstance {
    let requirements = derived_display_requirements_for_parts(
        &candidate.type_parameters,
        candidate.members.iter().map(|member| &member.type_ref),
        &candidate.trait_name,
    );
    let canonical_head = canonical_candidate_head(&candidate);
    let payload_evidence = candidate
        .members
        .iter()
        .filter_map(|member| member_evidence(&candidate, member, resolution))
        .filter(|evidence| !references_invalid_derived(&evidence.evidence, all_identities, valid))
        .collect();
    TypedInstance {
        identity: canonical_instance_identity(&candidate.trait_name, &canonical_head),
        trait_identity: candidate.trait_name.clone(),
        trait_name: candidate.trait_name.clone(),
        type_parameters: candidate.type_parameters.clone(),
        arguments: vec![TypedType::Named {
            name: candidate.name,
            arguments: candidate
                .type_parameters
                .iter()
                .map(|parameter| named(&parameter.name))
                .collect(),
        }],
        argument_identities: vec![canonical_head.clone()],
        type_identity: Some(canonical_head),
        constraint_identities: vec![Some(candidate.trait_identity); requirements.len()],
        constraints: requirements,
        supertrait_count: 0,
        origin: candidate.origin,
        implementation: TypedInstanceImplementation::DerivedShow {
            adt_symbol: candidate.symbol,
            payload_evidence,
        },
    }
}

fn member_evidence(
    candidate: &ShowCandidate,
    member: &ShowMember,
    resolution: &TypedResolution<'_>,
) -> Option<TypedShowPayloadEvidence> {
    let requirements = derived_display_requirements_for_parts(
        &candidate.type_parameters,
        candidate.members.iter().map(|member| &member.type_ref),
        &candidate.trait_name,
    );
    let constraint = TypedConstraint {
        name: candidate.trait_name.clone(),
        arguments: vec![derived_member_type(resolution, &member.type_ref)],
    };
    let evidence = crate::typed::call_evidence::select_derived_instance_evidence(
        &constraint,
        &candidate.trait_identity,
        &requirements,
        resolution,
    )?;
    let binders = candidate
        .type_parameters
        .iter()
        .enumerate()
        .map(|(index, parameter)| (parameter.name.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    Some(TypedShowPayloadEvidence {
        variant_symbol: member.key.clone(),
        type_identity: super::canonical_type_ref(&member.type_ref, resolution, &binders)
            .unwrap_or_else(|| render_type_ref(resolution, &member.type_ref)),
        evidence,
    })
}

fn derived_member_type(resolution: &TypedResolution<'_>, type_ref: &TypeRef) -> TypedType {
    match type_ref {
        TypeRef::Named {
            name,
            arguments,
            span,
        } => {
            let expanded = resolution.semantic_value_from_type_ref(type_ref).type_ref;
            if !matches!(
                &expanded,
                TypedType::Named { .. } | TypedType::ExternalNamed { .. }
            ) {
                return expanded;
            }
            let arguments = arguments
                .iter()
                .map(|argument| derived_member_type(resolution, argument))
                .collect();
            let imported_canonical =
                resolution
                    .target(*span, SymbolNamespace::Type)
                    .and_then(|target| {
                        resolution
                            .resolved()
                            .imports
                            .iter()
                            .find(|import| import.symbol == target)
                            .filter(|import| {
                                matches!(
                                    import.export.declaration_kind.as_deref(),
                                    Some(
                                        "type"
                                            | "opaque-type"
                                            | "newtype"
                                            | "struct"
                                            | "opaque-struct"
                                    )
                                )
                            })
                            .map(|import| import.export.symbol.clone())
                    });
            if imported_canonical.is_none()
                && matches!(
                    &expanded,
                    TypedType::Named {
                        name: expanded_name,
                        ..
                    } if expanded_name != name
                )
            {
                return expanded;
            }
            match imported_canonical {
                Some(canonical) => TypedType::ExternalNamed {
                    name: name.clone(),
                    canonical,
                    arguments,
                },
                None => TypedType::Named {
                    name: name.clone(),
                    arguments,
                },
            }
        }
        TypeRef::Record { closed, fields, .. } => TypedType::Record {
            closed: *closed,
            fields: fields
                .iter()
                .map(|field| TypedRecordField {
                    name: field.name.clone(),
                    optional: field.optional,
                    type_ref: derived_member_type(resolution, &field.type_ref),
                })
                .collect(),
        },
        TypeRef::Tuple { elements, .. } => TypedType::Tuple {
            elements: elements
                .iter()
                .map(|element| derived_member_type(resolution, element))
                .collect(),
        },
        TypeRef::Function {
            parameter, result, ..
        } => TypedType::Function {
            parameter: Box::new(derived_member_type(resolution, parameter)),
            result: Box::new(derived_member_type(resolution, result)),
        },
        TypeRef::Hole { .. } => TypedType::Hole,
    }
}

fn references_invalid_derived(
    evidence: &crate::TypedInstanceEvidence,
    all_identities: &BTreeSet<String>,
    valid: &BTreeSet<String>,
) -> bool {
    let arguments = match evidence {
        crate::TypedInstanceEvidence::Local {
            identity,
            evidence_arguments,
            ..
        } => {
            if all_identities.contains(identity) && !valid.contains(identity) {
                return true;
            }
            evidence_arguments
        }
        crate::TypedInstanceEvidence::Imported {
            evidence_arguments, ..
        }
        | crate::TypedInstanceEvidence::Standard {
            evidence_arguments, ..
        } => evidence_arguments,
        crate::TypedInstanceEvidence::Parameter { .. } => return false,
    };
    arguments
        .iter()
        .any(|argument| references_invalid_derived(&argument.evidence, all_identities, valid))
}

fn candidate_identity(candidate: &ShowCandidate) -> String {
    canonical_instance_identity(&candidate.trait_name, &canonical_candidate_head(candidate))
}

fn canonical_candidate_head(candidate: &ShowCandidate) -> String {
    if candidate.type_parameters.is_empty() {
        candidate.symbol.clone()
    } else {
        format!(
            "{}<{}>",
            candidate.symbol,
            (0..candidate.type_parameters.len())
                .map(|index| format!("${index}"))
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

pub(crate) fn derived_display_requirements(
    declaration: &SurfaceDecl,
    trait_name: &str,
) -> Vec<TypedConstraint> {
    let (type_parameters, type_refs): (&[TypeParameter], Vec<&TypeRef>) = match declaration {
        SurfaceDecl::Type {
            type_parameters,
            variants,
            ..
        } => (
            type_parameters,
            variants
                .iter()
                .filter_map(|variant| variant.payload.as_ref())
                .collect(),
        ),
        SurfaceDecl::Newtype {
            type_parameters,
            representation,
            ..
        } => (type_parameters, vec![representation]),
        SurfaceDecl::Struct {
            type_parameters,
            fields,
            ..
        } => (
            type_parameters,
            fields.iter().map(|field| &field.type_ref).collect(),
        ),
        _ => return Vec::new(),
    };
    derived_display_requirements_for_parts(type_parameters, type_refs, trait_name)
}

fn derived_display_requirements_for_parts<'a>(
    type_parameters: &[TypeParameter],
    type_refs: impl IntoIterator<Item = &'a TypeRef>,
    trait_name: &str,
) -> Vec<TypedConstraint> {
    let mut used = BTreeSet::new();
    for type_ref in type_refs {
        collect_type_parameters(type_ref, &mut used);
    }
    type_parameters
        .iter()
        .filter(|parameter| used.contains(&parameter.name))
        .map(|parameter| TypedConstraint {
            name: trait_name.to_owned(),
            arguments: vec![named(&parameter.name)],
        })
        .collect()
}

fn collect_type_parameters(type_ref: &TypeRef, output: &mut BTreeSet<String>) {
    match type_ref {
        TypeRef::Named {
            name, arguments, ..
        } => {
            if arguments.is_empty() {
                output.insert(name.clone());
            }
            for argument in arguments {
                collect_type_parameters(argument, output);
            }
        }
        TypeRef::Record { fields, .. } => {
            for field in fields {
                collect_type_parameters(&field.type_ref, output);
            }
        }
        TypeRef::Tuple { elements, .. } => {
            for element in elements {
                collect_type_parameters(element, output);
            }
        }
        TypeRef::Function {
            parameter, result, ..
        } => {
            collect_type_parameters(parameter, output);
            collect_type_parameters(result, output);
        }
        TypeRef::Hole { .. } => {}
    }
}

fn named(name: &str) -> TypedType {
    TypedType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}

fn type_ref_span(type_ref: &TypeRef) -> ByteSpan {
    match type_ref {
        TypeRef::Named { span, .. }
        | TypeRef::Hole { span }
        | TypeRef::Record { span, .. }
        | TypeRef::Tuple { span, .. }
        | TypeRef::Function { span, .. } => *span,
    }
}

fn render_type_ref(resolution: &TypedResolution<'_>, type_ref: &TypeRef) -> String {
    crate::TypeDocument::from_typed_type(&derived_member_type(resolution, type_ref))
        .render(crate::TypeRenderOptions::default())
}
