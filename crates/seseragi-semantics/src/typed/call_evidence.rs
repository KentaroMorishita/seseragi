use super::TypedResolution;
use crate::{
    SymbolNamespace, TypedCallEvidence, TypedConstraint, TypedInstanceEvidence, TypedType,
};
use seseragi_syntax::{SurfaceConstraint, SurfaceDecl};
use std::collections::{BTreeMap, BTreeSet};

mod imported;
mod local;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ScopedCallEvidence {
    trait_identity: String,
    constraint: TypedConstraint,
    index: usize,
}

impl ScopedCallEvidence {
    pub(crate) fn index(&self) -> usize {
        self.index
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResolvedCallConstraint {
    pub(crate) trait_identity: String,
    pub(crate) constraint: TypedConstraint,
}

pub(crate) fn scoped_call_evidence(
    constraints: &[SurfaceConstraint],
    resolution: &TypedResolution<'_>,
) -> Vec<ScopedCallEvidence> {
    scoped_call_evidence_from(constraints, resolution, 0)
}

pub(crate) fn scoped_call_evidence_from(
    constraints: &[SurfaceConstraint],
    resolution: &TypedResolution<'_>,
    start_index: usize,
) -> Vec<ScopedCallEvidence> {
    constraints
        .iter()
        .enumerate()
        .filter_map(|(index, constraint)| {
            let target = resolution.target(constraint.name_span, SymbolNamespace::Trait)?;
            let trait_identity = resolution.symbol(target)?.canonical.clone()?;
            let typed = TypedConstraint {
                name: constraint.name.clone(),
                arguments: constraint
                    .arguments
                    .iter()
                    .map(super::type_ref::typed_type_from_type_ref)
                    .collect(),
            };
            let mut evidence = vec![ScopedCallEvidence {
                trait_identity,
                constraint: typed.clone(),
                index: start_index + index,
            }];
            evidence.extend(
                expanded_supertrait_constraints(constraint.name_span, &typed.arguments, resolution)
                    .into_iter()
                    .map(|supertrait| ScopedCallEvidence {
                        trait_identity: supertrait.trait_identity,
                        constraint: supertrait.constraint,
                        index: start_index + index,
                    }),
            );
            Some(evidence)
        })
        .flatten()
        .collect()
}

pub(crate) fn scoped_resolved_call_evidence(
    constraints: &[ResolvedCallConstraint],
    resolution: &TypedResolution<'_>,
    start_index: usize,
) -> Vec<ScopedCallEvidence> {
    constraints
        .iter()
        .enumerate()
        .flat_map(|(index, constraint)| {
            let mut evidence = vec![ScopedCallEvidence {
                trait_identity: constraint.trait_identity.clone(),
                constraint: constraint.constraint.clone(),
                index: start_index + index,
            }];
            if let Some(origin) = trait_origin(resolution, &constraint.trait_identity) {
                evidence.extend(
                    expanded_supertrait_constraints(
                        origin,
                        &constraint.constraint.arguments,
                        resolution,
                    )
                    .into_iter()
                    .map(|supertrait| ScopedCallEvidence {
                        trait_identity: supertrait.trait_identity,
                        constraint: supertrait.constraint,
                        index: start_index + index,
                    }),
                );
            }
            evidence
        })
        .collect()
}

pub(crate) fn direct_supertrait_constraints(
    trait_origin: seseragi_syntax::ByteSpan,
    arguments: &[TypedType],
    resolution: &TypedResolution<'_>,
) -> Vec<ResolvedCallConstraint> {
    direct_supertrait_constraints_inner(trait_origin, arguments, resolution).unwrap_or_default()
}

fn expanded_supertrait_constraints(
    origin: seseragi_syntax::ByteSpan,
    arguments: &[TypedType],
    resolution: &TypedResolution<'_>,
) -> Vec<ResolvedCallConstraint> {
    fn visit(
        direct: Vec<ResolvedCallConstraint>,
        resolution: &TypedResolution<'_>,
        visited: &mut BTreeSet<String>,
        output: &mut Vec<ResolvedCallConstraint>,
    ) {
        for supertrait in direct {
            if !visited.insert(supertrait.trait_identity.clone()) {
                continue;
            }
            let nested = direct_supertrait_constraints_for_identity(
                &supertrait.trait_identity,
                &supertrait.constraint.arguments,
                resolution,
            );
            visit(nested, resolution, visited, output);
            output.push(supertrait);
        }
    }

    let mut output = Vec::new();
    visit(
        direct_supertrait_constraints(origin, arguments, resolution),
        resolution,
        &mut BTreeSet::new(),
        &mut output,
    );
    output
}

fn direct_supertrait_constraints_inner(
    trait_origin: seseragi_syntax::ByteSpan,
    arguments: &[TypedType],
    resolution: &TypedResolution<'_>,
) -> Option<Vec<ResolvedCallConstraint>> {
    let symbol = resolution
        .target(trait_origin, SymbolNamespace::Trait)
        .and_then(|target| resolution.symbol(target))
        .or_else(|| {
            resolution.resolved().symbols.iter().find(|symbol| {
                symbol.namespace == SymbolNamespace::Trait && symbol.origin == trait_origin
            })
        })?;
    if let Some(identity) = symbol.canonical.as_deref() {
        if crate::prelude::trait_by_canonical(identity).is_some() {
            return Some(direct_supertrait_constraints_for_identity(
                identity, arguments, resolution,
            ));
        }
    }
    let (parameters, constraints) = resolution.resolved().declarations.iter().find_map(|decl| {
        let SurfaceDecl::Trait {
            name_span,
            type_parameters,
            constraints,
            ..
        } = decl
        else {
            return None;
        };
        (*name_span == symbol.origin).then_some((type_parameters, constraints))
    })?;
    if parameters.len() != arguments.len() {
        return None;
    }
    let substitutions = parameters
        .iter()
        .zip(arguments)
        .map(|(parameter, argument)| (parameter.name.clone(), argument.clone()))
        .collect::<BTreeMap<_, _>>();
    Some(
        constraints
            .iter()
            .filter_map(|constraint| {
                let target = resolution.target(constraint.name_span, SymbolNamespace::Trait)?;
                let trait_identity = resolution.symbol(target)?.canonical.clone()?;
                Some(ResolvedCallConstraint {
                    trait_identity,
                    constraint: TypedConstraint {
                        name: constraint.name.clone(),
                        arguments: constraint
                            .arguments
                            .iter()
                            .map(super::type_ref::typed_type_from_type_ref)
                            .map(|argument| {
                                super::functions::substitute_type_parameters(
                                    &argument,
                                    &substitutions,
                                )
                            })
                            .collect(),
                    },
                })
            })
            .collect(),
    )
}

fn direct_supertrait_constraints_for_identity(
    trait_identity: &str,
    arguments: &[TypedType],
    resolution: &TypedResolution<'_>,
) -> Vec<ResolvedCallConstraint> {
    if let Some(trait_spec) = crate::prelude::trait_by_canonical(trait_identity) {
        let Some(supertrait_name) = trait_spec.supertrait else {
            return Vec::new();
        };
        let Some(supertrait) = crate::prelude::trait_by_name(supertrait_name) else {
            return Vec::new();
        };
        if arguments.len() != 1 {
            return Vec::new();
        }
        return vec![ResolvedCallConstraint {
            trait_identity: supertrait.canonical.to_owned(),
            constraint: TypedConstraint {
                name: supertrait.name.to_owned(),
                arguments: arguments.to_vec(),
            },
        }];
    }
    trait_origin(resolution, trait_identity)
        .and_then(|origin| direct_supertrait_constraints_inner(origin, arguments, resolution))
        .unwrap_or_default()
}

fn trait_origin(
    resolution: &TypedResolution<'_>,
    trait_identity: &str,
) -> Option<seseragi_syntax::ByteSpan> {
    resolution.resolved().symbols.iter().find_map(|symbol| {
        (symbol.namespace == SymbolNamespace::Trait
            && symbol.canonical.as_deref() == Some(trait_identity))
        .then_some(symbol.origin)
    })
}

/// Selects evidence for constraints attached to a saturated function call.
///
/// This is deliberately a small registry boundary. The first implemented
/// standard instances are registered here; local and imported instance
/// search can extend this module without teaching expression typing about
/// compiler-private standard names.
pub(crate) fn select_call_evidence(
    constraints: &[TypedConstraint],
) -> Result<Vec<TypedCallEvidence>, TypedConstraint> {
    constraints
        .iter()
        .cloned()
        .map(|constraint| {
            let identity =
                standard_instance_identity(&constraint).ok_or_else(|| constraint.clone())?;
            Ok(TypedCallEvidence {
                constraint,
                evidence: TypedInstanceEvidence::Standard { identity },
            })
        })
        .collect()
}

pub(crate) fn select_function_call_evidence(
    constraints: &[TypedConstraint],
    trait_identities: &[Option<String>],
    resolution: &TypedResolution<'_>,
    scoped: &[ScopedCallEvidence],
) -> Result<Vec<TypedCallEvidence>, TypedConstraint> {
    constraints
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, constraint)| {
            let trait_identity = trait_identities.get(index).and_then(Option::as_deref);
            let evidence = match trait_identity {
                Some(trait_identity) => {
                    select_resolved_evidence(&constraint, trait_identity, resolution, scoped)
                }
                // Standard operations such as `reduce` do not resolve through
                // a source trait method, but their constraint can still be
                // satisfied by evidence supplied by a generic caller or by a
                // user instance. Prefer that materializable dictionary and
                // retain the dedicated standard operation ABI as the fallback.
                None => {
                    let trait_identity = format!("std/prelude::{}", constraint.name);
                    select_resolved_evidence(&constraint, &trait_identity, resolution, scoped)
                        .or_else(|| {
                            standard_instance_identity(&constraint)
                                .map(|identity| TypedInstanceEvidence::Standard { identity })
                        })
                }
            }
            .ok_or_else(|| constraint.clone())?;
            Ok(TypedCallEvidence {
                constraint,
                evidence,
            })
        })
        .collect()
}

fn select_resolved_evidence(
    constraint: &TypedConstraint,
    trait_identity: &str,
    resolution: &TypedResolution<'_>,
    scoped: &[ScopedCallEvidence],
) -> Option<TypedInstanceEvidence> {
    if let Some(parameter) = scoped.iter().find(|available| {
        available.trait_identity == trait_identity
            && available.constraint.arguments.len() == constraint.arguments.len()
            && available
                .constraint
                .arguments
                .iter()
                .zip(&constraint.arguments)
                .all(|(available, required)| {
                    super::semantic_types::semantic_values_are_compatible(
                        &resolution.semantic_value_from_typed_type(available),
                        &resolution.semantic_value_from_typed_type(required),
                    )
                })
    }) {
        return Some(TypedInstanceEvidence::Parameter {
            index: parameter.index,
        });
    }
    local::select_local_instance(trait_identity, constraint, resolution, scoped)
        .or_else(|| {
            imported::select_imported_instance(trait_identity, constraint, resolution, scoped)
        })
        .or_else(|| select_standard_instance(Some(trait_identity), constraint))
}

pub(super) fn select_standard_instance(
    trait_identity: Option<&str>,
    constraint: &TypedConstraint,
) -> Option<TypedInstanceEvidence> {
    if let Some(identity) = crate::standard::standard_module_instance(trait_identity, constraint) {
        return Some(TypedInstanceEvidence::Standard {
            identity: identity.to_owned(),
        });
    }
    if trait_identity
        .is_some_and(|identity| identity != format!("std/prelude::{}", constraint.name))
    {
        return None;
    }
    let identity = standard_instance_identity(constraint)?;
    // Arithmetic, equality, and collection instances currently lower through
    // dedicated operation ABIs rather than first-class dictionary values.
    // Only select a standard instance for dictionary passing when the runtime
    // actually exports the corresponding dictionary object.
    (identity.starts_with("Show<")
        || crate::prelude::standard_instance_by_identity(&identity).is_some())
    .then_some(TypedInstanceEvidence::Standard { identity })
}

fn standard_instance_identity(constraint: &TypedConstraint) -> Option<String> {
    if let [type_ref] = constraint.arguments.as_slice() {
        if let Some(instance) = crate::prelude::standard_instance(&constraint.name, type_ref) {
            return Some(instance.identity.to_owned());
        }
    }
    if let Some(identity) = show_instance_identity(constraint) {
        return Some(identity.to_owned());
    }
    if let Some(identity) = arithmetic_instance_identity(constraint) {
        return Some(identity.to_owned());
    }
    if let Some(identity) = equality_instance_identity(constraint) {
        return Some(identity.to_owned());
    }
    let [collection, element] = constraint.arguments.as_slice() else {
        return None;
    };
    let TypedType::Named { name, arguments } = collection else {
        return None;
    };
    if !matches!(
        arguments.as_slice(),
        [collection_element] if collection_element == element
    ) {
        return None;
    }
    match (constraint.name.as_str(), name.as_str()) {
        ("Iterable", "Array") => Some("std/array::Iterable".to_owned()),
        ("Iterable", "List") => Some("std/list::Iterable".to_owned()),
        ("Iterable", "Range") if named_type_is(element, "Int") => {
            Some("std/range::Iterable".to_owned())
        }
        ("Reducible", "Array") => Some("std/array::Reducible".to_owned()),
        ("Reducible", "List") => Some("std/list::Reducible".to_owned()),
        ("Reducible", "Range") if named_type_is(element, "Int") => {
            Some("std/range::Reducible".to_owned())
        }
        _ => None,
    }
}

fn show_instance_identity(constraint: &TypedConstraint) -> Option<&'static str> {
    let [value] = constraint.arguments.as_slice() else {
        return None;
    };
    if constraint.name != "Show" {
        return None;
    }
    let TypedType::Named { name, arguments } = value else {
        return None;
    };
    if !arguments.is_empty() {
        return None;
    }
    match name.as_str() {
        "Int" => Some("Show<std/prelude::Int>"),
        "String" => Some("Show<std/prelude::String>"),
        "ConsoleError" => Some("Show<std/prelude::ConsoleError>"),
        "StdinError" => Some("Show<std/prelude::StdinError>"),
        _ => None,
    }
}

pub(crate) fn select_iterable_evidence(
    collection: TypedType,
    trait_identity: Option<&str>,
    resolution: &TypedResolution<'_>,
    scoped: &[ScopedCallEvidence],
) -> Result<(TypedType, TypedCallEvidence), TypedConstraint> {
    let missing = || TypedConstraint {
        name: "Iterable".to_owned(),
        arguments: vec![collection.clone(), TypedType::Hole],
    };
    if let Some(trait_identity) = trait_identity {
        let scoped_matches = scoped
            .iter()
            .filter(|available| {
                available.trait_identity == trait_identity
                    && matches!(available.constraint.arguments.as_slice(), [available, _]
                    if super::semantic_types::semantic_values_are_compatible(
                        &resolution.semantic_value_from_typed_type(available),
                        &resolution.semantic_value_from_typed_type(&collection),
                    ))
            })
            .take(2)
            .collect::<Vec<_>>();
        if let [available] = scoped_matches.as_slice() {
            let element = available.constraint.arguments[1].clone();
            return Ok((
                element,
                TypedCallEvidence {
                    constraint: available.constraint.clone(),
                    evidence: TypedInstanceEvidence::Parameter {
                        index: available.index,
                    },
                },
            ));
        }
        if let Some((element, evidence)) = local::infer_local_functional_instance(
            trait_identity,
            "Iterable",
            &collection,
            resolution,
            scoped,
        ) {
            return Ok((
                element.clone(),
                TypedCallEvidence {
                    constraint: TypedConstraint {
                        name: "Iterable".to_owned(),
                        arguments: vec![collection, element],
                    },
                    evidence,
                },
            ));
        }
        if let Some((element, evidence)) = imported::infer_imported_functional_instance(
            trait_identity,
            "Iterable",
            &collection,
            resolution,
            scoped,
        ) {
            return Ok((
                element.clone(),
                TypedCallEvidence {
                    constraint: TypedConstraint {
                        name: "Iterable".to_owned(),
                        arguments: vec![collection, element],
                    },
                    evidence,
                },
            ));
        }
    }
    let element = standard_iterable_element_type(&collection).ok_or_else(missing)?;
    let constraint = TypedConstraint {
        name: "Iterable".to_owned(),
        arguments: vec![collection, element.clone()],
    };
    select_call_evidence(std::slice::from_ref(&constraint))
        .map(|mut evidence| (element, evidence.remove(0)))
        .map_err(|_| constraint)
}

pub(crate) fn select_binary_operator_evidence(
    trait_name: &str,
    left: TypedType,
    right: TypedType,
    trait_identity: Option<&str>,
    resolution: &TypedResolution<'_>,
    scoped: &[ScopedCallEvidence],
) -> Result<(TypedType, TypedCallEvidence), TypedConstraint> {
    let missing = || TypedConstraint {
        name: trait_name.to_owned(),
        arguments: vec![left.clone(), right.clone(), TypedType::Hole],
    };
    if let Some(trait_identity) = trait_identity {
        let scoped_matches = scoped
            .iter()
            .filter(|available| {
                available.trait_identity == trait_identity
                    && matches!(available.constraint.arguments.as_slice(), [available_left, available_right, _]
                        if super::semantic_types::semantic_values_are_compatible(
                            &resolution.semantic_value_from_typed_type(available_left),
                            &resolution.semantic_value_from_typed_type(&left),
                        ) && super::semantic_types::semantic_values_are_compatible(
                            &resolution.semantic_value_from_typed_type(available_right),
                            &resolution.semantic_value_from_typed_type(&right),
                        ))
            })
            .take(2)
            .collect::<Vec<_>>();
        if let [available] = scoped_matches.as_slice() {
            let output = available.constraint.arguments[2].clone();
            return Ok((
                output,
                TypedCallEvidence {
                    constraint: available.constraint.clone(),
                    evidence: TypedInstanceEvidence::Parameter {
                        index: available.index,
                    },
                },
            ));
        }
        if let Some((output, evidence)) = local::infer_local_binary_instance(
            trait_identity,
            trait_name,
            &left,
            &right,
            resolution,
            scoped,
        ) {
            return Ok((
                output.clone(),
                TypedCallEvidence {
                    constraint: TypedConstraint {
                        name: trait_name.to_owned(),
                        arguments: vec![left, right, output],
                    },
                    evidence,
                },
            ));
        }
        if let Some((output, evidence)) = imported::infer_imported_binary_instance(
            trait_identity,
            trait_name,
            &left,
            &right,
            resolution,
            scoped,
        ) {
            return Ok((
                output.clone(),
                TypedCallEvidence {
                    constraint: TypedConstraint {
                        name: trait_name.to_owned(),
                        arguments: vec![left, right, output],
                    },
                    evidence,
                },
            ));
        }
    }
    let output = standard_binary_output(trait_name, &left, &right).ok_or_else(missing)?;
    let constraint = TypedConstraint {
        name: trait_name.to_owned(),
        arguments: vec![left, right, output.clone()],
    };
    select_call_evidence(std::slice::from_ref(&constraint))
        .map(|mut evidence| (output, evidence.remove(0)))
        .map_err(|_| constraint)
}

/// Selects a concrete binary operator head from the known parts of an
/// expected curried function type. Holes are wildcards produced by generic
/// application inference; ambiguity is rejected instead of guessing a type.
pub(crate) fn select_binary_operator_reference_evidence(
    trait_name: &str,
    left: TypedType,
    right: TypedType,
    output: TypedType,
    trait_identity: Option<&str>,
    resolution: &TypedResolution<'_>,
    scoped: &[ScopedCallEvidence],
) -> Result<([TypedType; 3], TypedCallEvidence), TypedConstraint> {
    let requested = [left, right, output];
    let expected = requested
        .each_ref()
        .map(|type_ref| (!matches!(type_ref, TypedType::Hole)).then_some(type_ref));
    let missing = || TypedConstraint {
        name: trait_name.to_owned(),
        arguments: requested.to_vec(),
    };
    if let Some(trait_identity) = trait_identity {
        let scoped_matches = scoped
            .iter()
            .filter(|available| {
                available.trait_identity == trait_identity
                    && matches!(available.constraint.arguments.as_slice(), [left, right, output]
                    if partial_binary_head_matches(
                        &[left.clone(), right.clone(), output.clone()],
                        &expected,
                        resolution,
                    ))
            })
            .take(2)
            .collect::<Vec<_>>();
        if let [available] = scoped_matches.as_slice() {
            let head: [TypedType; 3] = available
                .constraint
                .arguments
                .clone()
                .try_into()
                .expect("binary operator constraint must have three arguments");
            return Ok((
                head,
                TypedCallEvidence {
                    constraint: available.constraint.clone(),
                    evidence: TypedInstanceEvidence::Parameter {
                        index: available.index,
                    },
                },
            ));
        }
        if let Some((head, evidence)) = local::infer_local_binary_instance_from_partial(
            trait_identity,
            trait_name,
            &expected,
            resolution,
            scoped,
        ) {
            return Ok((
                head.clone(),
                TypedCallEvidence {
                    constraint: TypedConstraint {
                        name: trait_name.to_owned(),
                        arguments: head.to_vec(),
                    },
                    evidence,
                },
            ));
        }
        if let Some((head, evidence)) = imported::infer_imported_binary_instance_from_partial(
            trait_identity,
            trait_name,
            &expected,
            resolution,
            scoped,
        ) {
            return Ok((
                head.clone(),
                TypedCallEvidence {
                    constraint: TypedConstraint {
                        name: trait_name.to_owned(),
                        arguments: head.to_vec(),
                    },
                    evidence,
                },
            ));
        }
    }
    let matches = standard_binary_heads(trait_name)
        .into_iter()
        .filter(|head| partial_binary_head_matches(head, &expected, resolution))
        .take(2)
        .collect::<Vec<_>>();
    let [head] = matches.as_slice() else {
        return Err(missing());
    };
    let constraint = TypedConstraint {
        name: trait_name.to_owned(),
        arguments: head.to_vec(),
    };
    let evidence = select_call_evidence(std::slice::from_ref(&constraint))
        .map_err(|_| constraint.clone())?
        .remove(0);
    Ok((head.clone(), evidence))
}

fn partial_binary_head_matches(
    candidate: &[TypedType],
    expected: &[Option<&TypedType>; 3],
    resolution: &TypedResolution<'_>,
) -> bool {
    candidate.iter().zip(expected).all(|(candidate, expected)| {
        expected.is_none_or(|expected| {
            super::semantic_types::semantic_values_are_compatible(
                &resolution.semantic_value_from_typed_type(expected),
                &resolution.semantic_value_from_typed_type(candidate),
            )
        })
    })
}

fn standard_binary_heads(trait_name: &str) -> Vec<[TypedType; 3]> {
    let mut heads = Vec::new();
    if matches!(trait_name, "Add" | "Sub" | "Mul" | "Div" | "Rem" | "Pow") {
        let int = named_type("Int");
        heads.push([int.clone(), int.clone(), int]);
    }
    if trait_name == "Add" {
        let string = named_type("String");
        heads.push([string.clone(), string.clone(), string]);
    }
    heads
}

fn named_type(name: &str) -> TypedType {
    TypedType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}

fn contains_declared_type_parameter(
    type_ref: &TypedType,
    parameters: &[seseragi_syntax::TypeParameter],
) -> bool {
    match type_ref {
        TypedType::Named { name, arguments } => {
            parameters.iter().any(|parameter| parameter.name == *name)
                || arguments
                    .iter()
                    .any(|argument| contains_declared_type_parameter(argument, parameters))
        }
        TypedType::ExternalNamed { arguments, .. } => arguments
            .iter()
            .any(|argument| contains_declared_type_parameter(argument, parameters)),
        TypedType::Record { fields, .. } => fields
            .iter()
            .any(|field| contains_declared_type_parameter(&field.type_ref, parameters)),
        TypedType::Tuple { elements } => elements
            .iter()
            .any(|element| contains_declared_type_parameter(element, parameters)),
        TypedType::Function { parameter, result } => {
            contains_declared_type_parameter(parameter, parameters)
                || contains_declared_type_parameter(result, parameters)
        }
        TypedType::Hole => false,
    }
}

fn standard_binary_output(
    trait_name: &str,
    left: &TypedType,
    right: &TypedType,
) -> Option<TypedType> {
    if trait_name == "Add" && named_type_is(left, "String") && named_type_is(right, "String") {
        return Some(left.clone());
    }
    matches!(trait_name, "Add" | "Sub" | "Mul" | "Div" | "Rem" | "Pow")
        .then(|| named_type_is(left, "Int") && named_type_is(right, "Int"))
        .filter(|matches| *matches)
        .map(|_| left.clone())
}

fn standard_iterable_element_type(collection: &TypedType) -> Option<TypedType> {
    let TypedType::Named { name, arguments } = collection else {
        return None;
    };
    match (name.as_str(), arguments.as_slice()) {
        ("Array", [element]) => Some(element.clone()),
        ("List", [element]) => Some(element.clone()),
        ("Range", [element]) if named_type_is(element, "Int") => Some(element.clone()),
        _ => None,
    }
}

fn named_type_is(type_ref: &TypedType, expected: &str) -> bool {
    matches!(type_ref, TypedType::Named { name, arguments } if name == expected && arguments.is_empty())
}

fn equality_instance_identity(constraint: &TypedConstraint) -> Option<&'static str> {
    let [value] = constraint.arguments.as_slice() else {
        return None;
    };
    if constraint.name != "Eq" {
        return None;
    }
    crate::prelude::standard_equality_instance(value).map(|instance| instance.identity)
}

fn arithmetic_instance_identity(constraint: &TypedConstraint) -> Option<&'static str> {
    let [left, right, output] = constraint.arguments.as_slice() else {
        return None;
    };
    let all_string = [left, right, output].iter().all(|type_ref| {
        matches!(type_ref, TypedType::Named { name, arguments } if name == "String" && arguments.is_empty())
    });
    if constraint.name == "Add" && all_string {
        return Some("std/string::Add");
    }
    let all_int = [left, right, output]
        .iter()
        .all(|type_ref| matches!(type_ref, TypedType::Named { name, arguments } if name == "Int" && arguments.is_empty()));
    if !all_int {
        return None;
    }
    match constraint.name.as_str() {
        "Add" => Some("std/int::Add"),
        "Sub" => Some("std/int::Sub"),
        "Mul" => Some("std/int::Mul"),
        "Div" => Some("std/int::Div"),
        "Rem" => Some("std/int::Rem"),
        "Pow" => Some("std/int::Pow"),
        _ => None,
    }
}

#[cfg(test)]
fn select_equality_evidence(
    operator: &str,
    left: TypedType,
    right: TypedType,
) -> Vec<TypedCallEvidence> {
    if !matches!(operator, "==" | "!=") || left != right {
        return Vec::new();
    }
    select_call_evidence(&[TypedConstraint {
        name: "Eq".to_owned(),
        arguments: vec![left],
    }])
    .unwrap_or_default()
}

pub(crate) fn select_binary_equality_evidence(
    left: TypedType,
    right: TypedType,
    trait_identity: Option<&str>,
    resolution: &TypedResolution<'_>,
    scoped: &[ScopedCallEvidence],
) -> Result<TypedCallEvidence, TypedConstraint> {
    let missing = || TypedConstraint {
        name: "Eq".to_owned(),
        arguments: vec![left.clone()],
    };
    if left != right || matches!(left, TypedType::Hole) {
        return Err(missing());
    }
    let constraint = missing();
    let evidence = trait_identity
        .and_then(|trait_identity| {
            select_resolved_evidence(&constraint, trait_identity, resolution, scoped)
        })
        .or_else(|| {
            standard_instance_identity(&constraint)
                .map(|identity| TypedInstanceEvidence::Standard { identity })
        })
        .ok_or_else(|| constraint.clone())?;
    Ok(TypedCallEvidence {
        constraint,
        evidence,
    })
}

/// Selects an Eq head from the known operand parts of an expected curried
/// function type. A scoped generic constraint can supply the missing operand
/// type when higher-order application has not inferred later arguments yet.
pub(crate) fn select_equality_operator_reference_evidence(
    left: TypedType,
    right: TypedType,
    trait_identity: Option<&str>,
    resolution: &TypedResolution<'_>,
    scoped: &[ScopedCallEvidence],
) -> Result<(TypedType, TypedCallEvidence), TypedConstraint> {
    let expected =
        [&left, &right].map(|type_ref| (!matches!(type_ref, TypedType::Hole)).then_some(type_ref));
    let missing_type = expected
        .iter()
        .find_map(|type_ref| (*type_ref).cloned())
        .unwrap_or(TypedType::Hole);
    let missing = || TypedConstraint {
        name: "Eq".to_owned(),
        arguments: vec![missing_type.clone()],
    };

    if let Some(trait_identity) = trait_identity {
        let scoped_matches = scoped
            .iter()
            .filter(|available| {
                available.trait_identity == trait_identity
                    && matches!(available.constraint.arguments.as_slice(), [value]
                    if expected.iter().all(|expected| expected.is_none_or(|expected| {
                        super::semantic_types::semantic_values_are_compatible(
                            &resolution.semantic_value_from_typed_type(value),
                            &resolution.semantic_value_from_typed_type(expected),
                        )
                    })))
            })
            .take(2)
            .collect::<Vec<_>>();
        if let [available] = scoped_matches.as_slice() {
            let [value] = available.constraint.arguments.as_slice() else {
                unreachable!("scoped equality match must have one argument");
            };
            return Ok((
                value.clone(),
                TypedCallEvidence {
                    constraint: available.constraint.clone(),
                    evidence: TypedInstanceEvidence::Parameter {
                        index: available.index,
                    },
                },
            ));
        }
    }

    let value = expected
        .iter()
        .find_map(|type_ref| (*type_ref).cloned())
        .ok_or_else(missing)?;
    if !expected.iter().all(|expected| {
        expected.is_none_or(|expected| {
            super::semantic_types::semantic_values_are_compatible(
                &resolution.semantic_value_from_typed_type(&value),
                &resolution.semantic_value_from_typed_type(expected),
            )
        })
    }) {
        return Err(missing());
    }
    let evidence = select_binary_equality_evidence(
        value.clone(),
        value.clone(),
        trait_identity,
        resolution,
        scoped,
    )?;
    Ok((value, evidence))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(name: &str) -> TypedType {
        TypedType::Named {
            name: name.to_owned(),
            arguments: Vec::new(),
        }
    }

    #[test]
    fn selects_the_standard_array_reducible_instance() {
        let int = named("Int");
        let evidence = select_call_evidence(&[TypedConstraint {
            name: "Reducible".to_owned(),
            arguments: vec![
                TypedType::Named {
                    name: "Array".to_owned(),
                    arguments: vec![int.clone()],
                },
                int,
            ],
        }])
        .expect("standard array instance");

        assert!(matches!(
            evidence.as_slice(),
            [TypedCallEvidence {
                evidence: TypedInstanceEvidence::Standard { identity },
                ..
            }] if identity == "std/array::Reducible"
        ));
    }

    #[test]
    fn selects_the_standard_int_range_reducible_instance() {
        let int = named("Int");
        let evidence = select_call_evidence(&[TypedConstraint {
            name: "Reducible".to_owned(),
            arguments: vec![
                TypedType::Named {
                    name: "Range".to_owned(),
                    arguments: vec![int.clone()],
                },
                int,
            ],
        }])
        .expect("standard range instance");

        assert!(matches!(
            evidence.as_slice(),
            [TypedCallEvidence {
                evidence: TypedInstanceEvidence::Standard { identity },
                ..
            }] if identity == "std/range::Reducible"
        ));
    }

    #[test]
    fn selects_standard_iterable_instances_by_collection_type() {
        let int = named("Int");
        for (collection, expected) in [
            (
                TypedType::Named {
                    name: "Array".to_owned(),
                    arguments: vec![int.clone()],
                },
                "std/array::Iterable",
            ),
            (
                TypedType::Named {
                    name: "Range".to_owned(),
                    arguments: vec![int.clone()],
                },
                "std/range::Iterable",
            ),
            (
                TypedType::Named {
                    name: "List".to_owned(),
                    arguments: vec![int.clone()],
                },
                "std/list::Iterable",
            ),
        ] {
            let evidence = select_call_evidence(&[TypedConstraint {
                name: "Iterable".to_owned(),
                arguments: vec![collection, int.clone()],
            }])
            .expect("standard Iterable evidence")
            .remove(0);
            assert!(matches!(
                evidence.evidence,
                TypedInstanceEvidence::Standard { identity } if identity == expected
            ));
        }
    }

    #[test]
    fn does_not_invent_evidence_for_an_unsupported_collection() {
        let int = named("Int");
        assert!(select_call_evidence(&[TypedConstraint {
            name: "Reducible".to_owned(),
            arguments: vec![named("Int"), int],
        }])
        .is_err());
    }

    #[test]
    fn selects_standard_int_add_evidence() {
        let evidence = select_call_evidence(&[TypedConstraint {
            name: "Add".to_owned(),
            arguments: vec![named("Int"), named("Int"), named("Int")],
        }])
        .expect("standard Int Add evidence");
        assert!(matches!(
            evidence.as_slice(),
            [TypedCallEvidence {
                constraint: TypedConstraint { name, arguments },
                evidence: TypedInstanceEvidence::Standard { identity },
            }] if name == "Add" && arguments.len() == 3 && identity == "std/int::Add"
        ));
    }

    #[test]
    fn selects_standard_string_add_evidence() {
        let evidence = select_call_evidence(&[TypedConstraint {
            name: "Add".to_owned(),
            arguments: vec![named("String"), named("String"), named("String")],
        }])
        .expect("standard String Add evidence");
        assert!(matches!(
            evidence.as_slice(),
            [TypedCallEvidence {
                constraint: TypedConstraint { name, arguments },
                evidence: TypedInstanceEvidence::Standard { identity },
            }] if name == "Add" && arguments.len() == 3 && identity == "std/string::Add"
        ));
    }

    #[test]
    fn selects_materializable_standard_show_evidence() {
        for (name, identity) in [
            ("Int", "Show<std/prelude::Int>"),
            ("String", "Show<std/prelude::String>"),
        ] {
            let evidence = select_call_evidence(&[TypedConstraint {
                name: "Show".to_owned(),
                arguments: vec![named(name)],
            }])
            .expect("standard Show evidence");
            assert!(matches!(
                evidence.as_slice(),
                [TypedCallEvidence {
                    evidence: TypedInstanceEvidence::Standard { identity: selected },
                    ..
                }] if selected == identity
            ));
        }
    }

    #[test]
    fn selects_standard_equality_evidence_for_primitive_types() {
        for (name, identity) in [
            ("Int", "std/int::Eq"),
            ("Bool", "std/bool::Eq"),
            ("String", "std/string::Eq"),
        ] {
            let evidence = select_equality_evidence("==", named(name), named(name));
            assert!(matches!(
                evidence.as_slice(),
                [TypedCallEvidence {
                    constraint: TypedConstraint { name, arguments },
                    evidence: TypedInstanceEvidence::Standard { identity: selected },
                }] if name == "Eq" && arguments.len() == 1 && selected == identity
            ));
        }
    }

    #[test]
    fn does_not_select_equality_evidence_for_mixed_types() {
        assert!(select_equality_evidence("==", named("Int"), named("String")).is_empty());
    }
}
