use crate::{unit_type, TypedExpr, TypedType};
use seseragi_syntax::{InterfaceType, TypeRef};

pub(crate) fn typed_type_from_interface_type(type_ref: InterfaceType) -> Option<TypedType> {
    match type_ref {
        InterfaceType::Named { name, arguments } => Some(TypedType::Named {
            name,
            arguments: arguments
                .into_iter()
                .map(typed_type_from_interface_type)
                .collect::<Option<Vec<_>>>()?,
        }),
        InterfaceType::TypeConstructor { .. }
        | InterfaceType::Function { .. }
        | InterfaceType::Apply { .. } => None,
    }
}

pub(crate) fn typed_type_from_type_ref(type_ref: &TypeRef) -> TypedType {
    match type_ref {
        TypeRef::Named {
            name, arguments, ..
        } => TypedType::Named {
            name: name.clone(),
            arguments: arguments.iter().map(typed_type_from_type_ref).collect(),
        },
    }
}

pub(crate) fn inferred_type_from_expr(expr: &TypedExpr) -> TypedType {
    match expr {
        TypedExpr::Unit { .. } => unit_type(),
        TypedExpr::Integer { .. } => TypedType::Named {
            name: "Int".to_owned(),
            arguments: Vec::new(),
        },
        TypedExpr::String { .. } => TypedType::Named {
            name: "String".to_owned(),
            arguments: Vec::new(),
        },
        TypedExpr::EffectCall { .. } => unit_type(),
        TypedExpr::DoBlock { result, .. } => inferred_type_from_expr(result),
    }
}
