use crate::prelude::{is_standalone_symbol, sum_type_for_symbol};
use crate::{ExternalTypeBinding, ExternalTypeProvider, SymbolNamespace};
use seseragi_syntax::{InterfaceExport, InterfaceType, ModuleInterface};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn callable_scheme_type_bindings(
    provider: &ModuleInterface,
    callable: &InterfaceExport,
) -> Option<Vec<ExternalTypeBinding>> {
    if !matches!(
        callable.declaration_kind.as_deref(),
        Some("function" | "effect-function")
    ) {
        return None;
    }
    let candidates = provider_candidates(provider);
    let type_parameters = callable
        .scheme
        .type_parameters
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut bindings = Vec::new();
    collect_bindings(
        &callable.scheme.type_ref,
        &type_parameters,
        &candidates,
        &mut bindings,
    )?;
    Some(bindings)
}

fn provider_candidates(provider: &ModuleInterface) -> BTreeMap<String, Vec<ExternalTypeBinding>> {
    let mut candidates = BTreeMap::<String, Vec<ExternalTypeBinding>>::new();
    for export in provider
        .exports
        .iter()
        .filter(|export| export.namespace == "type")
    {
        push_candidate(
            &mut candidates,
            ExternalTypeBinding {
                spelling: export.name.clone(),
                canonical: export.symbol.clone(),
                provider: Some(ExternalTypeProvider {
                    module: provider.module.clone(),
                    export: export.name.clone(),
                }),
            },
        );
    }
    for dependency in &provider.dependencies {
        for import in dependency
            .imports
            .iter()
            .filter(|import| import.namespace == "type")
        {
            push_candidate(
                &mut candidates,
                ExternalTypeBinding {
                    spelling: import
                        .local_name
                        .clone()
                        .unwrap_or_else(|| import.name.clone()),
                    canonical: import.symbol.clone(),
                    provider: Some(ExternalTypeProvider {
                        module: dependency.module.clone(),
                        export: import.name.clone(),
                    }),
                },
            );
        }
    }
    candidates
}

fn push_candidate(
    candidates: &mut BTreeMap<String, Vec<ExternalTypeBinding>>,
    candidate: ExternalTypeBinding,
) {
    let same_spelling = candidates.entry(candidate.spelling.clone()).or_default();
    if !same_spelling.contains(&candidate) {
        same_spelling.push(candidate);
    }
}

fn collect_bindings(
    type_ref: &InterfaceType,
    type_parameters: &BTreeSet<String>,
    candidates: &BTreeMap<String, Vec<ExternalTypeBinding>>,
    bindings: &mut Vec<ExternalTypeBinding>,
) -> Option<()> {
    match type_ref {
        InterfaceType::Named { name, arguments } => {
            collect_named(name, type_parameters, candidates, bindings)?;
            for argument in arguments {
                collect_bindings(argument, type_parameters, candidates, bindings)?;
            }
        }
        InterfaceType::ExternalNamed {
            name,
            canonical,
            provider_module,
            provider_export,
            arguments,
        } => {
            let binding = ExternalTypeBinding {
                spelling: name.clone(),
                canonical: canonical.clone(),
                provider: Some(ExternalTypeProvider {
                    module: provider_module.clone(),
                    export: provider_export.clone(),
                }),
            };
            if !bindings.contains(&binding) {
                bindings.push(binding);
            }
            for argument in arguments {
                collect_bindings(argument, type_parameters, candidates, bindings)?;
            }
        }
        InterfaceType::TypeConstructor { name, .. } => {
            collect_named(name, type_parameters, candidates, bindings)?;
        }
        InterfaceType::Apply {
            constructor,
            arguments,
        } => {
            collect_named(constructor, type_parameters, candidates, bindings)?;
            for argument in arguments {
                collect_bindings(argument, type_parameters, candidates, bindings)?;
            }
        }
        InterfaceType::Function { parameter, result } => {
            collect_bindings(parameter, type_parameters, candidates, bindings)?;
            collect_bindings(result, type_parameters, candidates, bindings)?;
        }
        InterfaceType::Record { fields, .. } => {
            for field in fields {
                collect_bindings(&field.type_ref, type_parameters, candidates, bindings)?;
            }
        }
        InterfaceType::Tuple { elements } => {
            for element in elements {
                collect_bindings(element, type_parameters, candidates, bindings)?;
            }
        }
        InterfaceType::Hole => {}
    }
    Some(())
}

fn collect_named(
    name: &str,
    type_parameters: &BTreeSet<String>,
    candidates: &BTreeMap<String, Vec<ExternalTypeBinding>>,
    bindings: &mut Vec<ExternalTypeBinding>,
) -> Option<()> {
    if type_parameters.contains(name) {
        return Some(());
    }
    match candidates.get(name).map(Vec::as_slice) {
        Some([binding]) => {
            if !bindings.contains(binding) {
                bindings.push(binding.clone());
            }
            Some(())
        }
        Some(_) => None,
        None if is_prelude_type(name) => Some(()),
        None => None,
    }
}

fn is_prelude_type(name: &str) -> bool {
    is_standalone_symbol(SymbolNamespace::Type, name)
        || sum_type_for_symbol(SymbolNamespace::Type, name).is_some()
}

#[cfg(test)]
mod tests {
    use super::callable_scheme_type_bindings;
    use seseragi_syntax::{
        ByteSpan, InterfaceDependency, InterfaceExport, InterfaceImport, InterfaceScheme,
        InterfaceType, ModuleInterface, Visibility,
    };

    #[test]
    fn rejects_ambiguous_provider_local_type_spelling() {
        let mut provider = module(vec![type_export("User", "fixture/provider::User")]);
        provider.dependencies.push(InterfaceDependency {
            specifier: "./domain".to_owned(),
            module: "fixture/domain".to_owned(),
            origin: ByteSpan { start: 0, end: 10 },
            imports: vec![InterfaceImport {
                namespace: "type".to_owned(),
                name: "DomainUser".to_owned(),
                symbol: "fixture/domain::DomainUser".to_owned(),
                local_name: Some("User".to_owned()),
            }],
        });
        let callable = function_export("accept", named("User"), named("Unit"));

        assert_eq!(callable_scheme_type_bindings(&provider, &callable), None);
    }

    #[test]
    fn rejects_an_unresolved_non_prelude_scheme_type() {
        let provider = module(Vec::new());
        let callable = function_export("accept", named("Missing"), named("Unit"));

        assert_eq!(callable_scheme_type_bindings(&provider, &callable), None);
    }

    fn module(exports: Vec<InterfaceExport>) -> ModuleInterface {
        ModuleInterface {
            schema: 1,
            module: "fixture/provider".to_owned(),
            source: "provider.ssrg".to_owned(),
            dependencies: Vec::new(),
            exports,
            operators: Vec::new(),
            instances: Vec::new(),
        }
    }

    fn type_export(name: &str, symbol: &str) -> InterfaceExport {
        InterfaceExport {
            symbol: symbol.to_owned(),
            namespace: "type".to_owned(),
            name: name.to_owned(),
            constructor_of: None,
            visibility: Visibility::Public,
            declaration_kind: Some("type".to_owned()),
            declaration: ByteSpan { start: 0, end: 4 },
            scheme: InterfaceScheme {
                type_parameters: Vec::new(),
                constraints: Vec::new(),
                type_ref: InterfaceType::TypeConstructor {
                    name: name.to_owned(),
                    arity: 0,
                },
            },
            representation: None,
        }
    }

    fn function_export(
        name: &str,
        parameter: InterfaceType,
        result: InterfaceType,
    ) -> InterfaceExport {
        InterfaceExport {
            symbol: format!("fixture/provider::{name}"),
            namespace: "value".to_owned(),
            name: name.to_owned(),
            constructor_of: None,
            visibility: Visibility::Public,
            declaration_kind: Some("function".to_owned()),
            declaration: ByteSpan { start: 5, end: 20 },
            scheme: InterfaceScheme {
                type_parameters: Vec::new(),
                constraints: Vec::new(),
                type_ref: InterfaceType::Function {
                    parameter: Box::new(parameter),
                    result: Box::new(result),
                },
            },
            representation: None,
        }
    }

    fn named(name: &str) -> InterfaceType {
        InterfaceType::Named {
            name: name.to_owned(),
            arguments: Vec::new(),
        }
    }
}
