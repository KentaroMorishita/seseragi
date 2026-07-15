use crate::collection_ops::runtime_collection_operation;
use crate::effect_ops::runtime_effect_operation;
use crate::int_ops::runtime_int_operation_with_evidence;
use crate::sum_ops::runtime_sum_constructor;
use crate::{CoreExpr, CoreStatement, CoreType};
use std::collections::BTreeMap;

use super::decision::lower_core_decision;
use super::dictionaries::local_dictionary_expression;
use super::names::{local_name, safe_identifier};
use super::types::type_ref_from_core_type;
use super::{TypeScriptExpr, TypeScriptStatement};

pub(super) fn lower_core_expr_to_typescript(
    expr: CoreExpr,
    imported_values: &BTreeMap<String, String>,
    imported_types: &BTreeMap<String, String>,
) -> TypeScriptExpr {
    match expr {
        CoreExpr::Unit { .. } => TypeScriptExpr::Undefined,
        CoreExpr::Int64 { value, .. } => TypeScriptExpr::Bigint { value },
        CoreExpr::String { value, .. } => TypeScriptExpr::String { value },
        CoreExpr::Boolean { value, .. } => TypeScriptExpr::Boolean { value },
        CoreExpr::Variable {
            name,
            evidence,
            type_ref,
            ..
        } => {
            if matches!(type_ref, CoreType::Function { .. }) {
                if let Some(operation) = runtime_int_operation_with_evidence(&name, &evidence) {
                    return TypeScriptExpr::CurriedRuntimeReference {
                        name: operation.local_name.to_owned(),
                        arity: 2,
                    };
                }
            }
            runtime_sum_constructor(&name)
                .map(|constructor| TypeScriptExpr::RuntimeReference {
                    name: constructor.local_name.to_owned(),
                })
                .unwrap_or_else(|| TypeScriptExpr::Identifier {
                    name: imported_values
                        .get(&name)
                        .cloned()
                        .unwrap_or_else(|| local_name(&name)),
                })
        }
        CoreExpr::Call {
            callee,
            arguments,
            evidence,
            trait_dispatch,
            ..
        } => {
            let mut arguments = lower_core_expressions(arguments, imported_values, imported_types);
            if let Some(dispatch) = trait_dispatch {
                let (selected, method_evidence) = evidence
                    .split_first()
                    .expect("trait dispatch requires primary instance evidence");
                let selected = local_dictionary_expression(
                    &selected.evidence,
                    imported_values,
                    imported_types,
                )
                .expect("local trait dispatch requires materialized primary evidence");
                arguments.extend(method_evidence.iter().map(|selected| {
                    local_dictionary_expression(&selected.evidence, imported_values, imported_types)
                        .expect("trait method constraint requires materialized evidence")
                }));
                TypeScriptExpr::DictionaryCall {
                    dictionary: Box::new(selected),
                    method: dispatch.method,
                    arguments,
                }
            } else if let Some(operation) = runtime_collection_operation(&callee, &evidence) {
                TypeScriptExpr::RuntimeCall {
                    callee: operation.local_name.to_owned(),
                    arguments,
                }
            } else if let Some(constructor) = runtime_sum_constructor(&callee) {
                TypeScriptExpr::RuntimeCall {
                    callee: constructor.local_name.to_owned(),
                    arguments,
                }
            } else {
                arguments.extend(evidence.iter().map(|selected| {
                    local_dictionary_expression(&selected.evidence, imported_values, imported_types)
                        .expect("constrained function call requires materializable evidence")
                }));
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
            elements: lower_core_expressions(elements, imported_values, imported_types),
        },
        CoreExpr::Array {
            elements, type_ref, ..
        } => TypeScriptExpr::Array {
            elements: lower_core_expressions(elements, imported_values, imported_types),
            element_type: match type_ref_from_core_type(&type_ref, imported_types) {
                super::TypeScriptType::Array { element } => *element,
                _ => super::TypeScriptType::Unknown,
            },
        },
        CoreExpr::Binary {
            operator,
            left,
            right,
            evidence,
            type_ref,
            ..
        } => lower_binary(
            operator,
            *left,
            *right,
            evidence,
            type_ref,
            imported_values,
            imported_types,
        ),
        CoreExpr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => TypeScriptExpr::Conditional {
            condition: Box::new(lower_core_expr_to_typescript(
                *condition,
                imported_values,
                imported_types,
            )),
            then_branch: Box::new(lower_core_expr_to_typescript(
                *then_branch,
                imported_values,
                imported_types,
            )),
            else_branch: Box::new(lower_core_expr_to_typescript(
                *else_branch,
                imported_values,
                imported_types,
            )),
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
            imported_types,
        ),
        CoreExpr::EffectOperation {
            operation,
            arguments,
            ..
        } => lower_effect_operation(operation, arguments, imported_values, imported_types),
        CoreExpr::EffectInvoke {
            callee, arguments, ..
        } => TypeScriptExpr::Call {
            callee: imported_values
                .get(&callee)
                .cloned()
                .unwrap_or_else(|| local_name(&callee)),
            arguments: lower_core_expressions(arguments, imported_values, imported_types),
        },
        CoreExpr::Sequence {
            statements, result, ..
        } => TypeScriptExpr::Sequence {
            statements: statements
                .into_iter()
                .map(|statement| {
                    lower_core_statement_to_typescript(statement, imported_values, imported_types)
                })
                .collect(),
            result: Box::new(lower_core_expr_to_typescript(
                *result,
                imported_values,
                imported_types,
            )),
        },
    }
}

pub(super) fn typescript_expr_contains_await(expr: &TypeScriptExpr) -> bool {
    match expr {
        TypeScriptExpr::Await { .. } => true,
        TypeScriptExpr::Tuple { elements } | TypeScriptExpr::Array { elements, .. } => {
            elements.iter().any(typescript_expr_contains_await)
        }
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
        TypeScriptExpr::Call { arguments, .. }
        | TypeScriptExpr::TypeApplicationCall { arguments, .. }
        | TypeScriptExpr::RuntimeCall { arguments, .. } => {
            arguments.iter().any(typescript_expr_contains_await)
        }
        TypeScriptExpr::DictionaryCall {
            dictionary,
            arguments,
            ..
        } => {
            typescript_expr_contains_await(dictionary)
                || arguments.iter().any(typescript_expr_contains_await)
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
        | TypeScriptExpr::RuntimeReference { .. }
        | TypeScriptExpr::CurriedRuntimeReference { .. } => false,
    }
}

fn lower_core_expressions(
    expressions: Vec<CoreExpr>,
    imported_values: &BTreeMap<String, String>,
    imported_types: &BTreeMap<String, String>,
) -> Vec<TypeScriptExpr> {
    expressions
        .into_iter()
        .map(|expression| {
            lower_core_expr_to_typescript(expression, imported_values, imported_types)
        })
        .collect()
}

fn lower_binary(
    operator: String,
    left: CoreExpr,
    right: CoreExpr,
    evidence: Vec<crate::CoreCallEvidence>,
    type_ref: CoreType,
    imported_values: &BTreeMap<String, String>,
    imported_types: &BTreeMap<String, String>,
) -> TypeScriptExpr {
    let left = lower_core_expr_to_typescript(left, imported_values, imported_types);
    let right = lower_core_expr_to_typescript(right, imported_values, imported_types);
    if is_int_type(&type_ref) {
        if let Some(operation) = runtime_int_operation_with_evidence(&operator, &evidence) {
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
    imported_types: &BTreeMap<String, String>,
) -> TypeScriptExpr {
    let runtime_operation = runtime_effect_operation(&operation);
    let mut arguments = lower_core_expressions(arguments, imported_values, imported_types);
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
    imported_types: &BTreeMap<String, String>,
) -> TypeScriptStatement {
    match statement {
        CoreStatement::Effect { value } => TypeScriptStatement::Effect {
            value: lower_core_expr_to_typescript(value, imported_values, imported_types),
        },
        CoreStatement::PureLet {
            name,
            type_ref,
            value,
            origin,
        } => TypeScriptStatement::PureLet {
            name: safe_identifier(&name),
            type_ref: type_ref_from_core_type(&type_ref, imported_types),
            initializer: lower_core_expr_to_typescript(value, imported_values, imported_types),
            origin,
        },
        CoreStatement::Bind {
            name,
            type_ref,
            value,
            origin,
        } => TypeScriptStatement::Const {
            name: safe_identifier(&name),
            type_ref: type_ref_from_core_type(&type_ref, imported_types),
            initializer: lower_core_expr_to_typescript(value, imported_values, imported_types),
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
