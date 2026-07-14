use crate::surface::SurfaceDecl;

use super::types::interface_type_from_type_ref;
use super::{InterfaceConstraint, InterfaceInstance, InterfaceType};

pub(super) fn instance_from_surface_decl(declaration: SurfaceDecl) -> Option<InterfaceInstance> {
    match declaration {
        SurfaceDecl::Instance {
            type_parameters,
            trait_name,
            arguments,
            constraints,
            methods: _,
            span,
            ..
        } => Some(InterfaceInstance {
            identity: None,
            provider_module: None,
            type_identity: None,
            trait_name: trait_name.clone(),
            type_parameters,
            head: InterfaceType::Apply {
                constructor: trait_name,
                arguments: arguments.iter().map(interface_type_from_type_ref).collect(),
            },
            constraints: constraints
                .iter()
                .map(|constraint| InterfaceConstraint {
                    name: constraint.name.clone(),
                    arguments: constraint
                        .arguments
                        .iter()
                        .map(interface_type_from_type_ref)
                        .collect(),
                })
                .collect(),
            origin: span,
        }),
        _ => None,
    }
}
