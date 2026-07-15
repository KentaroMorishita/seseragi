use crate::prelude::is_standalone_symbol;
use crate::{ResolvedModule, SymbolId, SymbolNamespace, TypedConstraint};
use std::collections::{BTreeMap, BTreeSet};

use super::super::functions::TopLevelPureFunction;
use super::contains_function_type;
use super::imported_types::{flatten_function, ImportedTypeContext};

pub(super) fn collect_imported_callables(
    resolved: &ResolvedModule,
) -> BTreeMap<SymbolId, TopLevelPureFunction> {
    let types = ImportedTypeContext::new(resolved);
    resolved
        .imports
        .iter()
        .filter_map(|import| {
            if !import.in_scope {
                return None;
            }
            let export = &import.export;
            let scheme_type_bindings = import.scheme_type_bindings.as_deref()?;
            if export.declaration_kind.as_deref() != Some("function") {
                return None;
            }
            let (parameter_interfaces, result_interface) =
                flatten_function(export.scheme.type_ref.clone());
            let type_parameters = export
                .scheme
                .type_parameters
                .iter()
                .cloned()
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
                    if let Some(binding) =
                        import
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
            let result_type = result.type_ref.clone();
            if parameter_types.is_empty()
                || parameter_types.iter().any(contains_function_type)
                || contains_function_type(&result_type)
            {
                return None;
            }
            let semantic_parameters = parameters
                .into_iter()
                .map(|parameter| parameter.key)
                .collect();
            let semantic_result = result.key;
            Some((
                import.symbol,
                TopLevelPureFunction {
                    symbol: export.symbol.clone(),
                    trait_identity: None,
                    trait_method: None,
                    type_parameters: export.scheme.type_parameters.clone(),
                    constraints,
                    constraint_identities,
                    parameters: parameter_types,
                    semantic_parameters,
                    result: result_type,
                    semantic_result,
                },
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use seseragi_syntax::InterfaceType;

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
