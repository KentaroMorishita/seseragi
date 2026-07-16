use crate::{SurfaceConstraint, SurfaceMethod, SurfaceParameter, TypeRef};

use super::types::interface_type_from_type_ref;
use super::{InterfaceConstraint, InterfaceMethod, InterfaceScheme, InterfaceType};

pub(super) fn interface_method_from_surface(method: &SurfaceMethod) -> InterfaceMethod {
    InterfaceMethod {
        name: method.name.clone(),
        scheme: InterfaceScheme {
            type_parameters: method.type_parameters.clone(),
            constraints: method
                .constraints
                .iter()
                .map(interface_constraint_from_surface)
                .collect(),
            type_ref: function_interface_type(&method.parameters, &method.return_type),
        },
        origin: method.span,
    }
}

pub(super) fn interface_constraint_from_surface(
    constraint: &SurfaceConstraint,
) -> InterfaceConstraint {
    InterfaceConstraint {
        name: constraint.name.clone(),
        trait_identity: None,
        arguments: constraint
            .arguments
            .iter()
            .map(interface_type_from_type_ref)
            .collect(),
    }
}

pub(super) fn function_interface_type(
    parameters: &[SurfaceParameter],
    return_type: &TypeRef,
) -> InterfaceType {
    parameters.iter().rev().fold(
        interface_type_from_type_ref(return_type),
        |result, parameter| InterfaceType::Function {
            parameter: Box::new(interface_type_from_type_ref(&parameter.type_ref)),
            result: Box::new(result),
        },
    )
}
