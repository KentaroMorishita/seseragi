pub(super) fn type_label(type_ref: &crate::TypedType) -> String {
    match type_ref {
        crate::TypedType::Named { name, arguments }
        | crate::TypedType::ExternalNamed {
            name, arguments, ..
        } if arguments.is_empty() => name.clone(),
        crate::TypedType::Named { name, arguments }
        | crate::TypedType::ExternalNamed {
            name, arguments, ..
        } => format!(
            "{}<{}>",
            name,
            arguments
                .iter()
                .map(type_label)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        crate::TypedType::Function { .. } => "function".to_owned(),
        crate::TypedType::Record { .. } => "record".to_owned(),
        crate::TypedType::Tuple { elements } => format!(
            "({})",
            elements
                .iter()
                .map(type_label)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        crate::TypedType::Hole => "unknown".to_owned(),
    }
}
