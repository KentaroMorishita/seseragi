use crate::surface::TypeRef;

use super::InterfaceType;

pub(super) fn interface_type_from_type_ref(type_ref: &TypeRef) -> InterfaceType {
    match type_ref {
        TypeRef::Named {
            name, arguments, ..
        } => InterfaceType::Named {
            name: name.clone(),
            arguments: arguments.iter().map(interface_type_from_type_ref).collect(),
        },
    }
}
