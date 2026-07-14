use crate::{ResolvedModule, SymbolKind, SymbolNamespace};
use seseragi_syntax::{ByteSpan, SurfaceDecl, SurfaceMethod};
use std::collections::BTreeMap;

mod types;

struct LocalTraitContract<'a> {
    parameters: &'a [String],
    methods: &'a [SurfaceMethod],
    span: ByteSpan,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum InstanceContractIssue {
    ArityMismatch {
        trait_name: String,
        expected: usize,
        actual: usize,
        primary: ByteSpan,
        declaration: ByteSpan,
    },
    MissingMethod {
        method: String,
        primary: ByteSpan,
        contract: ByteSpan,
    },
    UnexpectedMethod {
        method: String,
        primary: ByteSpan,
        contract: ByteSpan,
    },
    DuplicateMethod {
        method: String,
        primary: ByteSpan,
        declaration: ByteSpan,
    },
    SignatureMismatch {
        method: String,
        primary: ByteSpan,
        contract: ByteSpan,
    },
    MissingBody {
        method: String,
        primary: ByteSpan,
        contract: ByteSpan,
    },
}

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
            let contract = local_trait_contract(resolved, *trait_name_span)?;
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
    contract: LocalTraitContract<'_>,
) -> Vec<InstanceContractIssue> {
    let mut issues = Vec::new();
    if arguments.len() != contract.parameters.len() {
        issues.push(InstanceContractIssue::ArityMismatch {
            trait_name: trait_name.to_owned(),
            expected: contract.parameters.len(),
            actual: arguments.len(),
            primary: instance_span,
            declaration: contract.span,
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

    let expected = contract
        .methods
        .iter()
        .map(|method| (method.name.as_str(), method))
        .collect::<BTreeMap<_, _>>();
    for expected_method in contract.methods {
        let Some(implementation) = actual
            .get(expected_method.name.as_str())
            .and_then(|methods| methods.first())
        else {
            issues.push(InstanceContractIssue::MissingMethod {
                method: expected_method.name.clone(),
                primary: instance_span,
                contract: expected_method.span,
            });
            continue;
        };
        if implementation.body.is_none() {
            issues.push(InstanceContractIssue::MissingBody {
                method: expected_method.name.clone(),
                primary: implementation.span,
                contract: expected_method.span,
            });
        }
        if matches!(
            types::method_contract_matches(
                resolved,
                contract.span,
                contract.parameters,
                arguments,
                expected_method,
                implementation,
            ),
            Some(false)
        ) {
            issues.push(InstanceContractIssue::SignatureMismatch {
                method: expected_method.name.clone(),
                primary: implementation.span,
                contract: expected_method.span,
            });
        }
    }
    for implementation in methods {
        if !expected.contains_key(implementation.name.as_str()) {
            issues.push(InstanceContractIssue::UnexpectedMethod {
                method: implementation.name.clone(),
                primary: implementation.name_span,
                contract: contract.span,
            });
        }
    }
    issues
}

fn local_trait_contract<'a>(
    resolved: &'a ResolvedModule,
    trait_name_span: ByteSpan,
) -> Option<LocalTraitContract<'a>> {
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
    resolved.declarations.iter().find_map(|declaration| {
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
        (*name_span == symbol.origin).then_some(LocalTraitContract {
            parameters: type_parameters,
            methods,
            span: *span,
        })
    })
}
