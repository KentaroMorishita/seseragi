use crate::TypedType;
use seseragi_syntax::{SurfaceDecl, SurfaceParameter};
use std::collections::BTreeMap;

use super::type_ref::typed_type_from_type_ref;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TopLevelPureFunction {
    pub(crate) symbol: String,
    pub(crate) parameters: Vec<TypedType>,
    pub(crate) result: TypedType,
}

pub(crate) fn collect_top_level_pure_function_signatures(
    module: &str,
    declarations: &[SurfaceDecl],
) -> BTreeMap<String, TopLevelPureFunction> {
    declarations
        .iter()
        .filter_map(|declaration| {
            let SurfaceDecl::Fn {
                name,
                type_parameters,
                parameters,
                return_type,
                constraints,
                ..
            } = declaration
            else {
                return None;
            };

            if !type_parameters.is_empty() || !constraints.is_empty() || parameters.is_empty() {
                return None;
            }

            let parameters = parameters
                .iter()
                .map(|parameter| typed_type_from_type_ref(&parameter.type_ref))
                .collect::<Vec<_>>();
            let result = typed_type_from_type_ref(return_type);
            if parameters.iter().any(contains_function_type) || contains_function_type(&result) {
                return None;
            }

            Some((
                name.clone(),
                TopLevelPureFunction {
                    symbol: format!("{module}::{name}"),
                    parameters,
                    result,
                },
            ))
        })
        .collect()
}

pub(crate) fn typed_parameters_from_surface(
    parameters: &[SurfaceParameter],
) -> Vec<crate::TypedParameter> {
    parameters
        .iter()
        .map(|parameter| crate::TypedParameter::Named {
            name: parameter.name.clone(),
            type_ref: typed_type_from_type_ref(&parameter.type_ref),
            origin: parameter.name_span,
        })
        .collect()
}

pub(crate) fn application_result_type(
    signature: &TopLevelPureFunction,
    argument_count: usize,
) -> TypedType {
    signature.parameters[argument_count..].iter().rev().fold(
        signature.result.clone(),
        |result, parameter| TypedType::Function {
            parameter: Box::new(parameter.clone()),
            result: Box::new(result),
        },
    )
}

fn contains_function_type(type_ref: &TypedType) -> bool {
    match type_ref {
        TypedType::Function { .. } => true,
        TypedType::Named { arguments, .. } => arguments.iter().any(contains_function_type),
        TypedType::Record { fields, .. } => fields
            .iter()
            .any(|field| contains_function_type(&field.type_ref)),
        TypedType::Tuple { elements } => elements.iter().any(contains_function_type),
        TypedType::Hole => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use seseragi_syntax::parse_surface_ast;

    #[test]
    fn excludes_generic_and_higher_order_functions_from_direct_call_signatures() {
        let module = parse_surface_ast(
            "main.ssrg",
            "fn identity<A> value: A -> A = value\nfn apply f: (Int -> Int) -> value: Int -> Int = f value\n",
        );

        let signatures =
            collect_top_level_pure_function_signatures("artifact/functions", &module.declarations);

        assert!(signatures.is_empty());
    }
}
