use super::*;
use crate::{TypedCallEvidence, TypedInstanceEvidence, TypedInstanceImplementation};
use seseragi_syntax::{standard_operator, InterfaceOperatorDeprecation};

pub(in crate::query) struct OperatorUse<'a> {
    notice: &'a InterfaceOperatorDeprecation,
    range: ByteSpan,
}

pub(in crate::query) fn catalog(
    resolved: &ResolvedModule,
    typed: &TypedModule,
) -> Vec<InterfaceOperatorDeprecation> {
    let imported = resolved
        .operator_deprecations
        .iter()
        .filter(|notice| {
            resolved.dependency_instances.iter().any(|instance| {
                instance.identity == notice.instance
                    && instance.provider_module == notice.provider_module
                    && instance.trait_identity == notice.trait_identity
            })
        })
        .cloned();
    crate::deprecated_operators::merge(imported.chain(crate::deprecated_operators::local(
        crate::deprecated_operators::sources(resolved),
        typed,
    )))
}

pub(in crate::query) fn uses<'a>(
    resolved: &ResolvedModule,
    typed: &TypedModule,
    notices: &'a [InterfaceOperatorDeprecation],
) -> Vec<OperatorUse<'a>> {
    let sources = super::source_calls::callee_ranges(resolved);
    let mut result = BTreeMap::new();
    let mut visit = |expression: &TypedExpr| {
        let (contract, origin, evidence) = match expression {
            TypedExpr::Binary {
                operator,
                origin,
                evidence,
                ..
            } => (operator_contract(operator), *origin, evidence.as_slice()),
            TypedExpr::Call {
                callee,
                origin,
                evidence,
                trait_dispatch,
                ..
            } => {
                let contract = trait_dispatch
                    .as_ref()
                    .map(|dispatch| (dispatch.trait_identity.clone(), dispatch.method.clone()))
                    .or_else(|| operator_contract(callee));
                (contract, *origin, evidence.as_slice())
            }
            TypedExpr::Variable {
                name,
                origin,
                evidence,
                ..
            } => (operator_contract(name), *origin, evidence.as_slice()),
            _ => return,
        };
        let Some((trait_identity, method)) = contract else {
            return;
        };
        let Some(range) = sources.get(&(origin.start, origin.end)).copied() else {
            return;
        };
        for notice in notices
            .iter()
            .filter(|notice| notice.trait_identity == trait_identity && notice.method == method)
        {
            if notice.provider_module == resolved.module
                && notice.declaration.start <= range.start
                && range.end <= notice.declaration.end
            {
                continue;
            }
            if evidence
                .iter()
                .any(|witness| selected(witness, notice, &resolved.module))
            {
                result.insert(
                    (range.start, range.end, notice.symbol()),
                    OperatorUse { notice, range },
                );
            }
        }
    };
    for declaration in &typed.declarations {
        match declaration {
            TypedDecl::Fn { body, .. } | TypedDecl::EffectFn { body, .. } => {
                super::super::walk_expression(body, &mut visit)
            }
            TypedDecl::Let { value, .. } => super::super::walk_expression(value, &mut visit),
            _ => {}
        }
    }
    for instance in &typed.instances {
        if let TypedInstanceImplementation::UserDefined { methods } = &instance.implementation {
            for method in methods {
                super::super::walk_expression(&method.body, &mut visit);
            }
        }
    }
    result.into_values().collect()
}

fn operator_contract(name: &str) -> Option<(String, String)> {
    let (trait_name, method_name) = if let Some(operator) = standard_operator(name) {
        (operator.trait_name, operator.method_name)
    } else {
        let method = crate::prelude::trait_method_by_canonical(name)?;
        (method.trait_name, method.name)
    };
    Some((
        crate::prelude::trait_by_name(trait_name)?
            .canonical
            .to_owned(),
        method_name.to_owned(),
    ))
}

fn selected(
    evidence: &TypedCallEvidence,
    notice: &InterfaceOperatorDeprecation,
    module: &str,
) -> bool {
    if evidence.trait_identity.as_deref() != Some(notice.trait_identity.as_str()) {
        return false;
    }
    match &evidence.evidence {
        TypedInstanceEvidence::Local { identity, .. } => {
            identity == &notice.instance && module == notice.provider_module
        }
        TypedInstanceEvidence::Imported {
            identity,
            provider_module,
            ..
        } => identity == &notice.instance && provider_module == &notice.provider_module,
        TypedInstanceEvidence::Standard { .. } | TypedInstanceEvidence::Parameter { .. } => false,
    }
}

pub(in crate::query) fn project(
    resolved: &ResolvedModule,
    notices: &[InterfaceOperatorDeprecation],
    uses: &[OperatorUse<'_>],
    symbols: &mut Vec<AnalysisSymbol>,
    occurrences: &mut Vec<AnalysisSymbolOccurrence>,
) {
    for notice in notices {
        let local = notice.provider_module == resolved.module;
        let id = symbols.len() as u32;
        symbols.push(AnalysisSymbol {
            id,
            identity: notice.symbol(),
            name: notice.spelling.clone(),
            module: notice.provider_module.clone(),
            namespace: "operator".to_owned(),
            kind: "operator-implementation".to_owned(),
            definition: if local {
                notice.name_span
            } else {
                ByteSpan { start: 0, end: 0 }
            },
            type_name: None,
            multiline_type_name: None,
            type_document: None,
            callable: None,
            description: None,
            deprecation: Some(notice.deprecation.clone()),
            member_owner: Some(notice.instance.clone()),
            scope: ScopeId(0),
        });
        let ranges = local.then_some(notice.name_span).into_iter().chain(
            uses.iter()
                .filter(|usage| std::ptr::eq(usage.notice, notice))
                .map(|usage| usage.range),
        );
        for range in ranges {
            // Keep the chosen implementation local to this proven operation;
            // the global Prelude method remains unmodified for other values.
            occurrences.retain(|occurrence| occurrence.range != range);
            occurrences.push(AnalysisSymbolOccurrence { range, symbol: id });
        }
    }
}

pub(in crate::query) fn warnings(uses: &[OperatorUse<'_>]) -> Vec<Diagnostic> {
    uses.iter()
        .map(|usage| {
            let notice = &usage.notice.deprecation;
            let since = notice
                .since
                .as_ref()
                .map(|version| format!(" since {version}"))
                .unwrap_or_default();
            Diagnostic {
                id: String::new(),
                code: "SES-L0201".to_owned(),
                severity: DiagnosticSeverity::Warning,
                message_key: "lint.deprecated-symbol".to_owned(),
                primary: ByteRange {
                    start: usage.range.start,
                    end: usage.range.end,
                },
                related: vec![RelatedDiagnostic {
                    source: Some(notice.source.clone()),
                    message: format!(
                        "operator `{}` is deprecated{since}: {}",
                        usage.notice.spelling, notice.message
                    ),
                    primary: ByteRange {
                        start: notice.origin.start,
                        end: notice.origin.end,
                    },
                }],
                fixes: Vec::new(),
                type_difference: None,
            }
        })
        .collect()
}
