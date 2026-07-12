use crate::{ResolvedModule, SymbolId, TypedType};
use seseragi_syntax::InterfaceType;
use std::collections::BTreeMap;

use super::super::functions::TopLevelPureFunction;
use super::super::semantic_types::SemanticTypeKey;
use super::super::type_ref::typed_type_from_interface_type;
use super::contains_function_type;

pub(super) fn collect_imported_callables(
    resolved: &ResolvedModule,
) -> BTreeMap<SymbolId, TopLevelPureFunction> {
    let type_names = resolved
        .imports
        .iter()
        .filter(|import| import.in_scope && import.export.namespace == "type")
        .map(|import| {
            (
                (import.module.clone(), import.export.name.clone()),
                import.local_name.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let type_owners = resolved
        .imports
        .iter()
        .filter(|import| {
            import.export.namespace == "type"
                && import.export.declaration_kind.as_deref() == Some("type")
        })
        .map(|import| {
            (
                (import.module.clone(), import.export.name.clone()),
                import.symbol,
            )
        })
        .collect::<BTreeMap<_, _>>();
    resolved
        .imports
        .iter()
        .filter_map(|import| {
            if !import.in_scope {
                return None;
            }
            let export = &import.export;
            if export.declaration_kind.as_deref() != Some("function")
                || !export.scheme.type_parameters.is_empty()
                || !export.scheme.constraints.is_empty()
            {
                return None;
            }
            let (parameter_interfaces, result_interface) =
                flatten_function(export.scheme.type_ref.clone())?;
            let parameters = parameter_interfaces
                .into_iter()
                .map(|type_ref| {
                    semantic_value_from_interface_type(
                        type_ref,
                        &import.module,
                        &type_names,
                        &type_owners,
                    )
                })
                .collect::<Option<Vec<_>>>()?;
            let result = semantic_value_from_interface_type(
                result_interface,
                &import.module,
                &type_names,
                &type_owners,
            )?;
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
                    type_parameters: Vec::new(),
                    parameters: parameter_types,
                    semantic_parameters,
                    result: result_type,
                    semantic_result,
                },
            ))
        })
        .collect()
}

fn localize_type(
    type_ref: TypedType,
    module: &str,
    names: &BTreeMap<(String, String), String>,
) -> TypedType {
    match type_ref {
        TypedType::Named { name, arguments } => TypedType::Named {
            name: names
                .get(&(module.to_owned(), name.clone()))
                .cloned()
                .unwrap_or(name),
            arguments: arguments
                .into_iter()
                .map(|argument| localize_type(argument, module, names))
                .collect(),
        },
        TypedType::Record { closed, fields } => TypedType::Record {
            closed,
            fields: fields
                .into_iter()
                .map(|field| crate::TypedRecordField {
                    name: field.name,
                    optional: field.optional,
                    type_ref: localize_type(field.type_ref, module, names),
                })
                .collect(),
        },
        TypedType::Tuple { elements } => TypedType::Tuple {
            elements: elements
                .into_iter()
                .map(|element| localize_type(element, module, names))
                .collect(),
        },
        TypedType::Function { parameter, result } => TypedType::Function {
            parameter: Box::new(localize_type(*parameter, module, names)),
            result: Box::new(localize_type(*result, module, names)),
        },
        TypedType::Hole => TypedType::Hole,
    }
}

fn semantic_value_from_interface_type(
    type_ref: InterfaceType,
    module: &str,
    names: &BTreeMap<(String, String), String>,
    owners: &BTreeMap<(String, String), SymbolId>,
) -> Option<super::super::semantic_types::SemanticValueType> {
    let key = semantic_key_from_interface_type(&type_ref, module, names, owners)?;
    let type_ref = typed_type_from_interface_type(type_ref)?;
    Some(super::super::semantic_types::SemanticValueType {
        type_ref: localize_type(type_ref, module, names),
        key,
    })
}

fn semantic_key_from_interface_type(
    type_ref: &InterfaceType,
    module: &str,
    names: &BTreeMap<(String, String), String>,
    owners: &BTreeMap<(String, String), SymbolId>,
) -> Option<SemanticTypeKey> {
    match type_ref {
        InterfaceType::Named { name, arguments } => {
            let Some(owner) = owners.get(&(module.to_owned(), name.clone())) else {
                return Some(SemanticTypeKey::Other);
            };
            let arguments = arguments
                .iter()
                .cloned()
                .map(|argument| semantic_value_from_interface_type(argument, module, names, owners))
                .collect::<Option<Vec<_>>>()?;
            Some(SemanticTypeKey::Adt {
                owner: *owner,
                arguments,
            })
        }
        InterfaceType::Tuple { elements } => Some(SemanticTypeKey::Tuple(
            elements
                .iter()
                .map(|element| semantic_key_from_interface_type(element, module, names, owners))
                .collect::<Option<Vec<_>>>()?,
        )),
        InterfaceType::Hole => Some(SemanticTypeKey::Invalid),
        InterfaceType::Function { .. }
        | InterfaceType::TypeConstructor { .. }
        | InterfaceType::Apply { .. }
        | InterfaceType::Record { .. } => Some(SemanticTypeKey::Other),
    }
}

fn flatten_function(type_ref: InterfaceType) -> Option<(Vec<InterfaceType>, InterfaceType)> {
    let mut parameters = Vec::new();
    let mut cursor = type_ref;
    loop {
        match cursor {
            InterfaceType::Function { parameter, result } => {
                parameters.push(*parameter);
                cursor = *result;
            }
            result => return Some((parameters, result)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flattens_a_curried_imported_function_scheme() {
        let (parameters, result) = flatten_function(InterfaceType::Function {
            parameter: Box::new(named("Int")),
            result: Box::new(InterfaceType::Function {
                parameter: Box::new(named("Int")),
                result: Box::new(named("Int")),
            }),
        })
        .unwrap();

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
