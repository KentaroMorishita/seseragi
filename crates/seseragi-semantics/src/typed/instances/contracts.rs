use crate::{ResolvedModule, SymbolKind, SymbolNamespace, TypedConstraint, TypedType};
use seseragi_syntax::{
    ByteSpan, InterfaceType, SurfaceDecl, SurfaceMethod, TypeParameter, TypeRef,
};
use std::collections::BTreeMap;

mod model;
mod types;

pub(crate) use model::InstanceContractIssue;
use model::{TraitContract, TraitMethodContract};

pub(crate) fn analyze_instance_contracts(
    resolved: &ResolvedModule,
    resolution: &crate::typed::TypedResolution<'_>,
) -> Vec<InstanceContractIssue> {
    resolved
        .declarations
        .iter()
        .filter_map(|declaration| {
            let SurfaceDecl::Instance {
                type_parameters,
                trait_name,
                trait_name_span,
                arguments,
                constraints,
                methods,
                span,
                ..
            } = declaration
            else {
                return None;
            };
            let contract = trait_contract(resolved, *trait_name_span)?;
            Some(validate_instance(
                resolved,
                resolution,
                type_parameters,
                trait_name,
                *trait_name_span,
                arguments,
                constraints,
                methods,
                *span,
                contract,
            ))
        })
        .flatten()
        .collect()
}

fn validate_instance(
    resolved: &ResolvedModule,
    resolution: &crate::typed::TypedResolution<'_>,
    instance_parameters: &[TypeParameter],
    trait_name: &str,
    trait_name_span: ByteSpan,
    arguments: &[seseragi_syntax::TypeRef],
    constraints: &[seseragi_syntax::SurfaceConstraint],
    methods: &[SurfaceMethod],
    instance_span: ByteSpan,
    contract: TraitContract<'_>,
) -> Vec<InstanceContractIssue> {
    let mut issues = Vec::new();
    let (parameters, contract_span) = match &contract {
        TraitContract::Local {
            parameters, span, ..
        } => (*parameters, *span),
        TraitContract::Imported {
            parameters,
            import_span,
            ..
        } => (*parameters, *import_span),
    };
    if arguments.len() != parameters.len() {
        issues.push(InstanceContractIssue::ArityMismatch {
            trait_name: trait_name.to_owned(),
            expected: parameters.len(),
            actual: arguments.len(),
            primary: instance_span,
            declaration: contract_span,
        });
        return issues;
    }
    for (parameter, argument) in parameters.iter().zip(arguments) {
        let Some(actual) = type_ref_arity(resolved, instance_parameters, argument) else {
            continue;
        };
        if actual != parameter.arity {
            issues.push(InstanceContractIssue::KindMismatch {
                parameter: parameter.name.clone(),
                expected: parameter.arity,
                actual,
                primary: type_ref_span(argument),
                declaration: contract_span,
            });
        }
    }

    let typed_arguments = arguments
        .iter()
        .map(crate::typed::typed_type_from_type_ref)
        .collect::<Vec<_>>();
    let scoped = crate::typed::scoped_call_evidence(constraints, resolution);
    for required in
        crate::typed::direct_supertrait_constraints(trait_name_span, &typed_arguments, resolution)
    {
        let selected = crate::typed::select_function_call_evidence(
            std::slice::from_ref(&required.constraint),
            &[Some(required.trait_identity)],
            resolution,
            &scoped,
        );
        if selected.is_err() {
            issues.push(InstanceContractIssue::MissingSupertrait {
                supertrait: render_constraint(&required.constraint),
                primary: instance_span,
                declaration: contract_span,
            });
        }
    }

    let mut actual = BTreeMap::<&str, Vec<&SurfaceMethod>>::new();
    for method in methods {
        actual.entry(&method.name).or_default().push(method);
    }
    for duplicates in actual.values().filter(|methods| methods.len() > 1) {
        for method in duplicates.iter().skip(1) {
            issues.push(InstanceContractIssue::DuplicateMethod {
                method: method.name.clone(),
                primary: method.name_span,
                declaration: duplicates[0].name_span,
            });
        }
    }

    let expected_methods = match &contract {
        TraitContract::Local { methods, .. } => methods
            .iter()
            .map(TraitMethodContract::Local)
            .collect::<Vec<_>>(),
        TraitContract::Imported { methods, .. } => methods
            .iter()
            .map(TraitMethodContract::Imported)
            .collect::<Vec<_>>(),
    };
    let expected = expected_methods
        .iter()
        .map(|method| (method.name(), method))
        .collect::<BTreeMap<_, _>>();
    for expected_method in &expected_methods {
        let Some(implementation) = actual
            .get(expected_method.name())
            .and_then(|methods| methods.first())
        else {
            issues.push(InstanceContractIssue::MissingMethod {
                method: expected_method.name().to_owned(),
                primary: instance_span,
                contract: contract_method_span(expected_method, contract_span),
            });
            continue;
        };
        if implementation.body.is_none() {
            issues.push(InstanceContractIssue::MissingBody {
                method: expected_method.name().to_owned(),
                primary: implementation.span,
                contract: contract_method_span(expected_method, contract_span),
            });
        }
        let matches = match (&contract, expected_method) {
            (
                TraitContract::Local {
                    parameters, span, ..
                },
                TraitMethodContract::Local(expected),
            ) => types::method_contract_matches(
                resolved,
                *span,
                parameters,
                arguments,
                expected,
                implementation,
            ),
            (
                TraitContract::Imported {
                    name,
                    canonical,
                    parameters,
                    bindings,
                    trait_bindings,
                    ..
                },
                TraitMethodContract::Imported(expected),
            ) => types::imported_method_contract_matches(
                resolved,
                arguments,
                expected,
                implementation,
                types::ImportedMethodContext {
                    trait_parameters: parameters,
                    bindings,
                    trait_bindings,
                    trait_name: name,
                    trait_canonical: canonical,
                },
            ),
            _ => None,
        };
        if matches == Some(false) {
            issues.push(InstanceContractIssue::SignatureMismatch {
                method: expected_method.name().to_owned(),
                primary: implementation.span,
                contract: contract_method_span(expected_method, contract_span),
            });
        }
    }
    for implementation in methods {
        if !expected.contains_key(implementation.name.as_str()) {
            issues.push(InstanceContractIssue::UnexpectedMethod {
                method: implementation.name.clone(),
                primary: implementation.name_span,
                contract: contract_span,
            });
        }
    }
    issues
}

fn render_constraint(constraint: &TypedConstraint) -> String {
    if constraint.arguments.is_empty() {
        return constraint.name.clone();
    }
    format!(
        "{}<{}>",
        constraint.name,
        constraint
            .arguments
            .iter()
            .map(render_typed_type)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn render_typed_type(type_ref: &TypedType) -> String {
    match type_ref {
        TypedType::Named { name, arguments }
        | TypedType::ExternalNamed {
            name, arguments, ..
        } => {
            if arguments.is_empty() {
                name.clone()
            } else {
                format!(
                    "{}<{}>",
                    name,
                    arguments
                        .iter()
                        .map(render_typed_type)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        TypedType::Tuple { elements } => format!(
            "({})",
            elements
                .iter()
                .map(render_typed_type)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        TypedType::Function { parameter, result } => {
            format!(
                "({} -> {})",
                render_typed_type(parameter),
                render_typed_type(result)
            )
        }
        TypedType::Record { .. } => "record".to_owned(),
        TypedType::Hole => "_".to_owned(),
    }
}

fn type_ref_arity(
    resolved: &ResolvedModule,
    instance_parameters: &[TypeParameter],
    type_ref: &TypeRef,
) -> Option<u32> {
    match type_ref {
        TypeRef::Named {
            name,
            arguments,
            span,
        } => {
            let declared = named_type_arity(resolved, instance_parameters, name, *span)?;
            let consumed = arguments
                .iter()
                .filter(|argument| !matches!(argument, TypeRef::Hole { .. }))
                .count() as u32;
            Some(declared.saturating_sub(consumed))
        }
        TypeRef::Hole { .. } => None,
        TypeRef::Record { .. } | TypeRef::Tuple { .. } | TypeRef::Function { .. } => Some(0),
    }
}

fn named_type_arity(
    resolved: &ResolvedModule,
    instance_parameters: &[TypeParameter],
    name: &str,
    origin: ByteSpan,
) -> Option<u32> {
    let target = resolved
        .references
        .iter()
        .find(|reference| {
            reference.namespace == SymbolNamespace::Type && reference.origin == origin
        })?
        .target?;
    let symbol = resolved.symbols.iter().find(|symbol| symbol.id == target)?;
    if symbol.kind == SymbolKind::TypeParameter {
        return instance_parameters
            .iter()
            .find(|parameter| parameter.name == name)
            .map(|parameter| parameter.arity);
    }
    if symbol
        .canonical
        .as_deref()
        .is_some_and(|canonical| canonical.starts_with("std/prelude::"))
    {
        return crate::prelude::type_constructor_arity(name);
    }
    if let Some(arity) = resolved.declarations.iter().find_map(|declaration| {
        let (name_span, parameters) = match declaration {
            SurfaceDecl::Newtype {
                name_span,
                type_parameters,
                ..
            }
            | SurfaceDecl::Alias {
                name_span,
                type_parameters,
                ..
            }
            | SurfaceDecl::Type {
                name_span,
                type_parameters,
                ..
            }
            | SurfaceDecl::Struct {
                name_span,
                type_parameters,
                ..
            } => (name_span, type_parameters),
            _ => return None,
        };
        (*name_span == symbol.origin).then_some(parameters.len() as u32)
    }) {
        return Some(arity);
    }
    resolved
        .imports
        .iter()
        .find(|import| import.symbol == target)
        .and_then(|import| match import.export.representation.as_ref() {
            Some(InterfaceType::TypeConstructor { arity, .. }) => Some(*arity),
            _ if import.export.namespace == "type" => {
                Some(import.export.scheme.type_parameters.len() as u32)
            }
            _ => None,
        })
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

fn trait_contract<'a>(
    resolved: &'a ResolvedModule,
    trait_name_span: ByteSpan,
) -> Option<TraitContract<'a>> {
    let target = resolved
        .references
        .iter()
        .find(|reference| {
            reference.namespace == SymbolNamespace::Trait && reference.origin == trait_name_span
        })?
        .target?;
    let symbol = resolved
        .symbols
        .iter()
        .find(|symbol| symbol.id == target && symbol.kind == SymbolKind::Trait)?;
    if let Some(contract) = resolved.declarations.iter().find_map(|declaration| {
        let SurfaceDecl::Trait {
            name_span,
            type_parameters,
            methods,
            span,
            ..
        } = declaration
        else {
            return None;
        };
        (*name_span == symbol.origin).then_some(TraitContract::Local {
            parameters: type_parameters,
            methods,
            span: *span,
        })
    }) {
        return Some(contract);
    }
    let imported = resolved.imports.iter().find(|import| {
        import.symbol == target
            && import.export.namespace == "trait"
            && import.export.declaration_kind.as_deref() == Some("trait")
    })?;
    Some(TraitContract::Imported {
        name: &imported.export.name,
        canonical: &imported.export.symbol,
        parameters: &imported.export.scheme.type_parameters,
        methods: &imported.export.methods,
        bindings: imported.scheme_type_bindings.as_deref().unwrap_or(&[]),
        trait_bindings: imported.contract_trait_bindings.as_deref().unwrap_or(&[]),
        import_span: imported.origin,
    })
}

fn contract_method_span(method: &TraitMethodContract<'_>, fallback: ByteSpan) -> ByteSpan {
    match method {
        TraitMethodContract::Local(method) => method.span,
        TraitMethodContract::Imported(_) => fallback,
    }
}
