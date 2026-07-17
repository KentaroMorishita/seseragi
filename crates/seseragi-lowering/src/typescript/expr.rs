use crate::collection_ops::runtime_collection_operation;
use crate::effect_ops::runtime_effect_operation;
use crate::int_ops::runtime_int_operation_with_evidence;
use crate::iterator_ops::runtime_iterator_operation;
use crate::list_ops::runtime_list_literal_operation;
use crate::range_ops::runtime_range_operation;
use crate::sum_ops::runtime_sum_constructor;
use crate::{
    CoreExpr, CoreMonadDoStatement, CoreRecordValueItem, CoreStatement, CoreTemplatePart, CoreType,
};
use std::collections::BTreeMap;

use super::decision::lower_core_decision;
use super::dictionaries::local_dictionary_expression;
use super::names::{local_name, safe_identifier};
use super::types::{
    render_typescript_type, type_ref_from_core_type, type_ref_from_core_type_with_erasure,
};
use super::{TypeScriptExpr, TypeScriptRecordValueItem, TypeScriptStatement};

mod comprehension;

use comprehension::lower_array_comprehension;

pub(super) fn lower_core_expr_to_typescript(
    expr: CoreExpr,
    imported_values: &BTreeMap<String, String>,
    imported_types: &BTreeMap<String, String>,
) -> TypeScriptExpr {
    match expr {
        CoreExpr::Unit { .. } => TypeScriptExpr::Undefined,
        CoreExpr::Int64 { value, .. } => TypeScriptExpr::Bigint { value },
        CoreExpr::String { value, .. } => TypeScriptExpr::String { value },
        CoreExpr::Template { parts, .. } => lower_template(parts, imported_values, imported_types),
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
                if let Some(method) = operator_trait_method(&name) {
                    if let Some(dictionary) = evidence.first().and_then(|selected| {
                        local_dictionary_expression(
                            &selected.evidence,
                            imported_values,
                            imported_types,
                        )
                    }) {
                        return curried_dictionary_method_reference(dictionary, method);
                    }
                    if name == "+"
                        && matches!(
                            evidence.first().map(|selected| &selected.evidence),
                            Some(crate::CoreInstanceEvidence::Standard { identity })
                                if identity == "std/string::Add"
                        )
                    {
                        return curried_binary_reference("+");
                    }
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
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
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
            } else if callee == "std/prelude::reduce"
                && matches!(evidence.as_slice(), [selected] if selected.constraint.name == "Reducible")
            {
                let dictionary = local_dictionary_expression(
                    &evidence[0].evidence,
                    imported_values,
                    imported_types,
                )
                .expect("user Reducible call requires materialized dictionary evidence");
                TypeScriptExpr::DictionaryCall {
                    dictionary: Box::new(dictionary),
                    method: "reduce".to_owned(),
                    arguments,
                }
            } else if let Some(operation) = runtime_iterator_operation(&callee) {
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
                let evidence = evidence
                    .iter()
                    .map(|selected| {
                        local_dictionary_expression(
                            &selected.evidence,
                            imported_values,
                            imported_types,
                        )
                        .expect("constrained function call requires materializable evidence")
                    })
                    .collect::<Vec<_>>();
                let callee = imported_values
                    .get(&callee)
                    .cloned()
                    .unwrap_or_else(|| local_name(&callee));
                lower_constrained_call(
                    callee,
                    arguments,
                    evidence,
                    deferred_evidence_parameters,
                    deferred_evidence_type_constructor_parameters,
                    imported_types,
                )
            }
        }
        CoreExpr::Tuple { elements, .. } => TypeScriptExpr::Tuple {
            elements: lower_core_expressions(elements, imported_values, imported_types),
        },
        CoreExpr::FieldAccess {
            receiver, field, ..
        } => TypeScriptExpr::FieldAccess {
            receiver: Box::new(lower_core_expr_to_typescript(
                *receiver,
                imported_values,
                imported_types,
            )),
            field,
        },
        CoreExpr::OptionalFieldAccess {
            receiver, field, ..
        } => TypeScriptExpr::OptionalFieldAccess {
            receiver: Box::new(lower_core_expr_to_typescript(
                *receiver,
                imported_values,
                imported_types,
            )),
            field,
            just_constructor: runtime_sum_constructor("std/prelude::Just")
                .expect("standard Just constructor must exist")
                .local_name
                .to_owned(),
            nothing_constructor: runtime_sum_constructor("std/prelude::Nothing")
                .expect("standard Nothing constructor must exist")
                .local_name
                .to_owned(),
        },
        CoreExpr::Record { items, .. } => TypeScriptExpr::Record {
            items: items
                .into_iter()
                .map(|item| match item {
                    CoreRecordValueItem::Field { name, value, .. } => {
                        TypeScriptRecordValueItem::Field {
                            name,
                            value: lower_core_expr_to_typescript(
                                value,
                                imported_values,
                                imported_types,
                            ),
                        }
                    }
                    CoreRecordValueItem::Spread { value, .. } => {
                        TypeScriptRecordValueItem::Spread {
                            value: lower_core_expr_to_typescript(
                                value,
                                imported_values,
                                imported_types,
                            ),
                        }
                    }
                })
                .collect(),
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
        CoreExpr::List {
            elements, type_ref, ..
        } => {
            let element_type = match type_ref_from_core_type(&type_ref, imported_types) {
                super::TypeScriptType::List { element } => *element,
                _ => super::TypeScriptType::Unknown,
            };
            let operation = runtime_list_literal_operation();
            TypeScriptExpr::RuntimeCall {
                callee: operation.local_name.to_owned(),
                arguments: vec![TypeScriptExpr::Array {
                    elements: lower_core_expressions(elements, imported_values, imported_types),
                    element_type,
                }],
            }
        }
        CoreExpr::ArrayComprehension {
            element, clauses, ..
        } => lower_array_comprehension(*element, clauses, imported_values, imported_types, 0),
        CoreExpr::ListComprehension {
            element, clauses, ..
        } => {
            let operation = runtime_list_literal_operation();
            TypeScriptExpr::RuntimeCall {
                callee: operation.local_name.to_owned(),
                arguments: vec![lower_array_comprehension(
                    *element,
                    clauses,
                    imported_values,
                    imported_types,
                    0,
                )],
            }
        }
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
        CoreExpr::MonadDo {
            statements,
            result,
            evidence,
            ..
        } => TypeScriptExpr::MonadDo {
            dictionary: Box::new(
                local_dictionary_expression(&evidence.evidence, imported_values, imported_types)
                    .expect("monad do requires materialized Monad evidence"),
            ),
            statements: statements
                .into_iter()
                .map(|statement| {
                    lower_monad_do_statement(statement, imported_values, imported_types)
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

fn lower_template(
    parts: Vec<CoreTemplatePart>,
    imported_values: &BTreeMap<String, String>,
    imported_types: &BTreeMap<String, String>,
) -> TypeScriptExpr {
    parts
        .into_iter()
        .map(|part| match part {
            CoreTemplatePart::Text { value, .. } => TypeScriptExpr::String { value },
            CoreTemplatePart::Interpolation {
                value, evidence, ..
            } => {
                let evidence = evidence.expect(
                    "template interpolation must select Show evidence before TypeScript lowering",
                );
                let dictionary = local_dictionary_expression(
                    &evidence.evidence,
                    imported_values,
                    imported_types,
                )
                .expect("template interpolation requires materializable Show evidence");
                TypeScriptExpr::DictionaryCall {
                    dictionary: Box::new(dictionary),
                    method: "show".to_owned(),
                    arguments: vec![lower_core_expr_to_typescript(
                        value,
                        imported_values,
                        imported_types,
                    )],
                }
            }
        })
        .reduce(|left, right| TypeScriptExpr::Binary {
            operator: "+".to_owned(),
            left: Box::new(left),
            right: Box::new(right),
        })
        .unwrap_or(TypeScriptExpr::String {
            value: String::new(),
        })
}

fn lower_constrained_call(
    callee: String,
    mut arguments: Vec<TypeScriptExpr>,
    evidence: Vec<TypeScriptExpr>,
    deferred_parameters: Vec<CoreType>,
    deferred_type_constructor_parameters: Vec<String>,
    imported_types: &BTreeMap<String, String>,
) -> TypeScriptExpr {
    if deferred_parameters.is_empty() {
        arguments.extend(evidence);
        return TypeScriptExpr::Call { callee, arguments };
    }

    let parameters = deferred_parameters
        .into_iter()
        .enumerate()
        .map(|(index, type_ref)| {
            let name = format!("__ssrg$partial${index}");
            let type_name = render_typescript_type(&type_ref_from_core_type_with_erasure(
                &type_ref,
                imported_types,
                &deferred_type_constructor_parameters,
            ));
            arguments.push(TypeScriptExpr::Identifier { name: name.clone() });
            (name, type_name)
        })
        .collect::<Vec<_>>();
    arguments.extend(evidence);

    parameters.into_iter().rev().fold(
        TypeScriptExpr::Call { callee, arguments },
        |body, (name, type_name)| TypeScriptExpr::Lambda {
            parameter: format!("{name}: {type_name}"),
            body: Box::new(body),
        },
    )
}

pub(super) fn typescript_expr_contains_await(expr: &TypeScriptExpr) -> bool {
    match expr {
        TypeScriptExpr::Await { .. } => true,
        TypeScriptExpr::Tuple { elements } | TypeScriptExpr::Array { elements, .. } => {
            elements.iter().any(typescript_expr_contains_await)
        }
        TypeScriptExpr::FieldAccess { receiver, .. }
        | TypeScriptExpr::OptionalFieldAccess { receiver, .. } => {
            typescript_expr_contains_await(receiver)
        }
        TypeScriptExpr::Record { items } => items
            .iter()
            .any(|item| typescript_expr_contains_await(item.value())),
        TypeScriptExpr::Lambda { body, .. } => typescript_expr_contains_await(body),
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
        TypeScriptExpr::MonadDo {
            dictionary,
            statements,
            result,
        } => {
            typescript_expr_contains_await(dictionary)
                || statements.iter().any(statement_contains_await)
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
    if let Some(operation) = runtime_range_operation(&operator) {
        return TypeScriptExpr::RuntimeCall {
            callee: operation.local_name.to_owned(),
            arguments: vec![left, right],
        };
    }
    if is_int_type(&type_ref) {
        if let Some(operation) = runtime_int_operation_with_evidence(&operator, &evidence) {
            return TypeScriptExpr::RuntimeCall {
                callee: operation.local_name.to_owned(),
                arguments: vec![left, right],
            };
        }
    }
    if matches!(operator.as_str(), "==" | "!=") {
        if let Some(selected) = evidence.first().and_then(|selected| {
            local_dictionary_expression(&selected.evidence, imported_values, imported_types)
        }) {
            let equals = TypeScriptExpr::DictionaryCall {
                dictionary: Box::new(selected),
                method: "eq".to_owned(),
                arguments: vec![left, right],
            };
            return if operator == "==" {
                equals
            } else {
                TypeScriptExpr::Binary {
                    operator: "===".to_owned(),
                    left: Box::new(equals),
                    right: Box::new(TypeScriptExpr::Boolean { value: false }),
                }
            };
        }
    }
    if let Some(method) = operator_trait_method(&operator) {
        if let Some(selected) = evidence.first().and_then(|selected| {
            local_dictionary_expression(&selected.evidence, imported_values, imported_types)
        }) {
            return TypeScriptExpr::DictionaryCall {
                dictionary: Box::new(selected),
                method: method.to_owned(),
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

fn operator_trait_method(operator: &str) -> Option<&'static str> {
    match operator {
        "+" => Some("add"),
        "-" => Some("sub"),
        "*" => Some("mul"),
        "/" => Some("div"),
        "%" => Some("rem"),
        "**" => Some("pow"),
        _ => None,
    }
}

fn curried_dictionary_method_reference(dictionary: TypeScriptExpr, method: &str) -> TypeScriptExpr {
    let left = "_argument0".to_owned();
    let right = "_argument1".to_owned();
    TypeScriptExpr::Lambda {
        parameter: left.clone(),
        body: Box::new(TypeScriptExpr::Lambda {
            parameter: right.clone(),
            body: Box::new(TypeScriptExpr::DictionaryCall {
                dictionary: Box::new(dictionary),
                method: method.to_owned(),
                arguments: vec![
                    TypeScriptExpr::Identifier { name: left },
                    TypeScriptExpr::Identifier { name: right },
                ],
            }),
        }),
    }
}

fn curried_binary_reference(operator: &str) -> TypeScriptExpr {
    let left = "_argument0".to_owned();
    let right = "_argument1".to_owned();
    TypeScriptExpr::Lambda {
        parameter: left.clone(),
        body: Box::new(TypeScriptExpr::Lambda {
            parameter: right.clone(),
            body: Box::new(TypeScriptExpr::Binary {
                operator: operator.to_owned(),
                left: Box::new(TypeScriptExpr::Identifier { name: left }),
                right: Box::new(TypeScriptExpr::Identifier { name: right }),
            }),
        }),
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

fn lower_monad_do_statement(
    statement: CoreMonadDoStatement,
    imported_values: &BTreeMap<String, String>,
    imported_types: &BTreeMap<String, String>,
) -> TypeScriptStatement {
    match statement {
        CoreMonadDoStatement::Expression { value } => TypeScriptStatement::Effect {
            value: lower_core_expr_to_typescript(value, imported_values, imported_types),
        },
        CoreMonadDoStatement::PureLet {
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
        CoreMonadDoStatement::Bind {
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
