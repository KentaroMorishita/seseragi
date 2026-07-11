use crate::{
    ResolvedModule, SymbolKind, SymbolNamespace, TypedInstance, TypedInstanceImplementation,
    TypedType,
};
use seseragi_syntax::{ByteSpan, SurfaceDecl, TypeRef};
use std::collections::BTreeSet;

use super::{DerivedInstanceIssue, TypedResolution};

const SHOW_PAYLOAD_IDENTITIES: &[&str] = &[
    "std/prelude::String",
    "std/prelude::ConsoleError",
    "std/prelude::StdinError",
];

pub(super) struct DerivedShowAnalysis {
    pub(super) instances: Vec<TypedInstance>,
    pub(super) issues: Vec<DerivedInstanceIssue>,
}

struct ShowCandidate {
    name: String,
    symbol: String,
    origin: ByteSpan,
    payloads: Vec<TypeRef>,
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
        .map(typed_instance)
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
                .filter_map(|variant| variant.payload.clone())
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
                    payload_support(resolution, payload, &valid) != PayloadSupport::Supported
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
            (payload_support(resolution, payload, valid) == PayloadSupport::Unsupported)
                .then(|| (payload, type_ref_label(payload)))
        }) {
            issues.push(DerivedInstanceIssue::UnsupportedShowPayload {
                payload_name: label,
                primary: type_ref_span(payload),
                declaration: candidate.origin,
            });
        }
    }
}

fn typed_instance(candidate: ShowCandidate) -> TypedInstance {
    TypedInstance {
        trait_name: "Show".to_owned(),
        head: TypedType::Named {
            name: candidate.name,
            arguments: Vec::new(),
        },
        type_identity: candidate.symbol.clone(),
        constraints: Vec::new(),
        origin: candidate.origin,
        implementation: TypedInstanceImplementation::DerivedShow {
            adt_symbol: candidate.symbol,
        },
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PayloadSupport {
    Supported,
    Unsupported,
    Unresolved,
}

fn payload_support(
    resolution: &TypedResolution<'_>,
    payload: &TypeRef,
    local_show_instances: &BTreeSet<String>,
) -> PayloadSupport {
    let TypeRef::Named {
        arguments, span, ..
    } = payload
    else {
        return PayloadSupport::Unsupported;
    };
    if !arguments.is_empty() {
        return PayloadSupport::Unsupported;
    }
    let Some(symbol) = resolution
        .target(*span, SymbolNamespace::Type)
        .and_then(|target| resolution.symbol(target))
        .and_then(|symbol| symbol.canonical.as_deref())
    else {
        return PayloadSupport::Unresolved;
    };
    if SHOW_PAYLOAD_IDENTITIES.contains(&symbol) || local_show_instances.contains(symbol) {
        PayloadSupport::Supported
    } else {
        PayloadSupport::Unsupported
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
