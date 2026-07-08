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
        TypeRef::Tuple { elements, .. } => InterfaceType::Tuple {
            elements: elements.iter().map(interface_type_from_type_ref).collect(),
        },
        TypeRef::Function {
            parameter, result, ..
        } => InterfaceType::Function {
            parameter: Box::new(interface_type_from_type_ref(parameter)),
            result: Box::new(interface_type_from_type_ref(result)),
        },
    }
}
