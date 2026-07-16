use crate::{ResolvedModule, SymbolKind, TypedInstance, TypedInstanceImplementation, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceDecl, TypeRef};
use std::collections::BTreeSet;

use super::{canonical_instance_identity, DerivedInstanceIssue, TypedResolution};

mod evidence;

use evidence::{payload_evidence, PayloadSupport};

pub(super) struct DerivedShowAnalysis {
    pub(super) instances: Vec<TypedInstance>,
    pub(super) issues: Vec<DerivedInstanceIssue>,
}

struct ShowCandidate {
    name: String,
    symbol: String,
    origin: ByteSpan,
    payloads: Vec<ShowPayload>,
}

pub(super) struct ShowPayload {
    variant_symbol: String,
    type_ref: TypeRef,
}

pub(super) fn analyze_derived_show(
    resolved: &ResolvedModule,
    resolution: &TypedResolution<'_>,
) -> DerivedShowAnalysis {
    let (candidates, mut issues) = collect_candidates(resolved, resolution);
    let valid = valid_candidate_symbols(&candidates, resolution);
    collect_payload_issues(&candidates, resolution, &valid, &mut issues);
    let instances = candidates
        .into_iter()
        .filter(|candidate| valid.contains(&candidate.symbol))
        .map(|candidate| typed_instance(candidate, resolution, &valid))
        .collect();
    DerivedShowAnalysis { instances, issues }
}

fn collect_candidates(
    resolved: &ResolvedModule,
    resolution: &TypedResolution<'_>,
) -> (Vec<ShowCandidate>, Vec<DerivedInstanceIssue>) {
    let mut candidates = Vec::new();
    let mut issues = Vec::new();
    for declaration in &resolved.declarations {
        let SurfaceDecl::Type {
            name,
            name_span,
            type_parameters,
            deriving,
            variants,
            span,
            ..
        } = declaration
        else {
            continue;
        };
        if !deriving.iter().any(|trait_name| trait_name == "Show") {
            continue;
        }
        if !type_parameters.is_empty() {
            issues.push(DerivedInstanceIssue::UnsupportedGenericShow {
                type_name: name.clone(),
                primary: *name_span,
                declaration: *span,
            });
            continue;
        }
        let Some(symbol) = resolution
            .declaration_symbol(*name_span, SymbolKind::Type)
            .and_then(|symbol| symbol.canonical.clone())
        else {
            continue;
        };
        candidates.push(ShowCandidate {
            name: name.clone(),
            symbol,
            origin: *span,
            payloads: variants
                .iter()
                .filter_map(|variant| {
                    let type_ref = variant.payload.clone()?;
                    let variant_symbol = resolution
                        .declaration_symbol(variant.name_span, SymbolKind::Constructor)?
                        .canonical
                        .clone()?;
                    Some(ShowPayload {
                        variant_symbol,
                        type_ref,
                    })
                })
                .collect(),
        });
    }
    (candidates, issues)
}

fn valid_candidate_symbols(
    candidates: &[ShowCandidate],
    resolution: &TypedResolution<'_>,
) -> BTreeSet<String> {
    let mut valid = candidates
        .iter()
        .map(|candidate| candidate.symbol.clone())
        .collect::<BTreeSet<_>>();
    loop {
        let invalid = candidates
            .iter()
            .filter(|candidate| valid.contains(&candidate.symbol))
            .filter(|candidate| {
                candidate.payloads.iter().any(|payload| {
                    !matches!(
                        payload_evidence(resolution, payload, &valid),
                        PayloadSupport::Supported(_)
                    )
                })
            })
            .map(|candidate| candidate.symbol.clone())
            .collect::<Vec<_>>();
        if invalid.is_empty() {
            return valid;
        }
        for symbol in invalid {
            valid.remove(&symbol);
        }
    }
}

fn collect_payload_issues(
    candidates: &[ShowCandidate],
    resolution: &TypedResolution<'_>,
    valid: &BTreeSet<String>,
    issues: &mut Vec<DerivedInstanceIssue>,
) {
    for candidate in candidates {
        if valid.contains(&candidate.symbol) {
            continue;
        }
        if let Some((payload, label)) = candidate.payloads.iter().find_map(|payload| {
            matches!(
                payload_evidence(resolution, payload, valid),
                PayloadSupport::Unsupported
            )
            .then(|| (payload, type_ref_label(&payload.type_ref)))
        }) {
            issues.push(DerivedInstanceIssue::UnsupportedShowPayload {
                payload_name: label,
                primary: type_ref_span(&payload.type_ref),
                declaration: candidate.origin,
            });
        }
    }
}

fn typed_instance(
    candidate: ShowCandidate,
    resolution: &TypedResolution<'_>,
    valid: &BTreeSet<String>,
) -> TypedInstance {
    let identity = canonical_instance_identity("Show", &candidate.symbol);
    let payload_evidence = candidate
        .payloads
        .iter()
        .filter_map(
            |payload| match payload_evidence(resolution, payload, valid) {
                PayloadSupport::Supported(evidence) => Some(evidence),
                PayloadSupport::Unsupported | PayloadSupport::Unresolved => None,
            },
        )
        .collect();
    TypedInstance {
        identity,
        trait_identity: "Show".to_owned(),
        trait_name: "Show".to_owned(),
        type_parameters: Vec::new(),
        arguments: vec![TypedType::Named {
            name: candidate.name,
            arguments: Vec::new(),
        }],
        argument_identities: vec![candidate.symbol.clone()],
        type_identity: Some(candidate.symbol.clone()),
        constraints: Vec::new(),
        constraint_identities: Vec::new(),
        supertrait_count: 0,
        origin: candidate.origin,
        implementation: TypedInstanceImplementation::DerivedShow {
            adt_symbol: candidate.symbol,
            payload_evidence,
        },
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

fn type_ref_label(type_ref: &TypeRef) -> String {
    match type_ref {
        TypeRef::Named {
            name, arguments, ..
        } if arguments.is_empty() => name.clone(),
        TypeRef::Named {
            name, arguments, ..
        } => format!(
            "{}<{}>",
            name,
            arguments
                .iter()
                .map(type_ref_label)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        TypeRef::Hole { .. } => "unknown".to_owned(),
        TypeRef::Record { .. } => "record".to_owned(),
        TypeRef::Tuple { .. } => "tuple".to_owned(),
        TypeRef::Function { .. } => "function".to_owned(),
    }
}
