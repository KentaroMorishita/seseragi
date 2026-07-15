use crate::prelude::is_standalone_symbol;
use crate::{ExternalTraitBinding, SymbolNamespace};
use seseragi_syntax::{InterfaceExport, ModuleInterface};
use std::collections::BTreeMap;

pub(crate) fn export_contract_trait_bindings(
    provider: &ModuleInterface,
    export: &InterfaceExport,
) -> Option<Vec<ExternalTraitBinding>> {
    if export.declaration_kind.as_deref() != Some("trait") {
        return None;
    }
    let candidates = provider_trait_candidates(provider);
    collect_constraint_bindings(
        export.scheme.constraints.iter().chain(
            export
                .methods
                .iter()
                .flat_map(|method| &method.scheme.constraints),
        ),
        &candidates,
    )
}

pub(crate) fn export_scheme_trait_bindings(
    provider: &ModuleInterface,
    export: &InterfaceExport,
) -> Option<Vec<ExternalTraitBinding>> {
    if !matches!(
        export.declaration_kind.as_deref(),
        Some("function" | "effect-function")
    ) {
        return None;
    }
    let candidates = provider_trait_candidates(provider);
    let bindings = collect_constraint_bindings(&export.scheme.constraints, &candidates)?;
    (!bindings.is_empty()).then_some(bindings)
}

fn collect_constraint_bindings<'a>(
    constraints: impl IntoIterator<Item = &'a seseragi_syntax::InterfaceConstraint>,
    candidates: &BTreeMap<String, Vec<ExternalTraitBinding>>,
) -> Option<Vec<ExternalTraitBinding>> {
    let mut bindings = Vec::new();
    for constraint in constraints {
        match candidates.get(&constraint.name) {
            Some(matches) => {
                let [binding] = matches.as_slice() else {
                    return None;
                };
                if !bindings.contains(binding) {
                    bindings.push(binding.clone());
                }
            }
            None if is_standalone_symbol(SymbolNamespace::Trait, &constraint.name) => {}
            None => return None,
        }
    }
    Some(bindings)
}

fn provider_trait_candidates(
    provider: &ModuleInterface,
) -> BTreeMap<String, Vec<ExternalTraitBinding>> {
    let mut candidates = BTreeMap::<String, Vec<ExternalTraitBinding>>::new();
    for export in provider
        .exports
        .iter()
        .filter(|export| export.namespace == "trait")
    {
        push_candidate(
            &mut candidates,
            ExternalTraitBinding {
                spelling: export.name.clone(),
                canonical: export.symbol.clone(),
            },
        );
    }
    for dependency in &provider.dependencies {
        for import in dependency
            .imports
            .iter()
            .filter(|import| import.namespace == "trait")
        {
            push_candidate(
                &mut candidates,
                ExternalTraitBinding {
                    spelling: import
                        .local_name
                        .clone()
                        .unwrap_or_else(|| import.name.clone()),
                    canonical: import.symbol.clone(),
                },
            );
        }
    }
    candidates
}

fn push_candidate(
    candidates: &mut BTreeMap<String, Vec<ExternalTraitBinding>>,
    candidate: ExternalTraitBinding,
) {
    let same_spelling = candidates.entry(candidate.spelling.clone()).or_default();
    if !same_spelling.contains(&candidate) {
        same_spelling.push(candidate);
    }
}

#[cfg(test)]
mod tests {
    use super::{export_contract_trait_bindings, export_scheme_trait_bindings};
    use seseragi_syntax::{
        ByteSpan, InterfaceConstraint, InterfaceDependency, InterfaceExport, InterfaceImport,
        InterfaceMethod, InterfaceScheme, InterfaceType, ModuleInterface, Visibility,
    };

    #[test]
    fn collects_provider_traits_from_trait_method_constraints() {
        let labeled = trait_export("Labeled", Vec::new());
        let render = trait_export(
            "Render",
            vec![InterfaceConstraint {
                name: "Labeled".to_owned(),
                arguments: vec![named("A")],
            }],
        );
        let provider = module(vec![labeled, render.clone()]);

        let bindings = export_contract_trait_bindings(&provider, &render).unwrap();

        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].spelling, "Labeled");
        assert_eq!(bindings[0].canonical, "fixture/provider::trait(Labeled)");
    }

    #[test]
    fn collects_provider_dependency_traits_from_method_constraints() {
        let render = trait_export(
            "Render",
            vec![InterfaceConstraint {
                name: "Named".to_owned(),
                arguments: vec![named("A")],
            }],
        );
        let mut provider = module(vec![render.clone()]);
        provider.dependencies.push(InterfaceDependency {
            specifier: "./names".to_owned(),
            module: "fixture/names".to_owned(),
            origin: ByteSpan { start: 0, end: 12 },
            imports: vec![InterfaceImport {
                namespace: "trait".to_owned(),
                name: "Labeled".to_owned(),
                symbol: "fixture/names::trait(Labeled)".to_owned(),
                local_name: Some("Named".to_owned()),
            }],
        });

        let bindings = export_contract_trait_bindings(&provider, &render).unwrap();

        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].spelling, "Named");
        assert_eq!(bindings[0].canonical, "fixture/names::trait(Labeled)");
    }

    #[test]
    fn collects_provider_traits_from_callable_scheme_constraints() {
        let ready = trait_export("Ready", Vec::new());
        let mut describe = function_export("describe");
        describe.scheme.constraints.push(InterfaceConstraint {
            name: "Ready".to_owned(),
            arguments: vec![named("T")],
        });
        let provider = module(vec![ready, describe.clone()]);

        let bindings = export_scheme_trait_bindings(&provider, &describe).unwrap();

        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].spelling, "Ready");
        assert_eq!(bindings[0].canonical, "fixture/provider::trait(Ready)");
    }

    #[test]
    fn prefers_a_provider_trait_that_shadows_a_standalone_spelling() {
        let show = trait_export("Show", Vec::new());
        let mut describe = function_export("describe");
        describe.scheme.constraints.push(InterfaceConstraint {
            name: "Show".to_owned(),
            arguments: vec![named("T")],
        });
        let provider = module(vec![show, describe.clone()]);

        let bindings = export_scheme_trait_bindings(&provider, &describe).unwrap();

        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].canonical, "fixture/provider::trait(Show)");
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

    fn trait_export(name: &str, constraints: Vec<InterfaceConstraint>) -> InterfaceExport {
        InterfaceExport {
            symbol: format!("fixture/provider::trait({name})"),
            namespace: "trait".to_owned(),
            name: name.to_owned(),
            constructor_of: None,
            visibility: Visibility::Public,
            declaration_kind: Some("trait".to_owned()),
            declaration: ByteSpan { start: 0, end: 8 },
            scheme: InterfaceScheme {
                type_parameters: vec!["A".to_owned()],
                constraints: Vec::new(),
                type_ref: InterfaceType::TypeConstructor {
                    name: name.to_owned(),
                    arity: 1,
                },
            },
            methods: vec![InterfaceMethod {
                name: "method".to_owned(),
                scheme: InterfaceScheme {
                    type_parameters: Vec::new(),
                    constraints,
                    type_ref: InterfaceType::Function {
                        parameter: Box::new(named("A")),
                        result: Box::new(named("String")),
                    },
                },
                origin: ByteSpan { start: 2, end: 7 },
            }],
            representation: None,
        }
    }

    fn function_export(name: &str) -> InterfaceExport {
        InterfaceExport {
            symbol: format!("fixture/provider::{name}"),
            namespace: "value".to_owned(),
            name: name.to_owned(),
            constructor_of: None,
            visibility: Visibility::Public,
            declaration_kind: Some("function".to_owned()),
            declaration: ByteSpan { start: 0, end: 8 },
            scheme: InterfaceScheme {
                type_parameters: vec!["T".to_owned()],
                constraints: Vec::new(),
                type_ref: InterfaceType::Function {
                    parameter: Box::new(named("T")),
                    result: Box::new(named("String")),
                },
            },
            methods: Vec::new(),
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
