use crate::surface::SurfaceDecl;

use super::types::interface_type_from_type_ref;
use super::{InterfaceConstraint, InterfaceInstance, InterfaceType};

pub(super) fn instances_from_surface_decl(declaration: &SurfaceDecl) -> Vec<InterfaceInstance> {
    let mut instances = crate::impl_operator_instances(declaration)
        .iter()
        .filter_map(instance_from_surface_decl)
        .collect::<Vec<_>>();
    instances.extend(instance_from_surface_decl(declaration));
    instances
}

fn instance_from_surface_decl(declaration: &SurfaceDecl) -> Option<InterfaceInstance> {
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
            trait_identity: None,
            argument_identities: Vec::new(),
            type_identity: None,
            trait_name: trait_name.clone(),
            type_parameters: type_parameters.clone(),
            head: InterfaceType::Apply {
                constructor: trait_name.clone(),
                arguments: arguments.iter().map(interface_type_from_type_ref).collect(),
            },
            constraints: constraints
                .iter()
                .map(|constraint| InterfaceConstraint {
                    name: constraint.name.clone(),
                    trait_identity: None,
                    arguments: constraint
                        .arguments
                        .iter()
                        .map(interface_type_from_type_ref)
                        .collect(),
                })
                .collect(),
            origin: *span,
        }),
        _ => None,
    }
}
