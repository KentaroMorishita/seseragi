use crate::prelude::is_standalone_symbol;
use crate::{ResolvedImport, ResolvedModule, SymbolId, SymbolNamespace, TypedConstraint};
use std::collections::{BTreeMap, BTreeSet};

use super::super::functions::TopLevelPureFunction;
use super::imported_types::{flatten_function, ImportedTypeContext};

pub(super) fn collect_imported_callables(
    resolved: &ResolvedModule,
) -> BTreeMap<SymbolId, TopLevelPureFunction> {
    let types = ImportedTypeContext::new(resolved);
    resolved
        .imports
        .iter()
        .filter(|import| {
            import.in_scope
                && (import.export.namespace == "operator"
                    || import.export.declaration_kind.as_deref() == Some("function"))
        })
        .filter_map(|import| {
            imported_callable(&types, import).map(|callable| (import.symbol, callable))
        })
        .collect()
}

pub(super) fn imported_callable(
    types: &ImportedTypeContext,
    import: &ResolvedImport,
) -> Option<TopLevelPureFunction> {
    let export = &import.export;
    let scheme_type_bindings = import.scheme_type_bindings.as_deref()?;
    let (parameter_interfaces, result_interface) = flatten_function(export.scheme.type_ref.clone());
    let type_parameters = export
        .scheme
        .type_parameters
        .iter()
        .map(|parameter| parameter.name.clone())
        .collect::<BTreeSet<_>>();
    let parameters = parameter_interfaces
        .into_iter()
        .map(|type_ref| {
            types.semantic_value(
                type_ref,
                &import.module,
                &type_parameters,
                scheme_type_bindings,
            )
        })
        .collect::<Option<Vec<_>>>()?;
    let result = types.semantic_value(
        result_interface,
        &import.module,
        &type_parameters,
        scheme_type_bindings,
    )?;
    let constraints = export
        .scheme
        .constraints
        .iter()
        .map(|constraint| {
            let arguments = constraint
                .arguments
                .iter()
                .cloned()
                .map(|argument| {
                    types
                        .semantic_value(
                            argument,
                            &import.module,
                            &type_parameters,
                            scheme_type_bindings,
                        )
                        .map(|argument| argument.type_ref)
                })
                .collect::<Option<Vec<_>>>()?;
            Some(TypedConstraint {
                name: constraint.name.clone(),
                arguments,
            })
        })
        .collect::<Option<Vec<_>>>()?;
    let constraint_identities = export
        .scheme
        .constraints
        .iter()
        .map(|constraint| {
            if let Some(binding) = import
                .scheme_trait_bindings
                .as_deref()
                .and_then(|bindings| {
                    bindings
                        .iter()
                        .find(|binding| binding.spelling == constraint.name)
                })
            {
                return Some(Some(binding.canonical.clone()));
            }
            is_standalone_symbol(SymbolNamespace::Trait, &constraint.name).then_some(None)
        })
        .collect::<Option<Vec<_>>>()?;
    let parameter_types = parameters
        .iter()
        .map(|parameter| parameter.type_ref.clone())
        .collect::<Vec<_>>();
    if parameter_types.is_empty() {
        return None;
    }
    Some(TopLevelPureFunction {
        symbol: export.symbol.clone(),
        trait_identity: None,
        trait_method: None,
        type_parameters: export.scheme.type_parameters.clone(),
        constraints,
        constraint_identities,
        parameters: parameter_types,
        semantic_parameters: parameters
            .into_iter()
            .map(|parameter| parameter.key)
            .collect(),
        result: result.type_ref,
        semantic_result: result.key,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use seseragi_syntax::InterfaceType;

    #[test]
    fn types_an_imported_operator_from_its_public_scheme() {
        let provider_source = "\
pub operator infixr 4 <^>\n\
  left: Int -> right: Int -> Int =\n\
  left - right\n";
        let provider = seseragi_syntax::parse_unlinked_module_interface(
            "provider.ssrg",
            "fixture/provider",
            provider_source,
        );
        let provider_interface = crate::analyze_module_interface(
            seseragi_syntax::parse_diagnostics("provider.ssrg", provider_source),
            provider.interface.clone(),
            provider_source,
        )
        .unwrap()
        .typed_interface
        .into_link_interface();
        let target =
            seseragi_project::ModuleLinkTarget::same_package(provider.header, provider_interface)
                .unwrap();
        let consumer_source = "\
import { operator <^> } from \"./provider\"\n\
pub fn run unit: Unit -> Int = 10 <^> 3 <^> 2\n";
        let consumer = seseragi_syntax::parse_unlinked_module_interface(
            "consumer.ssrg",
            "fixture/consumer",
            consumer_source,
        );
        let linked = seseragi_project::link_module(
            consumer,
            &BTreeMap::from([("./provider".to_owned(), target)]),
        )
        .unwrap();

        let analyzed = crate::analyze_linked_module(
            seseragi_syntax::parse_diagnostics("consumer.ssrg", consumer_source),
            linked,
            consumer_source,
        )
        .unwrap();

        let dependency = &analyzed.typed_hir.module_dependencies[0];
        assert_eq!(dependency.imports[0].namespace, "operator");
        assert_eq!(
            dependency.imports[0].canonical,
            "fixture/provider::operator(<^>)"
        );
        let crate::TypedDecl::Fn { body, .. } = &analyzed.typed_hir.declarations[0] else {
            panic!("expected typed consumer function");
        };
        assert!(matches!(
            body,
            crate::TypedExpr::Call {
                callee,
                arguments,
                type_ref: crate::TypedType::Named { name, arguments: type_arguments },
                ..
            } if callee == "fixture/provider::operator(<^>)"
                && arguments.len() == 2
                && matches!(
                    &arguments[1],
                    crate::TypedExpr::Call { callee, .. }
                        if callee == "fixture/provider::operator(<^>)"
                )
                && name == "Int"
                && type_arguments.is_empty()
        ));
    }

    #[test]
    fn flattens_a_curried_imported_function_scheme() {
        let (parameters, result) = flatten_function(InterfaceType::Function {
            parameter: Box::new(named("Int")),
            result: Box::new(InterfaceType::Function {
                parameter: Box::new(named("Int")),
                result: Box::new(named("Int")),
            }),
        });

        assert_eq!(parameters, vec![named("Int"), named("Int")]);
        assert_eq!(result, named("Int"));
    }

    fn named(name: &str) -> InterfaceType {
        InterfaceType::Named {
            name: name.to_owned(),
            arguments: Vec::new(),
        }
    }
}
