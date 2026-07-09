use crate::{effect_ops::runtime_effect_operation, CoreExpr, CoreParameter, CoreStatement};

use super::names::safe_identifier;
use super::{
    push_import_unique, push_unique, TypeScriptImport, TypeScriptParameter, TypeScriptType,
};

pub(super) fn collect_expr_runtime_requirements(expr: &CoreExpr, requirements: &mut Vec<String>) {
    match expr {
        CoreExpr::Unit { .. } => push_unique(requirements, "core.unit"),
        CoreExpr::Int64 { .. } => push_unique(requirements, "core.int64"),
        CoreExpr::String { .. } => push_unique(requirements, "core.string"),
        CoreExpr::Boolean { .. } => push_unique(requirements, "core.bool"),
        CoreExpr::Variable { type_name, .. } => {
            collect_type_runtime_requirement(type_name, requirements);
        }
        CoreExpr::Binary {
            left,
            right,
            type_name,
            ..
        } => {
            collect_type_runtime_requirement(type_name, requirements);
            collect_expr_runtime_requirements(left, requirements);
            collect_expr_runtime_requirements(right, requirements);
        }
        CoreExpr::EffectOperation {
            operation,
            arguments,
            ..
        } => {
            if let Some(operation) = runtime_effect_operation(operation) {
                push_unique(requirements, operation.runtime_feature);
            }
            for argument in arguments {
                collect_expr_runtime_requirements(argument, requirements);
            }
        }
        CoreExpr::Sequence {
            statements, result, ..
        } => {
            for statement in statements {
                collect_statement_runtime_requirements(statement, requirements);
            }
            collect_expr_runtime_requirements(result, requirements);
        }
    }
}

fn collect_statement_runtime_requirements(
    statement: &CoreStatement,
    requirements: &mut Vec<String>,
) {
    match statement {
        CoreStatement::Effect { value } | CoreStatement::Bind { value, .. } => {
            collect_expr_runtime_requirements(value, requirements);
        }
    }
}

pub(super) fn collect_expr_runtime_imports(expr: &CoreExpr, imports: &mut Vec<TypeScriptImport>) {
    match expr {
        CoreExpr::EffectOperation {
            operation,
            arguments,
            ..
        } => {
            if let Some(operation) = runtime_effect_operation(operation) {
                push_import_unique(
                    imports,
                    TypeScriptImport {
                        feature: operation.runtime_feature.to_owned(),
                        local: operation.local_name.to_owned(),
                    },
                );
            }
            for argument in arguments {
                collect_expr_runtime_imports(argument, imports);
            }
        }
        CoreExpr::Unit { .. }
        | CoreExpr::Int64 { .. }
        | CoreExpr::String { .. }
        | CoreExpr::Boolean { .. }
        | CoreExpr::Variable { .. } => {}
        CoreExpr::Binary { left, right, .. } => {
            collect_expr_runtime_imports(left, imports);
            collect_expr_runtime_imports(right, imports);
        }
        CoreExpr::Sequence {
            statements, result, ..
        } => {
            for statement in statements {
                collect_statement_runtime_imports(statement, imports);
            }
            collect_expr_runtime_imports(result, imports);
        }
    }
}

fn collect_statement_runtime_imports(
    statement: &CoreStatement,
    imports: &mut Vec<TypeScriptImport>,
) {
    match statement {
        CoreStatement::Effect { value } | CoreStatement::Bind { value, .. } => {
            collect_expr_runtime_imports(value, imports);
        }
    }
}

pub(super) fn type_ref_from_core_expr(expr: &CoreExpr) -> TypeScriptType {
    match expr {
        CoreExpr::Unit { .. } => TypeScriptType::Undefined,
        CoreExpr::Int64 { .. } => TypeScriptType::Bigint,
        CoreExpr::String { .. } => TypeScriptType::String,
        CoreExpr::Boolean { .. } => TypeScriptType::Boolean,
        CoreExpr::Variable { type_name, .. } => type_ref_from_type_name(type_name),
        CoreExpr::Binary { type_name, .. } => type_ref_from_type_name(type_name),
        CoreExpr::EffectOperation { .. } => TypeScriptType::Undefined,
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

pub(super) fn render_type_name(type_name: &str) -> &'static str {
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
