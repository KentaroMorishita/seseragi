use crate::TypedType;
use seseragi_syntax::SurfaceParameter;

use super::type_ref::typed_type_from_type_ref;

mod generic;

pub(crate) use generic::{instantiated_application, instantiated_application_result_type};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TopLevelPureFunction {
    pub(crate) symbol: String,
    pub(crate) type_parameters: Vec<String>,
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
    application_result_type_from(
        &signature.parameters,
        signature.result.clone(),
        argument_count,
    )
}

pub(super) fn application_result_type_from(
    parameters: &[TypedType],
    result: TypedType,
    argument_count: usize,
) -> TypedType {
    parameters[argument_count..]
        .iter()
        .rev()
        .fold(result, |result, parameter| TypedType::Function {
            parameter: Box::new(parameter.clone()),
            result: Box::new(result),
        })
}
