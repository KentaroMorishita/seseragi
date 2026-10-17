use crate::{
    ResolvedModule, SymbolNamespace, TypedConstraint, TypedInstance, TypedInstanceImplementation,
    TypedInstanceMethod, TypedScheme,
};
use seseragi_syntax::{SurfaceDecl, SurfaceMethod, TypeRef};
use std::collections::BTreeMap;

use super::{canonical_instance_head_identity, TypedResolution};
use crate::typed::functions::typed_parameters_from_surface;
use crate::typed::surface_expr::{analyze_resolved_expression, PureExpressionContext};
use crate::typed::type_ref::typed_type_from_type_ref;

pub(super) fn analyze_user_defined_instances(
    resolved: &ResolvedModule,
    resolution: &TypedResolution<'_>,
) -> Vec<TypedInstance> {
    resolved
        .declarations
        .iter()
        .filter_map(|declaration| typed_instance(declaration, resolution))
        .collect()
}

fn typed_instance(
    declaration: &SurfaceDecl,
    resolution: &TypedResolution<'_>,
) -> Option<TypedInstance> {
    let SurfaceDecl::Instance {
        type_parameters,
        trait_name,
        trait_name_span,
        arguments,
        constraints,
        methods,
        span,
    } = declaration
    else {
        return None;
    };
    let trait_identity = canonical_reference(resolution, *trait_name_span, SymbolNamespace::Trait)?;
    let binders = type_parameters
        .iter()
        .enumerate()
        .map(|(index, name)| (name.name.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let canonical_arguments = arguments
        .iter()
        .map(|argument| canonical_type_ref(argument, resolution, &binders))
        .collect::<Option<Vec<_>>>()?;
    let type_identity = (trait_identity == "std/prelude::Show")
        .then(|| canonical_arguments.as_slice())
        .and_then(|arguments| {
            let [argument] = arguments else {
                return None;
            };
            Some(argument.clone())
        });
    let typed_arguments = arguments
        .iter()
        .map(|argument| resolution.semantic_value_from_type_ref(argument).type_ref)
        .collect::<Vec<_>>();
    let supertraits =
        crate::typed::direct_supertrait_constraints(*trait_name_span, &typed_arguments, resolution);
    let constraint_identities = supertraits
        .iter()
        .map(|supertrait| Some(supertrait.trait_identity.clone()))
        .chain(constraints.iter().map(|constraint| {
            canonical_reference(resolution, constraint.name_span, SymbolNamespace::Trait)
        }))
        .collect();
    let mut scoped_evidence =
        crate::typed::scoped_resolved_call_evidence(&supertraits, resolution, 0);
    scoped_evidence.extend(crate::typed::scoped_call_evidence_from(
        constraints,
        resolution,
        supertraits.len(),
    ));
    let methods = methods
        .iter()
        .map(|method| typed_method(method, resolution, &scoped_evidence))
        .collect::<Option<Vec<_>>>()?;

    Some(TypedInstance {
        identity: canonical_instance_head_identity(&trait_identity, &canonical_arguments),
        trait_identity,
        trait_name: trait_name.clone(),
        type_parameters: type_parameters.clone(),
        arguments: typed_arguments,
        type_identity,
        argument_identities: canonical_arguments,
        constraints: supertraits
            .iter()
            .map(|supertrait| supertrait.constraint.clone())
            .chain(constraints.iter().map(|constraint| {
                TypedConstraint {
                    name: constraint.name.clone(),
                    arguments: constraint
                        .arguments
                        .iter()
                        .map(|argument| resolution.semantic_value_from_type_ref(argument).type_ref)
                        .collect(),
                }
            }))
            .collect(),
        constraint_identities,
        supertrait_count: supertraits.len(),
        origin: *span,
        implementation: TypedInstanceImplementation::UserDefined { methods },
    })
}

fn typed_method(
    method: &SurfaceMethod,
    resolution: &TypedResolution<'_>,
    scoped_evidence: &[crate::typed::ScopedCallEvidence],
) -> Option<TypedInstanceMethod> {
    let body = method.body.as_ref()?;
    let parameters = typed_parameters_from_surface(&method.parameters, resolution);
    let mut method_evidence = scoped_evidence.to_vec();
    let next_evidence_index = scoped_evidence
        .iter()
        .map(|evidence| evidence.index())
        .max()
        .map_or(0, |index| index + 1);
    method_evidence.extend(crate::typed::scoped_call_evidence_from(
        &method.constraints,
        resolution,
        next_evidence_index,
    ));
    let context = PureExpressionContext::new(&parameters, resolution)
        .with_evidence_parameters(method_evidence)
        .with_expected(Some(
            resolution.semantic_value_from_type_ref(&method.return_type),
        ));
    let body = analyze_resolved_expression(body, &context).value;

    Some(TypedInstanceMethod {
        name: method.name.clone(),
        scheme: TypedScheme {
            type_parameters: method.type_parameters.clone(),
            constraints: method
                .constraints
                .iter()
                .map(|constraint| TypedConstraint {
                    name: constraint.name.clone(),
                    arguments: constraint
                        .arguments
                        .iter()
                        .map(typed_type_from_type_ref)
                        .collect(),
                })
                .collect(),
            constraint_identities: method
                .constraints
                .iter()
                .map(|constraint| {
                    canonical_reference(resolution, constraint.name_span, SymbolNamespace::Trait)
                })
                .collect(),
            type_ref: typed_type_from_type_ref(&method.return_type),
        },
        parameters,
        body,
        origin: method.span,
    })
}

fn canonical_reference(
    resolution: &TypedResolution<'_>,
    origin: seseragi_syntax::ByteSpan,
    namespace: SymbolNamespace,
) -> Option<String> {
    let target = resolution.target(origin, namespace)?;
    let symbol = resolution.symbol(target)?;
    symbol
        .canonical
        .clone()
        .or_else(|| Some(symbol.spelling.clone()))
}

pub(crate) fn canonical_type_ref(
    type_ref: &TypeRef,
    resolution: &TypedResolution<'_>,
    binders: &BTreeMap<&str, usize>,
) -> Option<String> {
    super::contracts::canonical_type_identity(resolution.resolved(), type_ref, binders)
}
