use crate::collection_ops::runtime_iterable_operation;
use crate::iterator_ops::runtime_iterator_comprehension_operation;
use crate::typescript::types::TypeScriptTypeContext;
use crate::{CoreComprehensionClause, CoreExpr, CorePattern};
use std::collections::BTreeMap;

use super::lower_core_expr_to_typescript;
use crate::failure::{missing_evidence, unexpected_shape, Lowered};
use crate::typescript::decision::lower_core_pattern_decision;
use crate::typescript::dictionaries::local_dictionary_expression;
use crate::typescript::names::safe_identifier;
use crate::typescript::types::type_ref_from_core_expr;
use crate::typescript::{TypeScriptDecisionBranch, TypeScriptExpr, TypeScriptType};

pub(super) fn lower_array_comprehension(
    element: CoreExpr,
    clauses: Vec<CoreComprehensionClause>,
    imported_values: &BTreeMap<String, String>,
    imported_types: &TypeScriptTypeContext,
    depth: usize,
) -> Lowered<TypeScriptExpr> {
    let mut clauses = clauses.into_iter();
    let Some(CoreComprehensionClause::Generator {
        pattern,
        source,
        evidence,
        origin,
    }) = clauses.next()
    else {
        return Err(unexpected_shape(
            "typed comprehension must begin with a generator",
        ));
    };
    (|| {
        let direct_parameter = match &pattern {
            CorePattern::Binding { name, .. } => Some(safe_identifier(name)),
            CorePattern::Wildcard { .. } => Some(format!("$ssrg_item{depth}")),
            _ => None,
        };
        let remaining = clauses.collect::<Vec<_>>();
        let next_generator = remaining
            .iter()
            .position(|clause| matches!(clause, CoreComprehensionClause::Generator { .. }));
        let (guards, nested) = next_generator
            .map(|index| remaining.split_at(index))
            .unwrap_or((&remaining, &[]));
        let guard = guards
            .iter()
            .filter_map(|clause| match clause {
                CoreComprehensionClause::Guard { condition, .. } => {
                    Some(lower_core_expr_to_typescript(
                        condition.clone(),
                        imported_values,
                        imported_types,
                    ))
                }
                CoreComprehensionClause::Generator { .. } => None,
            })
            .collect::<Lowered<Vec<_>>>()?
            .into_iter()
            .reduce(|left, right| TypeScriptExpr::Binary {
                operator: "&&".to_owned(),
                left: Box::new(left),
                right: Box::new(right),
            });
        let flatten = !nested.is_empty();
        let element_type = type_ref_from_core_expr(&element, imported_types)?;
        let transform_type = if flatten {
            TypeScriptType::Array {
                element: Box::new(element_type.clone()),
            }
        } else {
            element_type
        };
        let transform = if flatten {
            lower_array_comprehension(
                element,
                nested.to_vec(),
                imported_values,
                imported_types,
                depth + 1,
            )?
        } else {
            lower_core_expr_to_typescript(element, imported_values, imported_types)?
        };
        let (parameter, predicate, transform) = match direct_parameter {
            Some(parameter) => (
                parameter,
                guard.unwrap_or(TypeScriptExpr::Boolean { value: true }),
                transform,
            ),
            None => {
                let pattern = lower_core_pattern_decision(pattern, imported_types)?;
                let parameter = format!("$ssrg_item{depth}");
                let predicate = pattern_predicate(&parameter, &pattern, guard);
                let transform = pattern_transform(&parameter, &pattern, transform, transform_type);
                (parameter, predicate, transform)
            }
        };
        let source = lower_core_expr_to_typescript(source, imported_values, imported_types)?;
        let (callee, source) = if let Some(operation) =
            runtime_iterable_operation(&evidence, flatten)
        {
            (operation.local_name, source)
        } else {
            let dictionary =
                local_dictionary_expression(&evidence.evidence, imported_values, imported_types)?
                    .ok_or_else(|| {
                    missing_evidence(
                        "user Iterable comprehension requires materialized dictionary evidence",
                    )
                })?;
            (
                runtime_iterator_comprehension_operation(flatten).local_name,
                TypeScriptExpr::DictionaryCall {
                    dictionary: Box::new(dictionary),
                    method: "iterate".to_owned(),
                    arguments: vec![source],
                },
            )
        };
        Ok(TypeScriptExpr::RuntimeCall {
            callee: callee.to_owned(),
            arguments: vec![
                source,
                TypeScriptExpr::Lambda {
                    parameter: parameter.clone(),
                    body: Box::new(predicate),
                },
                TypeScriptExpr::Lambda {
                    parameter,
                    body: Box::new(transform),
                },
            ],
        })
    })()
    .map_err(|error: crate::LoweringInternalError| error.at(&origin))
}

fn pattern_predicate(
    parameter: &str,
    pattern: &crate::typescript::decision::TypeScriptPatternDecision,
    guard: Option<TypeScriptExpr>,
) -> TypeScriptExpr {
    TypeScriptExpr::Decision {
        scrutinee: Box::new(pattern_scrutinee(parameter, pattern)),
        scrutinee_type: pattern.scrutinee_type.clone(),
        branches: vec![
            TypeScriptDecisionBranch {
                list_prefix: None,
                tests: pattern.tests.clone(),
                bindings: pattern.bindings.clone(),
                guard,
                value: TypeScriptExpr::Boolean { value: true },
            },
            TypeScriptDecisionBranch {
                list_prefix: None,
                tests: Vec::new(),
                bindings: Vec::new(),
                guard: None,
                value: TypeScriptExpr::Boolean { value: false },
            },
        ],
        type_ref: TypeScriptType::Boolean,
    }
}

fn pattern_transform(
    parameter: &str,
    pattern: &crate::typescript::decision::TypeScriptPatternDecision,
    transform: TypeScriptExpr,
    transform_type: TypeScriptType,
) -> TypeScriptExpr {
    TypeScriptExpr::Decision {
        scrutinee: Box::new(pattern_scrutinee(parameter, pattern)),
        scrutinee_type: pattern.scrutinee_type.clone(),
        branches: vec![TypeScriptDecisionBranch {
            list_prefix: None,
            tests: pattern.tests.clone(),
            bindings: pattern.bindings.clone(),
            guard: None,
            value: transform,
        }],
        type_ref: transform_type,
    }
}

fn pattern_scrutinee(
    parameter: &str,
    pattern: &crate::typescript::decision::TypeScriptPatternDecision,
) -> TypeScriptExpr {
    let value = TypeScriptExpr::Identifier {
        name: parameter.to_owned(),
    };
    if pattern.private_representation {
        crate::typescript::types::assert_private_representation(
            value,
            pattern.scrutinee_type.clone(),
        )
    } else {
        value
    }
}
