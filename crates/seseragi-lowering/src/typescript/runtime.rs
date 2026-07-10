use crate::{
    effect_ops::runtime_effect_operation, CoreExpr, CoreParameter, CoreStatement, CoreType,
};

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
        CoreExpr::Variable { type_ref, .. } => {
            collect_type_runtime_requirement(type_ref, requirements);
        }
        CoreExpr::Call {
            arguments,
            type_ref,
            ..
        } => {
            // A normal Call is not a runtime operation. Its type and its
            // argument expressions can still require core representations.
            collect_type_runtime_requirement(type_ref, requirements);
            for argument in arguments {
                collect_expr_runtime_requirements(argument, requirements);
            }
        }
        CoreExpr::Binary {
            left,
            right,
            type_ref,
            ..
        } => {
            collect_type_runtime_requirement(type_ref, requirements);
            collect_expr_runtime_requirements(left, requirements);
            collect_expr_runtime_requirements(right, requirements);
        }
        CoreExpr::If {
            condition,
            then_branch,
            else_branch,
            type_ref,
            ..
        } => {
            collect_type_runtime_requirement(type_ref, requirements);
            collect_expr_runtime_requirements(condition, requirements);
            collect_expr_runtime_requirements(then_branch, requirements);
            collect_expr_runtime_requirements(else_branch, requirements);
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
        CoreExpr::Call { arguments, .. } => {
            // Calls to user functions are emitted as local TypeScript calls;
            // only nested effect operations contribute runtime imports.
            for argument in arguments {
                collect_expr_runtime_imports(argument, imports);
            }
        }
        CoreExpr::Binary { left, right, .. } => {
            collect_expr_runtime_imports(left, imports);
            collect_expr_runtime_imports(right, imports);
        }
        CoreExpr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            collect_expr_runtime_imports(condition, imports);
            collect_expr_runtime_imports(then_branch, imports);
            collect_expr_runtime_imports(else_branch, imports);
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
        CoreExpr::Variable { type_ref, .. } => type_ref_from_core_type(type_ref),
        CoreExpr::Call { type_ref, .. } => type_ref_from_core_type(type_ref),
        CoreExpr::Binary { type_ref, .. } => type_ref_from_core_type(type_ref),
        CoreExpr::If { type_ref, .. } => type_ref_from_core_type(type_ref),
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
        CoreType::Hole
        | CoreType::Named { .. }
        | CoreType::Record { .. }
        | CoreType::Tuple { .. }
        | CoreType::Function { .. } => TypeScriptType::Unknown,
    }
}

pub(super) fn render_core_type(type_ref: &CoreType) -> String {
    match type_ref_from_core_type(type_ref) {
        TypeScriptType::Bigint => "bigint".to_owned(),
        TypeScriptType::Boolean => "boolean".to_owned(),
        TypeScriptType::String => "string".to_owned(),
        TypeScriptType::Undefined => "undefined".to_owned(),
        TypeScriptType::Unknown => "unknown".to_owned(),
        TypeScriptType::Maybe { element } => {
            format!("{} | undefined", render_typescript_type(&element))
        }
    }
}

fn render_typescript_type(type_ref: &TypeScriptType) -> String {
    match type_ref {
        TypeScriptType::Bigint => "bigint".to_owned(),
        TypeScriptType::Boolean => "boolean".to_owned(),
        TypeScriptType::String => "string".to_owned(),
        TypeScriptType::Undefined => "undefined".to_owned(),
        TypeScriptType::Unknown => "unknown".to_owned(),
        TypeScriptType::Maybe { element } => {
            format!("{} | undefined", render_typescript_type(element))
        }
    }
}

pub(super) fn collect_type_runtime_requirement(
    type_ref: &CoreType,
    requirements: &mut Vec<String>,
) {
    match type_ref {
        CoreType::Named { name, arguments } => {
            match name.as_str() {
                "Int" => push_unique(requirements, "core.int64"),
                "String" => push_unique(requirements, "core.string"),
                "Bool" => push_unique(requirements, "core.bool"),
                "Unit" => push_unique(requirements, "core.unit"),
                _ => {}
            }
            for argument in arguments {
                collect_type_runtime_requirement(argument, requirements);
            }
        }
        CoreType::Record { fields, .. } => {
            for field in fields {
                collect_type_runtime_requirement(&field.type_ref, requirements);
            }
        }
        CoreType::Tuple { elements } => {
            for element in elements {
                collect_type_runtime_requirement(element, requirements);
            }
        }
        CoreType::Function { parameter, result } => {
            collect_type_runtime_requirement(parameter, requirements);
            collect_type_runtime_requirement(result, requirements);
        }
        CoreType::Hole => {}
    }
}
