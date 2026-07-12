use crate::{CoreDecisionBranch, CoreExpr, CoreModule, CoreStatement, CoreType};
use std::collections::BTreeSet;

pub(super) fn referenced_value_symbols(module: &CoreModule) -> BTreeSet<String> {
    let mut values = BTreeSet::new();
    for binding in &module.bindings {
        collect_expr_value_symbols(&binding.value, &mut values);
    }
    for function in &module.functions {
        collect_expr_value_symbols(&function.body, &mut values);
    }
    values
}

fn collect_expr_value_symbols(expr: &CoreExpr, values: &mut BTreeSet<String>) {
    match expr {
        CoreExpr::Variable { name, .. } => {
            values.insert(name.clone());
        }
        CoreExpr::Call {
            callee, arguments, ..
        } => {
            values.insert(callee.clone());
            for argument in arguments {
                collect_expr_value_symbols(argument, values);
            }
        }
        CoreExpr::Tuple { elements, .. } => {
            for element in elements {
                collect_expr_value_symbols(element, values);
            }
        }
        CoreExpr::Binary { left, right, .. } => {
            collect_expr_value_symbols(left, values);
            collect_expr_value_symbols(right, values);
        }
        CoreExpr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            collect_expr_value_symbols(condition, values);
            collect_expr_value_symbols(then_branch, values);
            collect_expr_value_symbols(else_branch, values);
        }
        CoreExpr::Decision {
            scrutinee,
            branches,
            ..
        } => {
            collect_expr_value_symbols(scrutinee, values);
            for branch in branches {
                collect_branch_value_symbols(branch, values);
            }
        }
        CoreExpr::EffectOperation { arguments, .. } => {
            for argument in arguments {
                collect_expr_value_symbols(argument, values);
            }
        }
        CoreExpr::EffectInvoke {
            callee, arguments, ..
        } => {
            values.insert(callee.clone());
            for argument in arguments {
                collect_expr_value_symbols(argument, values);
            }
        }
        CoreExpr::Sequence {
            statements, result, ..
        } => {
            for statement in statements {
                match statement {
                    CoreStatement::Effect { value }
                    | CoreStatement::PureLet { value, .. }
                    | CoreStatement::Bind { value, .. } => {
                        collect_expr_value_symbols(value, values);
                    }
                }
            }
            collect_expr_value_symbols(result, values);
        }
        CoreExpr::Unit { .. }
        | CoreExpr::Int64 { .. }
        | CoreExpr::String { .. }
        | CoreExpr::Boolean { .. } => {}
    }
}

fn collect_branch_value_symbols(branch: &CoreDecisionBranch, values: &mut BTreeSet<String>) {
    if let Some(guard) = &branch.guard {
        collect_expr_value_symbols(guard, values);
    }
    collect_expr_value_symbols(&branch.value, values);
}

pub(super) fn referenced_type_names(module: &CoreModule) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for adt in &module.adts {
        for variant in &adt.variants {
            if let Some(payload) = &variant.payload {
                collect_type_names(payload, &mut names);
            }
        }
    }
    for instance in &module.instances {
        collect_type_names(&instance.head, &mut names);
    }
    for binding in &module.bindings {
        collect_expr_type_names(&binding.value, &mut names);
    }
    for function in &module.functions {
        for parameter in &function.parameters {
            collect_type_names(&parameter.type_ref, &mut names);
        }
        collect_expr_type_names(&function.body, &mut names);
    }
    names
}

fn collect_expr_type_names(expr: &CoreExpr, names: &mut BTreeSet<String>) {
    match expr {
        CoreExpr::Variable { type_ref, .. } | CoreExpr::Call { type_ref, .. } => {
            collect_type_names(type_ref, names);
        }
        CoreExpr::Tuple {
            elements, type_ref, ..
        } => {
            collect_type_names(type_ref, names);
            for element in elements {
                collect_expr_type_names(element, names);
            }
        }
        CoreExpr::Binary {
            left,
            right,
            type_ref,
            ..
        } => {
            collect_type_names(type_ref, names);
            collect_expr_type_names(left, names);
            collect_expr_type_names(right, names);
        }
        CoreExpr::If {
            condition,
            then_branch,
            else_branch,
            type_ref,
            ..
        } => {
            collect_type_names(type_ref, names);
            collect_expr_type_names(condition, names);
            collect_expr_type_names(then_branch, names);
            collect_expr_type_names(else_branch, names);
        }
        CoreExpr::Decision {
            scrutinee,
            scrutinee_type,
            branches,
            type_ref,
            ..
        } => {
            collect_type_names(scrutinee_type, names);
            collect_type_names(type_ref, names);
            collect_expr_type_names(scrutinee, names);
            for branch in branches {
                for binding in &branch.bindings {
                    collect_type_names(&binding.type_ref, names);
                }
                if let Some(guard) = &branch.guard {
                    collect_expr_type_names(guard, names);
                }
                collect_expr_type_names(&branch.value, names);
            }
        }
        CoreExpr::EffectOperation {
            requirements,
            failure,
            success,
            arguments,
            ..
        }
        | CoreExpr::EffectInvoke {
            requirements,
            failure,
            success,
            arguments,
            ..
        } => {
            collect_type_names(requirements, names);
            collect_type_names(failure, names);
            collect_type_names(success, names);
            for argument in arguments {
                collect_expr_type_names(argument, names);
            }
        }
        CoreExpr::Sequence {
            statements, result, ..
        } => {
            for statement in statements {
                match statement {
                    CoreStatement::Effect { value } => collect_expr_type_names(value, names),
                    CoreStatement::PureLet {
                        type_ref, value, ..
                    }
                    | CoreStatement::Bind {
                        type_ref, value, ..
                    } => {
                        collect_type_names(type_ref, names);
                        collect_expr_type_names(value, names);
                    }
                }
            }
            collect_expr_type_names(result, names);
        }
        CoreExpr::Unit { .. }
        | CoreExpr::Int64 { .. }
        | CoreExpr::String { .. }
        | CoreExpr::Boolean { .. } => {}
    }
}

fn collect_type_names(type_ref: &CoreType, names: &mut BTreeSet<String>) {
    match type_ref {
        CoreType::Named { name, arguments } => {
            names.insert(name.clone());
            for argument in arguments {
                collect_type_names(argument, names);
            }
        }
        CoreType::Record { fields, .. } => {
            for field in fields {
                collect_type_names(&field.type_ref, names);
            }
        }
        CoreType::Tuple { elements } => {
            for element in elements {
                collect_type_names(element, names);
            }
        }
        CoreType::Function { parameter, result } => {
            collect_type_names(parameter, names);
            collect_type_names(result, names);
        }
        CoreType::Hole => {}
    }
}
