use crate::{CoreExpr, CoreParameter, CoreType};

use super::names::{local_name, safe_identifier};
use super::{TypeScriptParameter, TypeScriptType};

pub(super) fn type_ref_from_core_expr(expr: &CoreExpr) -> TypeScriptType {
    match expr {
        CoreExpr::Unit { .. } => TypeScriptType::Undefined,
        CoreExpr::Int64 { .. } => TypeScriptType::Bigint,
        CoreExpr::String { .. } => TypeScriptType::String,
        CoreExpr::Boolean { .. } => TypeScriptType::Boolean,
        CoreExpr::Variable { type_ref, .. }
        | CoreExpr::Call { type_ref, .. }
        | CoreExpr::Tuple { type_ref, .. }
        | CoreExpr::Binary { type_ref, .. }
        | CoreExpr::If { type_ref, .. }
        | CoreExpr::Decision { type_ref, .. } => type_ref_from_core_type(type_ref),
        CoreExpr::EffectOperation { success, .. } => type_ref_from_core_type(success),
        CoreExpr::Sequence { result, .. } => type_ref_from_core_expr(result),
    }
}

pub(super) fn lower_core_parameter_to_typescript(parameter: CoreParameter) -> TypeScriptParameter {
    TypeScriptParameter {
        name: if parameter.kind == "implicit" {
            "_unit".to_owned()
        } else {
            safe_identifier(&parameter.id)
        },
        type_name: render_core_type(&parameter.type_ref),
        implicit: parameter.kind == "implicit",
    }
}

pub(super) fn type_ref_from_core_type(type_ref: &CoreType) -> TypeScriptType {
    match type_ref {
        CoreType::Named { name, arguments } if name == "Int" && arguments.is_empty() => {
            TypeScriptType::Bigint
        }
        CoreType::Named { name, arguments } if name == "String" && arguments.is_empty() => {
            TypeScriptType::String
        }
        CoreType::Named { name, arguments } if name == "Bool" && arguments.is_empty() => {
            TypeScriptType::Boolean
        }
        CoreType::Named { name, arguments } if name == "Unit" && arguments.is_empty() => {
            TypeScriptType::Undefined
        }
        CoreType::Named { name, arguments } if name == "Maybe" && arguments.len() == 1 => {
            TypeScriptType::Maybe {
                element: Box::new(type_ref_from_core_type(&arguments[0])),
            }
        }
        CoreType::Named { name, arguments } => TypeScriptType::Reference {
            name: local_name(name),
            arguments: arguments.iter().map(type_ref_from_core_type).collect(),
        },
        CoreType::Tuple { elements } => TypeScriptType::Tuple {
            elements: elements.iter().map(type_ref_from_core_type).collect(),
        },
        CoreType::Hole | CoreType::Record { .. } | CoreType::Function { .. } => {
            TypeScriptType::Unknown
        }
    }
}

pub(super) fn render_core_type(type_ref: &CoreType) -> String {
    render_typescript_type(&type_ref_from_core_type(type_ref))
}

pub(crate) fn render_typescript_type(type_ref: &TypeScriptType) -> String {
    match type_ref {
        TypeScriptType::Bigint => "bigint".to_owned(),
        TypeScriptType::Boolean => "boolean".to_owned(),
        TypeScriptType::String => "string".to_owned(),
        TypeScriptType::Undefined => "undefined".to_owned(),
        TypeScriptType::Unknown => "unknown".to_owned(),
        TypeScriptType::Reference { name, arguments } if arguments.is_empty() => name.clone(),
        TypeScriptType::Reference { name, arguments } => format!(
            "{name}<{}>",
            arguments
                .iter()
                .map(render_typescript_type)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        TypeScriptType::Maybe { element } => {
            format!("{} | undefined", render_typescript_type(element))
        }
        TypeScriptType::Tuple { elements } => format!(
            "readonly [{}]",
            elements
                .iter()
                .map(render_typescript_type)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{render_core_type, CoreType};

    #[test]
    fn renders_qualified_nominal_types_with_local_backend_names() {
        let type_ref = CoreType::Named {
            name: "artifact/domain::Hand".to_owned(),
            arguments: Vec::new(),
        };

        assert_eq!(render_core_type(&type_ref), "Hand");
    }
}
