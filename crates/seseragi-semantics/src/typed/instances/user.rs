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
        .map(|(index, name)| (name.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let canonical_arguments = arguments
        .iter()
        .map(|argument| canonical_type_ref(argument, resolution, &binders))
        .collect::<Option<Vec<_>>>()?;
    let scoped_evidence = crate::typed::scoped_call_evidence(constraints, resolution);
    let methods = methods
        .iter()
        .map(|method| typed_method(method, resolution, &scoped_evidence))
        .collect::<Option<Vec<_>>>()?;

    Some(TypedInstance {
        identity: canonical_instance_head_identity(&trait_identity, &canonical_arguments),
        trait_identity,
        trait_name: trait_name.clone(),
        type_parameters: type_parameters.clone(),
        arguments: arguments.iter().map(typed_type_from_type_ref).collect(),
        type_identity: None,
        argument_identities: canonical_arguments,
        constraints: constraints
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
    let parameters = typed_parameters_from_surface(&method.parameters);
    let mut method_evidence = scoped_evidence.to_vec();
    method_evidence.extend(crate::typed::scoped_call_evidence_from(
        &method.constraints,
        resolution,
        method_evidence.len(),
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
    match type_ref {
        TypeRef::Named {
            name,
            arguments,
            span,
        } => {
            let constructor = if let Some(index) = binders.get(name.as_str()) {
                format!("${index}")
            } else {
                canonical_reference(resolution, *span, SymbolNamespace::Type)?
            };
            if arguments.is_empty() {
                Some(constructor)
            } else {
                Some(format!(
                    "{constructor}<{}>",
                    arguments
                        .iter()
                        .map(|argument| canonical_type_ref(argument, resolution, binders))
                        .collect::<Option<Vec<_>>>()?
                        .join(",")
                ))
            }
        }
        TypeRef::Hole { .. } => Some("_".to_owned()),
        TypeRef::Tuple { elements, .. } => Some(format!(
            "({})",
            elements
                .iter()
                .map(|element| canonical_type_ref(element, resolution, binders))
                .collect::<Option<Vec<_>>>()?
                .join(",")
        )),
        TypeRef::Function {
            parameter, result, ..
        } => Some(format!(
            "({}->{})",
            canonical_type_ref(parameter, resolution, binders)?,
            canonical_type_ref(result, resolution, binders)?
        )),
        TypeRef::Record { closed, fields, .. } => {
            let mut fields = fields
                .iter()
                .map(|field| {
                    Some(format!(
                        "{}{}:{}",
                        field.name,
                        if field.optional { "?" } else { "" },
                        canonical_type_ref(&field.type_ref, resolution, binders)?
                    ))
                })
                .collect::<Option<Vec<_>>>()?;
            fields.sort();
            Some(format!(
                "{}{{{}}}",
                if *closed { "" } else { "open" },
                fields.join(",")
            ))
        }
    }
}
