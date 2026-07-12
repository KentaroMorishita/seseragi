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
