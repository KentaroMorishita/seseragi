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
        InterfaceType::ExternalNamed {
            name,
            canonical,
            arguments,
            ..
        } => Some(TypedType::ExternalNamed {
            name,
            canonical,
            arguments: arguments
                .into_iter()
                .map(typed_type_from_interface_type)
                .collect::<Option<Vec<_>>>()?,
        }),
        InterfaceType::Hole => Some(TypedType::Hole),
        InterfaceType::Record { closed, fields } => Some(TypedType::Record {
            closed,
            fields: fields
                .into_iter()
                .map(|field| {
                    Some(crate::TypedRecordField {
                        name: field.name,
                        optional: field.optional,
                        type_ref: typed_type_from_interface_type(field.type_ref)?,
                    })
                })
                .collect::<Option<Vec<_>>>()?,
        }),
        InterfaceType::Tuple { elements } => Some(TypedType::Tuple {
            elements: elements
                .into_iter()
                .map(typed_type_from_interface_type)
                .collect::<Option<Vec<_>>>()?,
        }),
        InterfaceType::Function { parameter, result } => Some(TypedType::Function {
            parameter: Box::new(typed_type_from_interface_type(*parameter)?),
            result: Box::new(typed_type_from_interface_type(*result)?),
        }),
        InterfaceType::TypeConstructor { .. } | InterfaceType::Apply { .. } => None,
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
        TypeRef::Hole { .. } => TypedType::Hole,
        TypeRef::Record { closed, fields, .. } => TypedType::Record {
            closed: *closed,
            fields: fields
                .iter()
                .map(|field| crate::TypedRecordField {
                    name: field.name.clone(),
                    optional: field.optional,
                    type_ref: typed_type_from_type_ref(&field.type_ref),
                })
                .collect(),
        },
        TypeRef::Tuple { elements, .. } => TypedType::Tuple {
            elements: elements.iter().map(typed_type_from_type_ref).collect(),
        },
        TypeRef::Function {
            parameter, result, ..
        } => TypedType::Function {
            parameter: Box::new(typed_type_from_type_ref(parameter)),
            result: Box::new(typed_type_from_type_ref(result)),
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
        TypedExpr::Boolean { .. } => TypedType::Named {
            name: "Bool".to_owned(),
            arguments: Vec::new(),
        },
        TypedExpr::Variable { type_ref, .. }
        | TypedExpr::FieldAccess { type_ref, .. }
        | TypedExpr::OptionalFieldAccess { type_ref, .. }
        | TypedExpr::Template { type_ref, .. }
        | TypedExpr::Call { type_ref, .. }
        | TypedExpr::Tuple { type_ref, .. }
        | TypedExpr::Array { type_ref, .. }
        | TypedExpr::List { type_ref, .. }
        | TypedExpr::Record { type_ref, .. }
        | TypedExpr::ArrayComprehension { type_ref, .. }
        | TypedExpr::ListComprehension { type_ref, .. }
        | TypedExpr::Binary { type_ref, .. }
        | TypedExpr::If { type_ref, .. }
        | TypedExpr::Match { type_ref, .. } => type_ref.clone(),
        TypedExpr::EffectCall { effect, .. } | TypedExpr::EffectInvoke { effect, .. } => {
            effect.success.clone()
        }
        TypedExpr::DoBlock { result, .. } => inferred_type_from_expr(result),
        TypedExpr::MonadDo { type_ref, .. } => type_ref.clone(),
    }
}

pub(crate) fn typed_type_contains_hole(type_ref: &TypedType) -> bool {
    match type_ref {
        TypedType::Hole => true,
        TypedType::Named { arguments, .. } | TypedType::ExternalNamed { arguments, .. } => {
            arguments.iter().any(typed_type_contains_hole)
        }
        TypedType::Record { fields, .. } => fields
            .iter()
            .any(|field| typed_type_contains_hole(&field.type_ref)),
        TypedType::Tuple { elements } => elements.iter().any(typed_type_contains_hole),
        TypedType::Function { parameter, result } => {
            typed_type_contains_hole(parameter) || typed_type_contains_hole(result)
        }
    }
}
