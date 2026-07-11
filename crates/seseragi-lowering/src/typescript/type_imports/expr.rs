use crate::{CoreExpr, CoreStatement};
use seseragi_semantics::ExternalTypeBinding;

use super::super::TypeScriptTypeImport;
use super::collect_type_imports;

pub(super) fn collect_expr_type_imports(
    expr: &CoreExpr,
    bindings: &[ExternalTypeBinding],
    requirements: &mut Vec<String>,
    imports: &mut Vec<TypeScriptTypeImport>,
) {
    match expr {
        CoreExpr::Unit { .. }
        | CoreExpr::Int64 { .. }
        | CoreExpr::String { .. }
        | CoreExpr::Boolean { .. } => {}
        CoreExpr::Variable { type_ref, .. } => {
            collect_type_imports(type_ref, bindings, requirements, imports);
        }
        CoreExpr::Call {
            arguments,
            type_ref,
            ..
        } => {
            collect_type_imports(type_ref, bindings, requirements, imports);
            collect_exprs(arguments, bindings, requirements, imports);
        }
        CoreExpr::Tuple {
            elements, type_ref, ..
        } => {
            collect_type_imports(type_ref, bindings, requirements, imports);
            collect_exprs(elements, bindings, requirements, imports);
        }
        CoreExpr::Binary {
            left,
            right,
            type_ref,
            ..
        } => {
            collect_type_imports(type_ref, bindings, requirements, imports);
            collect_expr_type_imports(left, bindings, requirements, imports);
            collect_expr_type_imports(right, bindings, requirements, imports);
        }
        CoreExpr::If {
            condition,
            then_branch,
            else_branch,
            type_ref,
            ..
        } => {
            collect_type_imports(type_ref, bindings, requirements, imports);
            collect_expr_type_imports(condition, bindings, requirements, imports);
            collect_expr_type_imports(then_branch, bindings, requirements, imports);
            collect_expr_type_imports(else_branch, bindings, requirements, imports);
        }
        CoreExpr::Decision {
            scrutinee,
            scrutinee_type,
            branches,
            type_ref,
            ..
        } => {
            collect_type_imports(scrutinee_type, bindings, requirements, imports);
            collect_type_imports(type_ref, bindings, requirements, imports);
            collect_expr_type_imports(scrutinee, bindings, requirements, imports);
            for branch in branches {
                for binding in &branch.bindings {
                    collect_type_imports(&binding.type_ref, bindings, requirements, imports);
                }
                if let Some(guard) = &branch.guard {
                    collect_expr_type_imports(guard, bindings, requirements, imports);
                }
                collect_expr_type_imports(&branch.value, bindings, requirements, imports);
            }
        }
        CoreExpr::EffectOperation {
            arguments, success, ..
        } => {
            collect_type_imports(success, bindings, requirements, imports);
            collect_exprs(arguments, bindings, requirements, imports);
        }
        CoreExpr::Sequence {
            statements, result, ..
        } => {
            for statement in statements {
                let value = match statement {
                    CoreStatement::Effect { value } => value,
                    CoreStatement::PureLet {
                        type_ref, value, ..
                    }
                    | CoreStatement::Bind {
                        type_ref, value, ..
                    } => {
                        collect_type_imports(type_ref, bindings, requirements, imports);
                        value
                    }
                };
                collect_expr_type_imports(value, bindings, requirements, imports);
            }
            collect_expr_type_imports(result, bindings, requirements, imports);
        }
    }
}

fn collect_exprs(
    expressions: &[CoreExpr],
    bindings: &[ExternalTypeBinding],
    requirements: &mut Vec<String>,
    imports: &mut Vec<TypeScriptTypeImport>,
) {
    for expression in expressions {
        collect_expr_type_imports(expression, bindings, requirements, imports);
    }
}
