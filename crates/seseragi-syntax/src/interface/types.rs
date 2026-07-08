use crate::surface::TypeRef;

use super::{InterfaceRecordField, InterfaceType};

pub(super) fn interface_type_from_type_ref(type_ref: &TypeRef) -> InterfaceType {
    match type_ref {
        TypeRef::Named {
            name, arguments, ..
        } => InterfaceType::Named {
            name: name.clone(),
            arguments: arguments.iter().map(interface_type_from_type_ref).collect(),
        },
        TypeRef::Record { closed, fields, .. } => InterfaceType::Record {
            closed: *closed,
            fields: fields
                .iter()
                .map(|field| InterfaceRecordField {
                    name: field.name.clone(),
                    optional: field.optional,
                    type_ref: interface_type_from_type_ref(&field.type_ref),
                })
                .collect(),
        },
    }
}
