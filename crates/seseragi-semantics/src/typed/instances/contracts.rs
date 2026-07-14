use crate::{ResolvedModule, SymbolKind, SymbolNamespace};
use seseragi_syntax::{ByteSpan, SurfaceDecl, SurfaceMethod};
use std::collections::BTreeMap;

mod model;
mod types;

pub(crate) use model::InstanceContractIssue;
use model::{TraitContract, TraitMethodContract};

pub(crate) fn analyze_instance_contracts(resolved: &ResolvedModule) -> Vec<InstanceContractIssue> {
    resolved
        .declarations
        .iter()
        .filter_map(|declaration| {
            let SurfaceDecl::Instance {
                trait_name,
                trait_name_span,
                arguments,
                methods,
                span,
                ..
            } = declaration
            else {
                return None;
            };
            let contract = trait_contract(resolved, *trait_name_span)?;
            Some(validate_instance(
                resolved, trait_name, arguments, methods, *span, contract,
            ))
        })
        .flatten()
        .collect()
}

fn validate_instance(
    resolved: &ResolvedModule,
    trait_name: &str,
    arguments: &[seseragi_syntax::TypeRef],
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
        import_span: imported.origin,
    })
}

fn contract_method_span(method: &TraitMethodContract<'_>, fallback: ByteSpan) -> ByteSpan {
    match method {
        TraitMethodContract::Local(method) => method.span,
        TraitMethodContract::Imported(_) => fallback,
    }
}
