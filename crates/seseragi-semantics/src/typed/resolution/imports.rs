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
            let (parameter_types, result_type) = flatten_function(export.scheme.type_ref.clone())?;
            let parameter_types = parameter_types
                .into_iter()
                .map(|type_ref| localize_type(type_ref, &import.module, &type_names))
                .collect::<Vec<_>>();
            let result_type = localize_type(result_type, &import.module, &type_names);
            if parameter_types.is_empty()
                || parameter_types.iter().any(contains_function_type)
                || contains_function_type(&result_type)
            {
                return None;
            }
            let semantic_parameters = parameter_types
                .iter()
                .map(semantic_key_from_typed_type)
                .collect();
            let semantic_result = semantic_key_from_typed_type(&result_type);
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

fn flatten_function(type_ref: InterfaceType) -> Option<(Vec<TypedType>, TypedType)> {
    let mut parameters = Vec::new();
    let mut cursor = type_ref;
    loop {
        match cursor {
            InterfaceType::Function { parameter, result } => {
                parameters.push(typed_type_from_interface_type(*parameter)?);
                cursor = *result;
            }
            result => return Some((parameters, typed_type_from_interface_type(result)?)),
        }
    }
}

fn semantic_key_from_typed_type(type_ref: &TypedType) -> SemanticTypeKey {
    match type_ref {
        TypedType::Tuple { elements } => {
            SemanticTypeKey::Tuple(elements.iter().map(semantic_key_from_typed_type).collect())
        }
        TypedType::Hole => SemanticTypeKey::Invalid,
        _ => SemanticTypeKey::Other,
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

        assert_eq!(parameters, vec![typed_named("Int"), typed_named("Int")]);
        assert_eq!(result, typed_named("Int"));
    }

    fn named(name: &str) -> InterfaceType {
        InterfaceType::Named {
            name: name.to_owned(),
            arguments: Vec::new(),
        }
    }

    fn typed_named(name: &str) -> TypedType {
        TypedType::Named {
            name: name.to_owned(),
            arguments: Vec::new(),
        }
    }
}
