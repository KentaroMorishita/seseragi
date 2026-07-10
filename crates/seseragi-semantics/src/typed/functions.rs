use crate::TypedType;
use seseragi_syntax::SurfaceParameter;

use super::type_ref::typed_type_from_type_ref;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TopLevelPureFunction {
    pub(crate) symbol: String,
    pub(crate) parameters: Vec<TypedType>,
    pub(crate) result: TypedType,
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
