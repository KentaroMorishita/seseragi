use crate::{
    ResolvedModule, SymbolKind, SymbolNamespace, TypedConstraint, TypedInstance,
    TypedInstanceImplementation, TypedRecordField, TypedShowPayloadEvidence, TypedType,
};
use seseragi_syntax::{ByteSpan, SurfaceDecl, TypeParameter, TypeRef};
use std::collections::{BTreeMap, BTreeSet};

use super::{canonical_instance_identity, DerivedInstanceIssue, TypedResolution};

const DERIVED_TRAITS: [(&str, &str); 7] = [
    ("Eq", "std/prelude::Eq"),
    ("Ord", "std/prelude::Ord"),
    ("Hash", "std/prelude::Hash"),
    ("Show", "std/prelude::Show"),
    ("Debug", "std/prelude::Debug"),
    ("JsonEncode", "std/prelude::JsonEncode"),
    ("JsonDecode", "std/prelude::JsonDecode"),
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
    transparent_newtype: bool,
    requirements: Vec<TypedConstraint>,
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
        let Some((
            name,
            name_span,
            type_parameters,
            deriving,
            members,
            origin,
            transparent_newtype,
        )) = display_declaration(declaration, resolution)
        else {
            continue;
        };
        let Some(symbol) = resolution
            .declaration_symbol(name_span, SymbolKind::Type)
            .and_then(|symbol| symbol.canonical.clone())
        else {
            continue;
        };
        for (trait_name, trait_identity) in DERIVED_TRAITS {
            if deriving.iter().any(|derived| derived == trait_name) {
                candidates.push(ShowCandidate {
                    requirements: derived_display_requirements(declaration, trait_name, resolution),
                    trait_name: trait_name.to_owned(),
                    trait_identity: trait_identity.to_owned(),
                    name: name.clone(),
                    symbol: symbol.clone(),
                    origin,
                    type_parameters: type_parameters.clone(),
                    members: members.clone(),
                    transparent_newtype,
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
    bool,
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
            false,
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
            true,
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
            false,
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
                missing_eq_evidence(candidate, resolution, all_identities, &valid)
                    || candidate.members.iter().any(|member| {
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
            if missing_eq_evidence(candidate, resolution, all_identities, valid) {
                return Some(DerivedInstanceIssue::UnsupportedDerivedMember {
                    trait_name: "Eq".to_owned(),
                    member_type: candidate.requirements[0].arguments[0].clone(),
                    primary: candidate.origin,
                    declaration: candidate.origin,
                });
            }
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

fn missing_eq_evidence(
    candidate: &ShowCandidate,
    resolution: &TypedResolution<'_>,
    all: &BTreeSet<String>,
    valid: &BTreeSet<String>,
) -> bool {
    if !matches!(candidate.trait_name.as_str(), "Ord" | "Hash") {
        return false;
    }
    // A conditional explicit Eq instance is valid here even when its own
    // requirements differ from Hash/Ord payload requirements. The generated
    // factory requires Eq<Self>, so callers must still supply that evidence.
    let head = canonical_candidate_head(candidate);
    if resolution
        .resolved()
        .declarations
        .iter()
        .any(|declaration| {
            let SurfaceDecl::Instance {
                trait_name_span,
                arguments,
                type_parameters,
                ..
            } = declaration
            else {
                return false;
            };
            let is_eq = resolution
                .target(*trait_name_span, SymbolNamespace::Trait)
                .and_then(|symbol| resolution.symbol(symbol))
                .is_some_and(|symbol| symbol.canonical.as_deref() == Some("std/prelude::Eq"));
            let [argument] = arguments.as_slice() else {
                return false;
            };
            let binders = type_parameters
                .iter()
                .enumerate()
                .map(|(index, parameter)| (parameter.name.as_str(), index))
                .collect();
            is_eq
                && super::canonical_type_ref(argument, resolution, &binders).as_deref()
                    == Some(head.as_str())
        })
    {
        return false;
    }
    let requirements = candidate
        .type_parameters
        .iter()
        .map(|parameter| TypedConstraint {
            name: "Eq".to_owned(),
            arguments: vec![named(&parameter.name)],
        })
        .collect::<Vec<_>>();
    crate::typed::call_evidence::select_derived_instance_evidence(
        &candidate.requirements[0],
        "std/prelude::Eq",
        &requirements,
        resolution,
    )
    .is_none_or(|evidence| references_invalid_derived(&evidence, all, valid))
}

fn typed_instance(
    candidate: ShowCandidate,
    resolution: &TypedResolution<'_>,
    all_identities: &BTreeSet<String>,
    valid: &BTreeSet<String>,
) -> TypedInstance {
    let requirements = candidate.requirements.clone();
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
        constraint_identities: requirements
            .iter()
            .map(|required| Some(format!("std/prelude::{}", required.name)))
            .collect(),
        constraints: requirements,
        supertrait_count: usize::from(candidate.trait_name == "Ord"),
        origin: candidate.origin,
        implementation: if matches!(candidate.trait_name.as_str(), "JsonEncode" | "JsonDecode") {
            TypedInstanceImplementation::DerivedJson {
                adt_symbol: candidate.symbol,
                payload_evidence,
                transparent_newtype: candidate.transparent_newtype,
            }
        } else if matches!(candidate.trait_name.as_str(), "Eq" | "Ord" | "Hash") {
            TypedInstanceImplementation::DerivedStructural {
                adt_symbol: candidate.symbol,
                payload_evidence,
                transparent_newtype: candidate.transparent_newtype,
            }
        } else {
            TypedInstanceImplementation::DerivedShow {
                adt_symbol: candidate.symbol,
                payload_evidence,
            }
        },
    }
}

fn member_evidence(
    candidate: &ShowCandidate,
    member: &ShowMember,
    resolution: &TypedResolution<'_>,
) -> Option<TypedShowPayloadEvidence> {
    let requirements = candidate.requirements.clone();
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
        TypeRef::RequirementMerge { operands, .. } => {
            crate::typed::type_ref::normalize_requirement_merge(
                operands
                    .iter()
                    .map(|operand| derived_member_type(resolution, operand))
                    .collect(),
            )
        }
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
    resolution: &TypedResolution<'_>,
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
    let mut requirements = derived_display_requirements_for_parts(
        type_parameters,
        type_refs.iter().copied(),
        trait_name,
        resolution,
    );
    if trait_name == "Hash" {
        let parameters = type_parameters
            .iter()
            .map(|parameter| parameter.name.clone())
            .collect::<BTreeSet<_>>();
        for member in &type_refs {
            collect_nested_hash_eq(
                &derived_member_type(resolution, member),
                &parameters,
                resolution,
                &mut BTreeSet::new(),
                &mut requirements,
            );
        }
    }
    if matches!(trait_name, "Ord" | "Hash") {
        let name = match declaration {
            SurfaceDecl::Type { name, .. }
            | SurfaceDecl::Struct { name, .. }
            | SurfaceDecl::Newtype { name, .. } => name,
            _ => unreachable!(),
        };
        requirements.insert(
            0,
            TypedConstraint {
                name: "Eq".to_owned(),
                arguments: vec![TypedType::Named {
                    name: name.clone(),
                    arguments: type_parameters
                        .iter()
                        .map(|parameter| named(&parameter.name))
                        .collect(),
                }],
            },
        );
    }
    let mut unique = Vec::new();
    for required in requirements {
        if !unique.contains(&required) {
            unique.push(required);
        }
    }
    unique
}

// A nested derived Hash dictionary also needs its own nominal Eq evidence.
// Keep those obligations as nominal constraints rather than assuming that
// Eq<Outer<A>> can be projected into an unrelated Eq<Inner<A>> dictionary.
fn collect_nested_hash_eq(
    type_ref: &TypedType,
    parameters: &BTreeSet<String>,
    resolution: &TypedResolution<'_>,
    visiting: &mut BTreeSet<String>,
    requirements: &mut Vec<TypedConstraint>,
) {
    let (name, arguments) = match type_ref {
        TypedType::Named { name, arguments }
        | TypedType::ExternalNamed {
            name, arguments, ..
        } => (name, arguments),
        TypedType::Tuple { elements } => {
            for element in elements {
                collect_nested_hash_eq(element, parameters, resolution, visiting, requirements);
            }
            return;
        }
        _ => return,
    };
    if !has_parameter(type_ref, parameters) {
        return;
    }
    if let TypedType::ExternalNamed { canonical, .. } = type_ref {
        for instance in &resolution.resolved().dependency_instances {
            if !matches!(
                instance.trait_identity.as_str(),
                "Hash" | "std/prelude::Hash"
            ) || instance.trait_name != "Hash"
                || !instance
                    .argument_identities
                    .first()
                    .is_some_and(|head| head.split('<').next() == Some(canonical.as_str()))
            {
                continue;
            }
            let substitutions = instance
                .type_parameters
                .iter()
                .zip(arguments)
                .map(|(parameter, argument)| (parameter.name.clone(), argument.clone()))
                .collect();
            for required in &instance.constraints {
                if required.name != "Eq" {
                    continue;
                }
                let types = required
                    .arguments
                    .iter()
                    .filter_map(|argument| {
                        resolution.semantic_value_from_imported_type(
                            argument.clone(),
                            &instance.provider_module,
                            &instance.type_parameters,
                        )
                    })
                    .map(|value| {
                        crate::typed::functions::substitute_type_parameters(
                            &value.type_ref,
                            &substitutions,
                        )
                    })
                    .collect::<Vec<_>>();
                if types.len() == required.arguments.len() {
                    requirements.push(TypedConstraint {
                        name: "Eq".to_owned(),
                        arguments: types,
                    });
                }
            }
        }
        return;
    }
    let declaration = resolution
        .resolved()
        .declarations
        .iter()
        .find_map(|declaration| {
            let parts = display_declaration(declaration, resolution)?;
            (parts.0 == *name && parts.3.iter().any(|trait_name| trait_name == "Hash"))
                .then_some(parts)
        });
    if let Some((_, _, type_parameters, _, members, _, _)) = declaration {
        let required = TypedConstraint {
            name: "Eq".to_owned(),
            arguments: vec![type_ref.clone()],
        };
        if crate::typed::call_evidence::select_derived_instance_evidence(
            &required,
            "std/prelude::Eq",
            &[],
            resolution,
        )
        .is_none()
        {
            requirements.push(required);
        }
        if !visiting.insert(name.clone()) {
            return;
        }
        let substitutions = type_parameters
            .iter()
            .zip(arguments)
            .map(|(parameter, argument)| (parameter.name.clone(), argument.clone()))
            .collect();
        for member in members {
            let member = crate::typed::functions::substitute_type_parameters(
                &derived_member_type(resolution, &member.type_ref),
                &substitutions,
            );
            collect_nested_hash_eq(&member, parameters, resolution, visiting, requirements);
        }
        visiting.remove(name);
    } else {
        for argument in arguments {
            collect_nested_hash_eq(argument, parameters, resolution, visiting, requirements);
        }
    }
}

fn has_parameter(type_ref: &TypedType, parameters: &BTreeSet<String>) -> bool {
    match type_ref {
        TypedType::Named { name, arguments } => {
            parameters.contains(name)
                || arguments
                    .iter()
                    .any(|argument| has_parameter(argument, parameters))
        }
        TypedType::ExternalNamed { arguments, .. } => arguments
            .iter()
            .any(|argument| has_parameter(argument, parameters)),
        TypedType::Tuple { elements } => elements
            .iter()
            .any(|element| has_parameter(element, parameters)),
        TypedType::Record { fields, .. } => fields
            .iter()
            .any(|field| has_parameter(&field.type_ref, parameters)),
        _ => false,
    }
}

fn derived_display_requirements_for_parts<'a>(
    type_parameters: &[TypeParameter],
    type_refs: impl IntoIterator<Item = &'a TypeRef>,
    trait_name: &str,
    resolution: &TypedResolution<'_>,
) -> Vec<TypedConstraint> {
    let needs = derived_parameter_needs(resolution, trait_name);
    let mut used = BTreeSet::new();
    for type_ref in type_refs {
        collect_type_parameters(type_ref, &mut used, &needs);
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

// Least fixed point: a parameter used only through phantom or recursive
// nominal positions must not acquire an otherwise unnecessary constraint.
fn derived_parameter_needs(
    resolution: &TypedResolution<'_>,
    trait_name: &str,
) -> BTreeMap<String, BTreeSet<usize>> {
    if !matches!(trait_name, "Eq" | "Ord" | "Hash") {
        return BTreeMap::new();
    }
    let declarations = resolution
        .resolved()
        .declarations
        .iter()
        .filter_map(|declaration| {
            let (name, parameters, deriving, members) = match declaration {
                SurfaceDecl::Type {
                    name,
                    type_parameters,
                    deriving,
                    variants,
                    ..
                } => (
                    name,
                    type_parameters,
                    deriving,
                    variants
                        .iter()
                        .filter_map(|variant| variant.payload.as_ref())
                        .collect::<Vec<_>>(),
                ),
                SurfaceDecl::Struct {
                    name,
                    type_parameters,
                    deriving,
                    fields,
                    ..
                } => (
                    name,
                    type_parameters,
                    deriving,
                    fields.iter().map(|field| &field.type_ref).collect(),
                ),
                SurfaceDecl::Newtype {
                    name,
                    type_parameters,
                    deriving,
                    representation,
                    ..
                } => (name, type_parameters, deriving, vec![representation]),
                _ => return None,
            };
            deriving
                .iter()
                .any(|name| name == trait_name)
                .then_some((name, parameters, members))
        })
        .collect::<Vec<_>>();
    let mut needs = declarations
        .iter()
        .map(|(name, _, _)| ((*name).clone(), BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    // Imported unconditional nominal instances carry the same phantom information
    // in their public constraints; preserve it under local import aliases.
    for imported in &resolution.resolved().imports {
        if imported.export.namespace != "type" || !imported.in_scope {
            continue;
        }
        let unconditional = resolution
            .resolved()
            .dependency_instances
            .iter()
            .any(|instance| {
                instance.trait_name == trait_name
                    && (instance.trait_identity == trait_name
                        || instance.trait_identity == format!("std/prelude::{trait_name}"))
                    && instance.constraints.is_empty()
                    && instance.argument_identities.first().is_some_and(|head| {
                        head.split('<').next() == Some(imported.export.symbol.as_str())
                    })
            });
        if unconditional {
            needs.entry(imported.local_name.clone()).or_default();
        }
    }
    loop {
        let mut next = needs.clone();
        for (name, parameters, members) in &declarations {
            let mut used = BTreeSet::new();
            for member in members {
                collect_type_parameters(member, &mut used, &needs);
            }
            next.insert(
                (*name).clone(),
                parameters
                    .iter()
                    .enumerate()
                    .filter_map(|(index, parameter)| {
                        used.contains(&parameter.name).then_some(index)
                    })
                    .collect(),
            );
        }
        if next == needs {
            return needs;
        }
        needs = next;
    }
}

fn collect_type_parameters(
    type_ref: &TypeRef,
    output: &mut BTreeSet<String>,
    needs: &BTreeMap<String, BTreeSet<usize>>,
) {
    match type_ref {
        TypeRef::Named {
            name, arguments, ..
        } => {
            if arguments.is_empty() {
                output.insert(name.clone());
            }
            for (index, argument) in arguments.iter().enumerate() {
                if needs
                    .get(name)
                    .is_none_or(|required| required.contains(&index))
                {
                    collect_type_parameters(argument, output, needs);
                }
            }
        }
        TypeRef::Record { fields, .. } => {
            for field in fields {
                collect_type_parameters(&field.type_ref, output, needs);
            }
        }
        TypeRef::Tuple { elements, .. } => {
            for element in elements {
                collect_type_parameters(element, output, needs);
            }
        }
        TypeRef::Function {
            parameter, result, ..
        } => {
            collect_type_parameters(parameter, output, needs);
            collect_type_parameters(result, output, needs);
        }
        TypeRef::RequirementMerge { operands, .. } => {
            for operand in operands {
                collect_type_parameters(operand, output, needs);
            }
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
        | TypeRef::Function { span, .. }
        | TypeRef::RequirementMerge { span, .. } => *span,
    }
}

fn render_type_ref(resolution: &TypedResolution<'_>, type_ref: &TypeRef) -> String {
    crate::TypeDocument::from_typed_type(&derived_member_type(resolution, type_ref))
        .render(crate::TypeRenderOptions::default())
}
