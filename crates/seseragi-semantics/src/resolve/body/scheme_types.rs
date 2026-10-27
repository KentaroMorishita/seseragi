use crate::prelude::{is_standalone_symbol, sum_type_for_symbol};
use crate::{ExternalTypeBinding, ExternalTypeProvider, SymbolNamespace};
use seseragi_syntax::{InterfaceExport, InterfaceType, ModuleInterface};
use std::collections::{BTreeMap, BTreeSet};

mod trait_bindings;

pub(super) use trait_bindings::{export_contract_trait_bindings, export_scheme_trait_bindings};

fn has_callable_scheme(export: &InterfaceExport) -> bool {
    export.namespace == "operator"
        || (export.namespace == "value"
            && matches!(export.scheme.type_ref, InterfaceType::Function { .. })
            && !export.scheme.type_parameters.is_empty())
        || matches!(
            export.declaration_kind.as_deref(),
            Some("constructor" | "function" | "effect-function" | "inherent-method")
        )
}

pub(super) fn export_scheme_type_bindings(
    provider: &ModuleInterface,
    export: &InterfaceExport,
) -> Option<Vec<ExternalTypeBinding>> {
    let is_alias = export.declaration_kind.as_deref() == Some("alias");
    if !has_callable_scheme(export)
        && export.declaration_kind.as_deref() != Some("trait")
        && !is_alias
    {
        return None;
    }
    let candidates = provider_candidates(provider, export);
    let type_parameters = export
        .scheme
        .type_parameters
        .iter()
        .map(|parameter| parameter.name.clone())
        .collect::<BTreeSet<_>>();
    let mut bindings = Vec::new();
    if is_alias {
        collect_bindings(
            export.representation.as_ref()?,
            &type_parameters,
            &candidates,
            &mut bindings,
        )?;
        return Some(bindings);
    }
    if export.declaration_kind.as_deref() != Some("trait") {
        collect_bindings(
            &export.scheme.type_ref,
            &type_parameters,
            &candidates,
            &mut bindings,
        )?;
    }
    for constraint in &export.scheme.constraints {
        for argument in &constraint.arguments {
            collect_bindings(argument, &type_parameters, &candidates, &mut bindings)?;
        }
    }
    for method in &export.methods {
        let method_parameters = type_parameters
            .iter()
            .cloned()
            .chain(
                method
                    .scheme
                    .type_parameters
                    .iter()
                    .map(|parameter| parameter.name.clone()),
            )
            .collect::<BTreeSet<_>>();
        collect_bindings(
            &method.scheme.type_ref,
            &method_parameters,
            &candidates,
            &mut bindings,
        )?;
        for constraint in &method.scheme.constraints {
            for argument in &constraint.arguments {
                collect_bindings(argument, &method_parameters, &candidates, &mut bindings)?;
            }
        }
    }
    Some(bindings)
}

fn provider_candidates(
    provider: &ModuleInterface,
    scheme: &InterfaceExport,
) -> BTreeMap<String, Vec<ExternalTypeBinding>> {
    let mut candidates = BTreeMap::<String, Vec<ExternalTypeBinding>>::new();
    for export in provider
        .exports
        .iter()
        .filter(|export| export.namespace == "type")
    {
        push_candidate(
            &mut candidates,
            ExternalTypeBinding {
                arity: Some(export.scheme.type_parameters.len() as u32),
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
                    arity: None,
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
    // A reexport does not move the callable's defining scope. A type name in
    // its legacy Named scheme can therefore differ from the facade's alias.
    // Restore that name only through a checked declaration/import whose full
    // canonical owner matches the callable, retaining the actual provider route.
    // Quantified parameters still take precedence in collect_named.
    let Some((owner, _)) = scheme.symbol.rsplit_once("::") else {
        return candidates;
    };
    if owner == provider.module {
        // Within the defining module, conflicting lexical imports must keep
        // the existing ambiguity refusal rather than being overwritten.
        return candidates;
    }
    let mut defining = BTreeMap::new();
    for export in provider.exports.iter().filter(|e| e.namespace == "type") {
        let InterfaceType::TypeConstructor { name, arity } = &export.scheme.type_ref else {
            continue;
        };
        if export.symbol == format!("{owner}::{name}")
            && *arity as usize == export.scheme.type_parameters.len()
        {
            push_candidate(
                &mut defining,
                ExternalTypeBinding {
                    arity: Some(*arity),
                    spelling: name.clone(),
                    canonical: export.symbol.clone(),
                    provider: Some(ExternalTypeProvider {
                        module: provider.module.clone(),
                        export: export.name.clone(),
                    }),
                },
            );
        }
    }
    for dependency in &provider.dependencies {
        for import in dependency.imports.iter().filter(|i| i.namespace == "type") {
            if import.symbol == format!("{owner}::{}", import.name) {
                push_candidate(
                    &mut defining,
                    ExternalTypeBinding {
                        arity: None,
                        spelling: import.name.clone(),
                        canonical: import.symbol.clone(),
                        provider: Some(ExternalTypeProvider {
                            module: dependency.module.clone(),
                            export: import.name.clone(),
                        }),
                    },
                );
            }
        }
    }
    candidates.extend(defining);
    candidates
}

fn push_candidate(
    candidates: &mut BTreeMap<String, Vec<ExternalTypeBinding>>,
    candidate: ExternalTypeBinding,
) {
    let same_spelling = candidates.entry(candidate.spelling.clone()).or_default();
    // A re-export and its dependency import name the same type identity.
    // Prefer the exported route (inserted first), retaining its known arity.
    if let Some(existing) = same_spelling
        .iter_mut()
        .find(|binding| binding.canonical == candidate.canonical)
    {
        existing.arity = existing.arity.or(candidate.arity);
    } else {
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
                arity: candidates
                    .values()
                    .flatten()
                    .find(|binding| binding.canonical == *canonical)
                    .and_then(|binding| binding.arity),
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
        InterfaceType::RequirementMerge { operands } => {
            for operand in operands {
                collect_bindings(operand, type_parameters, candidates, bindings)?;
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
    use super::export_scheme_type_bindings;
    use seseragi_syntax::{
        ByteSpan, InterfaceDependency, InterfaceExport, InterfaceImport, InterfaceMethod,
        InterfaceScheme, InterfaceType, ModuleInterface, Visibility,
    };

    #[test]
    fn collects_provider_types_from_trait_method_contracts() {
        let trait_export = InterfaceExport {
            symbol: "fixture/provider::trait(Convert)".to_owned(),
            namespace: "trait".to_owned(),
            name: "Convert".to_owned(),
            deprecation: None,
            constructor_of: None,
            call_arity: None,
            visibility: Visibility::Public,
            declaration_kind: Some("trait".to_owned()),
            declaration: ByteSpan { start: 5, end: 40 },
            scheme: InterfaceScheme {
                type_parameters: vec![seseragi_syntax::TypeParameter::value("A")],
                constraints: Vec::new(),
                type_ref: InterfaceType::TypeConstructor {
                    name: "Convert".to_owned(),
                    arity: 1,
                },
            },
            methods: vec![InterfaceMethod {
                deprecation: None,
                name: "convert".to_owned(),
                scheme: InterfaceScheme {
                    type_parameters: Vec::new(),
                    constraints: Vec::new(),
                    type_ref: InterfaceType::Function {
                        parameter: Box::new(named("User")),
                        result: Box::new(named("A")),
                    },
                },
                origin: ByteSpan { start: 20, end: 35 },
            }],
            representation: None,
        };
        let provider = module(vec![
            type_export("User", "fixture/provider::User"),
            trait_export.clone(),
        ]);

        let bindings = export_scheme_type_bindings(&provider, &trait_export).unwrap();

        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].spelling, "User");
        assert_eq!(bindings[0].canonical, "fixture/provider::User");
    }

    #[test]
    fn collects_provider_types_from_operator_schemes() {
        let provider = module(vec![type_export("User", "fixture/provider::User")]);
        let mut operator = function_export(
            "<^>",
            named("User"),
            InterfaceType::Function {
                parameter: Box::new(named("User")),
                result: Box::new(named("User")),
            },
        );
        operator.symbol = "fixture/provider::operator(<^>)".to_owned();
        operator.namespace = "operator".to_owned();
        operator.declaration_kind = Some("custom-operator".to_owned());

        let bindings = export_scheme_type_bindings(&provider, &operator).unwrap();

        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].spelling, "User");
        assert_eq!(bindings[0].canonical, "fixture/provider::User");
    }

    #[test]
    fn collects_provider_types_from_constructor_payloads() {
        let provider = module(vec![
            type_export("ResponseHead", "std/http::ResponseHead"),
            InterfaceExport {
                symbol: "std/http::ResponseStarted".to_owned(),
                namespace: "value".to_owned(),
                name: "ResponseStarted".to_owned(),
                deprecation: None,
                constructor_of: Some("std/http::HttpEvent".to_owned()),
                call_arity: None,
                visibility: Visibility::Public,
                declaration_kind: Some("constructor".to_owned()),
                declaration: ByteSpan { start: 0, end: 0 },
                scheme: InterfaceScheme {
                    type_parameters: Vec::new(),
                    constraints: Vec::new(),
                    type_ref: InterfaceType::Function {
                        parameter: Box::new(named("ResponseHead")),
                        result: Box::new(InterfaceType::ExternalNamed {
                            name: "HttpEvent".to_owned(),
                            canonical: "std/http::HttpEvent".to_owned(),
                            provider_module: "std/http".to_owned(),
                            provider_export: "HttpEvent".to_owned(),
                            arguments: Vec::new(),
                        }),
                    },
                },
                methods: Vec::new(),
                representation: None,
            },
        ]);
        let constructor = &provider.exports[1];

        let bindings = export_scheme_type_bindings(&provider, constructor).unwrap();

        assert_eq!(bindings.len(), 2);
        assert!(bindings.iter().any(|binding| {
            binding.spelling == "ResponseHead" && binding.canonical == "std/http::ResponseHead"
        }));
        assert!(bindings
            .iter()
            .any(|binding| binding.canonical == "std/http::HttpEvent"));
    }

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

        assert_eq!(export_scheme_type_bindings(&provider, &callable), None);
    }

    #[test]
    fn rejects_an_unresolved_non_prelude_scheme_type() {
        let provider = module(Vec::new());
        let callable = function_export("accept", named("Missing"), named("Unit"));

        assert_eq!(export_scheme_type_bindings(&provider, &callable), None);
    }

    fn renamed_provider() -> (ModuleInterface, InterfaceExport) {
        let mut user = type_export("User", "fixture/domain::User");
        user.name = "ExportedUser".to_owned();
        let mut callable = function_export("accept", named("User"), named("Unit"));
        callable.symbol = "fixture/domain::accept".to_owned();
        (module(vec![user]), callable)
    }

    #[test]
    fn reexported_callable_uses_its_defining_name_and_actual_facade_route() {
        let (provider, callable) = renamed_provider();
        let bindings = export_scheme_type_bindings(&provider, &callable).unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].spelling, "User");
        assert_eq!(bindings[0].canonical, "fixture/domain::User");
        let route = bindings[0].provider.as_ref().unwrap();
        assert_eq!(route.module, "fixture/provider");
        assert_eq!(route.export, "ExportedUser");
    }

    #[test]
    fn reexported_defining_name_rejects_wrong_owner_namespace_and_kind() {
        let (provider, callable) = renamed_provider();
        let mut wrong_owner = provider.clone();
        wrong_owner.exports[0].symbol = "fixture/unrelated::User".to_owned();
        let mut wrong_namespace = provider.clone();
        wrong_namespace.exports[0].namespace = "trait".to_owned();
        let mut wrong_kind = provider;
        wrong_kind.exports[0].scheme.type_ref = InterfaceType::TypeConstructor {
            name: "User".to_owned(),
            arity: 1,
        };
        for invalid in [wrong_owner, wrong_namespace, wrong_kind] {
            assert_eq!(export_scheme_type_bindings(&invalid, &callable), None);
        }
    }

    #[test]
    fn reexported_defining_name_does_not_capture_a_quantified_parameter() {
        let (provider, mut callable) = renamed_provider();
        callable
            .scheme
            .type_parameters
            .push(seseragi_syntax::TypeParameter::value("User"));
        assert_eq!(
            export_scheme_type_bindings(&provider, &callable),
            Some(Vec::new())
        );
    }

    fn module(exports: Vec<InterfaceExport>) -> ModuleInterface {
        ModuleInterface {
            namespace_deprecations: Vec::new(),
            operator_deprecations: Vec::new(),
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
            deprecation: None,
            constructor_of: None,
            call_arity: None,
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
            methods: Vec::new(),
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
            deprecation: None,
            constructor_of: None,
            call_arity: None,
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
