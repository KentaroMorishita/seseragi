use crate::effect_ops::runtime_effect_operation;
use crate::int_ops::runtime_int_operation;
use crate::sum_ops::runtime_sum_constructor;
use crate::{CoreExpr, CoreStatement, CoreType};
use std::collections::BTreeMap;

use super::decision::lower_core_decision;
use super::names::{local_name, safe_identifier};
use super::types::type_ref_from_core_type;
use super::{TypeScriptExpr, TypeScriptStatement};

pub(super) fn lower_core_expr_to_typescript(
    expr: CoreExpr,
    imported_values: &BTreeMap<String, String>,
) -> TypeScriptExpr {
    match expr {
        CoreExpr::Unit { .. } => TypeScriptExpr::Undefined,
        CoreExpr::Int64 { value, .. } => TypeScriptExpr::Bigint { value },
        CoreExpr::String { value, .. } => TypeScriptExpr::String { value },
        CoreExpr::Boolean { value, .. } => TypeScriptExpr::Boolean { value },
        CoreExpr::Variable { name, .. } => runtime_sum_constructor(&name)
            .map(|constructor| TypeScriptExpr::RuntimeReference {
                name: constructor.local_name.to_owned(),
            })
            .unwrap_or_else(|| TypeScriptExpr::Identifier {
                name: imported_values
                    .get(&name)
                    .cloned()
                    .unwrap_or_else(|| local_name(&name)),
            }),
        CoreExpr::Call {
            callee, arguments, ..
        } => {
            let arguments = lower_core_expressions(arguments, imported_values);
            if let Some(constructor) = runtime_sum_constructor(&callee) {
                TypeScriptExpr::RuntimeCall {
                    callee: constructor.local_name.to_owned(),
                    arguments,
                }
            } else {
                TypeScriptExpr::Call {
                    callee: imported_values
                        .get(&callee)
                        .cloned()
                        .unwrap_or_else(|| local_name(&callee)),
                    arguments,
                }
            }
        }
        CoreExpr::Tuple { elements, .. } => TypeScriptExpr::Tuple {
            elements: lower_core_expressions(elements, imported_values),
        },
        CoreExpr::Binary {
            operator,
            left,
            right,
            type_ref,
            ..
        } => lower_binary(operator, *left, *right, type_ref, imported_values),
        CoreExpr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => TypeScriptExpr::Conditional {
            condition: Box::new(lower_core_expr_to_typescript(*condition, imported_values)),
            then_branch: Box::new(lower_core_expr_to_typescript(*then_branch, imported_values)),
            else_branch: Box::new(lower_core_expr_to_typescript(*else_branch, imported_values)),
        },
        CoreExpr::Decision {
            scrutinee,
            scrutinee_type,
            branches,
            type_ref,
            ..
        } => lower_core_decision(
            *scrutinee,
            scrutinee_type,
            branches,
            type_ref,
            imported_values,
        ),
        CoreExpr::EffectOperation {
            operation,
            arguments,
            ..
        } => lower_effect_operation(operation, arguments, imported_values),
        CoreExpr::Sequence {
            statements, result, ..
        } => TypeScriptExpr::Sequence {
            statements: statements
                .into_iter()
                .map(|statement| lower_core_statement_to_typescript(statement, imported_values))
                .collect(),
            result: Box::new(lower_core_expr_to_typescript(*result, imported_values)),
        },
    }
}

pub(super) fn typescript_expr_contains_await(expr: &TypeScriptExpr) -> bool {
    match expr {
        TypeScriptExpr::Await { .. } => true,
        TypeScriptExpr::Tuple { elements } => elements.iter().any(typescript_expr_contains_await),
        TypeScriptExpr::Binary { left, right, .. } => {
            typescript_expr_contains_await(left) || typescript_expr_contains_await(right)
        }
        TypeScriptExpr::Conditional {
            condition,
            then_branch,
            else_branch,
        } => {
            typescript_expr_contains_await(condition)
                || typescript_expr_contains_await(then_branch)
                || typescript_expr_contains_await(else_branch)
        }
        TypeScriptExpr::Decision {
            scrutinee,
            branches,
            ..
        } => {
            typescript_expr_contains_await(scrutinee)
                || branches.iter().any(|branch| {
                    branch
                        .guard
                        .as_ref()
                        .is_some_and(typescript_expr_contains_await)
                        || typescript_expr_contains_await(&branch.value)
                })
        }
        TypeScriptExpr::Call { arguments, .. } | TypeScriptExpr::RuntimeCall { arguments, .. } => {
            arguments.iter().any(typescript_expr_contains_await)
        }
        TypeScriptExpr::Sequence { statements, result } => {
            statements.iter().any(statement_contains_await)
                || typescript_expr_contains_await(result)
        }
        TypeScriptExpr::Undefined
        | TypeScriptExpr::Bigint { .. }
        | TypeScriptExpr::String { .. }
        | TypeScriptExpr::Boolean { .. }
        | TypeScriptExpr::Identifier { .. }
        | TypeScriptExpr::RuntimeReference { .. } => false,
    }
}

fn lower_core_expressions(
    expressions: Vec<CoreExpr>,
    imported_values: &BTreeMap<String, String>,
) -> Vec<TypeScriptExpr> {
    expressions
        .into_iter()
        .map(|expression| lower_core_expr_to_typescript(expression, imported_values))
        .collect()
}

fn lower_binary(
    operator: String,
    left: CoreExpr,
    right: CoreExpr,
    type_ref: CoreType,
    imported_values: &BTreeMap<String, String>,
) -> TypeScriptExpr {
    let left = lower_core_expr_to_typescript(left, imported_values);
    let right = lower_core_expr_to_typescript(right, imported_values);
    if is_int_type(&type_ref) {
        if let Some(operation) = runtime_int_operation(&operator) {
            return TypeScriptExpr::RuntimeCall {
                callee: operation.local_name.to_owned(),
                arguments: vec![left, right],
            };
        }
    }
    TypeScriptExpr::Binary {
        operator: typescript_binary_operator(&operator).to_owned(),
        left: Box::new(left),
        right: Box::new(right),
    }
}

fn lower_effect_operation(
    operation: String,
    arguments: Vec<CoreExpr>,
    imported_values: &BTreeMap<String, String>,
) -> TypeScriptExpr {
    let runtime_operation = runtime_effect_operation(&operation);
    let mut arguments = lower_core_expressions(arguments, imported_values);
    if operation == "effect.succeed" && arguments.is_empty() {
        arguments.push(TypeScriptExpr::Undefined);
    }
    TypeScriptExpr::RuntimeCall {
        callee: runtime_operation
            .map(|operation| operation.local_name.to_owned())
            .unwrap_or_else(|| safe_identifier(&operation)),
        arguments,
    }
}

fn lower_core_statement_to_typescript(
    statement: CoreStatement,
    imported_values: &BTreeMap<String, String>,
) -> TypeScriptStatement {
    match statement {
        CoreStatement::Effect { value } => TypeScriptStatement::Effect {
            value: lower_core_expr_to_typescript(value, imported_values),
        },
        CoreStatement::PureLet {
            name,
            type_ref,
            value,
            origin,
        } => TypeScriptStatement::PureLet {
            name: safe_identifier(&name),
            type_ref: type_ref_from_core_type(&type_ref),
            initializer: lower_core_expr_to_typescript(value, imported_values),
            origin,
        },
        CoreStatement::Bind {
            name,
            type_ref,
            value,
            origin,
        } => TypeScriptStatement::Const {
            name: safe_identifier(&name),
            type_ref: type_ref_from_core_type(&type_ref),
            initializer: lower_core_expr_to_typescript(value, imported_values),
            origin,
        },
    }
}

fn typescript_binary_operator(operator: &str) -> &str {
    match operator {
        "==" => "===",
        "!=" => "!==",
        _ => operator,
    }
}

fn is_int_type(type_ref: &CoreType) -> bool {
    matches!(type_ref, CoreType::Named { name, arguments } if name == "Int" && arguments.is_empty())
}

fn statement_contains_await(statement: &TypeScriptStatement) -> bool {
    match statement {
        TypeScriptStatement::Effect { value } => typescript_expr_contains_await(value),
        TypeScriptStatement::PureLet { initializer, .. }
        | TypeScriptStatement::Const { initializer, .. } => {
            typescript_expr_contains_await(initializer)
        }
    }
}
