use crate::{CoreExpr, CoreParameter};

use super::{push_unique, TypeScriptParameter, TypeScriptType};

pub(super) fn collect_expr_runtime_requirements(expr: &CoreExpr, requirements: &mut Vec<String>) {
    match expr {
        CoreExpr::Unit { .. } => push_unique(requirements, "core.unit"),
        CoreExpr::Int64 { .. } => push_unique(requirements, "core.int64"),
        CoreExpr::String { .. } => push_unique(requirements, "core.string"),
        CoreExpr::Boolean { .. } => push_unique(requirements, "core.bool"),
        CoreExpr::Variable { type_name, .. } => {
            collect_type_runtime_requirement(type_name, requirements);
        }
        CoreExpr::EffectOperation {
            operation,
            arguments,
            ..
        } => {
            if operation == "console.println" {
                push_unique(requirements, "effect.console.println");
            }
            for argument in arguments {
                collect_expr_runtime_requirements(argument, requirements);
            }
        }
    }
}

pub(super) fn expr_requires_feature(expr: &CoreExpr, feature: &str) -> bool {
    match expr {
        CoreExpr::EffectOperation {
            operation,
            arguments,
            ..
        } => {
            (feature == "effect.console.println" && operation == "console.println")
                || arguments
                    .iter()
                    .any(|argument| expr_requires_feature(argument, feature))
        }
        CoreExpr::Unit { .. }
        | CoreExpr::Int64 { .. }
        | CoreExpr::String { .. }
        | CoreExpr::Boolean { .. }
        | CoreExpr::Variable { .. } => false,
    }
}

pub(super) fn type_ref_from_core_expr(expr: &CoreExpr) -> TypeScriptType {
    match expr {
        CoreExpr::Unit { .. } => TypeScriptType::Undefined,
        CoreExpr::Int64 { .. } => TypeScriptType::Bigint,
        CoreExpr::String { .. } => TypeScriptType::String,
        CoreExpr::Boolean { .. } => TypeScriptType::Boolean,
        CoreExpr::Variable { type_name, .. } => type_ref_from_type_name(type_name),
        CoreExpr::EffectOperation { .. } => TypeScriptType::Undefined,
    }
}

pub(super) fn lower_core_parameter_to_typescript(parameter: CoreParameter) -> TypeScriptParameter {
    TypeScriptParameter {
        name: if parameter.kind == "implicit" {
            "_unit".to_owned()
        } else {
            parameter.id
        },
        type_name: render_type_name(&parameter.type_name).to_owned(),
        implicit: parameter.kind == "implicit",
    }
}

fn type_ref_from_type_name(type_name: &str) -> TypeScriptType {
    match type_name {
        "Int" => TypeScriptType::Bigint,
        "String" => TypeScriptType::String,
        "Bool" => TypeScriptType::Boolean,
        "Unit" => TypeScriptType::Undefined,
        _ => TypeScriptType::Undefined,
    }
}

fn render_type_name(type_name: &str) -> &'static str {
    match type_name {
        "Int" => "bigint",
        "String" => "string",
        "Bool" => "boolean",
        "Unit" => "undefined",
        _ => "unknown",
    }
}

pub(super) fn collect_type_runtime_requirement(type_name: &str, requirements: &mut Vec<String>) {
    match type_name {
        "Int" => push_unique(requirements, "core.int64"),
        "String" => push_unique(requirements, "core.string"),
        "Bool" => push_unique(requirements, "core.bool"),
        "Unit" => push_unique(requirements, "core.unit"),
        _ => {}
    }
}
