use crate::bytes_ops::runtime_bytes_operation;
use crate::collection_ops::{runtime_collection_operation, runtime_prelude_collection_operation};
use crate::effect_ops::runtime_effect_operation;
use crate::equality_ops::strict_equality_operator_with_evidence;
use crate::int_ops::{runtime_int_operation, runtime_int_operation_with_evidence};
use crate::iterator_ops::runtime_iterator_operation;
use crate::json_ops::{json_decoder_factory_arity, runtime_json_operation};
use crate::list_ops::runtime_list_literal_operation;
use crate::numeric_ops::runtime_numeric_operation;
use crate::provider_service_ops::{
    runtime_provider_service_operation, runtime_provider_service_value_operation,
};
use crate::range_ops::runtime_range_operation;
use crate::regex_ops::runtime_regex_operation;
use crate::signal_ops::{runtime_signal_distinct_operation, runtime_signal_operation};
use crate::standard_ops::runtime_standard_operation;
use crate::stream_ops::runtime_stream_operation;
use crate::sum_ops::runtime_sum_constructor;
use crate::text_ops::runtime_text_operation;
use crate::trait_method_ops::{runtime_trait_method_call_operation, RuntimeTraitMethodOperation};
use crate::typescript::types::TypeScriptTypeContext;
use crate::web_html_ops::runtime_web_html_operation;
use crate::{
    CoreExpr, CoreMonadDoStatement, CoreRecordValueItem, CoreStatement, CoreTemplatePart, CoreType,
};
use seseragi_syntax::{
    standard_operator, standard_trait_operator, StandardOperatorKind, StandardTraitOperator,
};
use std::collections::BTreeMap;

use super::closed_display::selected_dictionary_expression;
use super::decision::lower_core_decision;
use super::names::{local_name, safe_identifier};
use super::types::{
    lower_core_parameter_to_typescript, render_typescript_type, type_ref_from_core_expr,
    type_ref_from_core_type, type_ref_from_core_type_with_erasure,
};
use super::{TypeScriptExpr, TypeScriptRecordValueItem, TypeScriptStatement};

mod comprehension;

use comprehension::lower_array_comprehension;

pub(super) fn lower_core_expr_to_typescript(
    expr: CoreExpr,
    imported_values: &BTreeMap<String, String>,
    imported_types: &TypeScriptTypeContext,
) -> TypeScriptExpr {
    match expr {
        CoreExpr::Unit { .. } => TypeScriptExpr::Undefined,
        CoreExpr::Integer { value, .. } => TypeScriptExpr::Number { value },
        CoreExpr::Float64 { value, .. } => TypeScriptExpr::Number { value },
        CoreExpr::String { value, .. } | CoreExpr::Char { value, .. } => {
            TypeScriptExpr::String { value }
        }
        CoreExpr::Template { parts, .. } => lower_template(parts, imported_values, imported_types),
        CoreExpr::Boolean { value, .. } => TypeScriptExpr::Boolean { value },
        CoreExpr::Variable {
            name,
            callable_signature,
            evidence,
            type_ref,
            origin,
        } => {
            if matches!(type_ref, CoreType::Function { .. }) {
                if runtime_prelude_collection_operation(&name, &evidence).is_some()
                    || runtime_standard_operation(&name).is_some()
                    || runtime_iterator_operation(&name).is_some()
                    || runtime_stream_operation(&name).is_some()
                {
                    // A bare function reference is a partial application with
                    // no source arguments. Keep evidence and currying identical
                    // to ordinary calls instead of emitting an unbound name.
                    return lower_core_expr_to_typescript(
                        CoreExpr::Call {
                            callee_signature: callable_signature,
                            callee: name,
                            arguments: Vec::new(),
                            evidence,
                            deferred_evidence_parameters: Vec::new(),
                            deferred_evidence_type_constructor_parameters: Vec::new(),
                            evidence_argument_index: None,
                            trait_dispatch: None,
                            type_ref,
                            origin,
                        },
                        imported_values,
                        imported_types,
                    );
                }
                if let Some(operation) = runtime_signal_distinct_operation(&name, &evidence) {
                    let source = "_source".to_owned();
                    let equals = equality_reference_from_evidence(
                        &evidence,
                        imported_values,
                        imported_types,
                    )
                    .expect("Signal.distinct requires materialized Eq evidence");
                    return TypeScriptExpr::Lambda {
                        parameter: source.clone(),
                        body: Box::new(TypeScriptExpr::RuntimeCall {
                            callee: operation.local_name.to_owned(),
                            arguments: vec![equals, TypeScriptExpr::Identifier { name: source }],
                        }),
                    };
                }
                if let Some(operation) = runtime_int_operation_with_evidence(&name, &evidence) {
                    return checked_operator_reference(
                        TypeScriptExpr::CurriedRuntimeReference {
                            name: operation.local_name.to_owned(),
                            arity: 2,
                        },
                        &type_ref,
                        imported_types,
                    );
                }
                if let Some(method) = operator_trait_method(&name) {
                    if let Some(dictionary) = evidence.first().and_then(|selected| {
                        selected_dictionary_expression(&selected, imported_values, imported_types)
                    }) {
                        return checked_operator_reference(
                            curried_dictionary_method_reference(dictionary, method),
                            &type_ref,
                            imported_types,
                        );
                    }
                    if name == "+"
                        && matches!(
                            evidence.first().map(|selected| &selected.evidence),
                            Some(crate::CoreInstanceEvidence::Standard { identity, .. })
                                if identity == "std/string::Add"
                        )
                    {
                        return checked_operator_reference(
                            curried_binary_reference("+"),
                            &type_ref,
                            imported_types,
                        );
                    }
                }
                if standard_operator(&name)
                    .is_some_and(|operator| operator.kind == StandardOperatorKind::Comparison)
                {
                    let dictionary = evidence
                        .first()
                        .and_then(|selected| {
                            selected_dictionary_expression(
                                &selected,
                                imported_values,
                                imported_types,
                            )
                        })
                        .expect("comparison reference requires materialized Ord evidence");
                    return lower_comparison_call(
                        &name,
                        dictionary,
                        vec![],
                        &type_ref,
                        imported_types,
                        origin,
                    );
                }
                if matches!(
                    standard_operator(&name).map(|operator| operator.kind),
                    Some(StandardOperatorKind::Equality)
                ) {
                    if let Some(dictionary) = evidence.first().and_then(|selected| {
                        selected_dictionary_expression(&selected, imported_values, imported_types)
                    }) {
                        return checked_operator_reference(
                            if name == "!=" {
                                negated_equality_reference(
                                    dictionary,
                                    &type_ref,
                                    imported_types,
                                    origin,
                                )
                            } else {
                                curried_dictionary_method_reference(dictionary, "eq")
                            },
                            &type_ref,
                            imported_types,
                        );
                    }
                    if let Some(operator) = strict_equality_operator_with_evidence(&name, &evidence)
                    {
                        return checked_operator_reference(
                            curried_binary_reference(operator),
                            &type_ref,
                            imported_types,
                        );
                    }
                }
            }
            let value = runtime_signal_operation(&name)
                .map(|operation| TypeScriptExpr::RuntimeReference {
                    name: operation.local_name.to_owned(),
                })
                .or_else(|| {
                    runtime_provider_service_value_operation(&name, &type_ref).map(|operation| {
                        TypeScriptExpr::RuntimeReference {
                            name: operation.local_name.to_owned(),
                        }
                    })
                })
                .or_else(|| {
                    runtime_effect_operation(&name).map(|operation| {
                        let arity = core_function_arity(&type_ref);
                        if arity > 1 {
                            TypeScriptExpr::CurriedRuntimeReference {
                                name: operation.local_name.to_owned(),
                                arity,
                            }
                        } else {
                            TypeScriptExpr::RuntimeReference {
                                name: operation.local_name.to_owned(),
                            }
                        }
                    })
                })
                .or_else(|| {
                    runtime_stream_operation(&name).map(|operation| {
                        TypeScriptExpr::RuntimeReference {
                            name: operation.local_name.to_owned(),
                        }
                    })
                })
                .or_else(|| {
                    runtime_bytes_operation(&name)
                        .or_else(|| runtime_text_operation(&name))
                        .or_else(|| runtime_regex_operation(&name))
                        .map(|operation| {
                            let arity = core_function_arity(&type_ref);
                            if arity > 1 {
                                TypeScriptExpr::CurriedRuntimeReference {
                                    name: operation.local_name.to_owned(),
                                    arity,
                                }
                            } else {
                                TypeScriptExpr::RuntimeReference {
                                    name: operation.local_name.to_owned(),
                                }
                            }
                        })
                })
                .or_else(|| {
                    runtime_json_operation(&name).map(|operation| {
                        let arity = json_decoder_factory_arity(&name)
                            .unwrap_or_else(|| core_function_arity(&type_ref));
                        if arity > 1 {
                            TypeScriptExpr::CurriedRuntimeReference {
                                name: operation.local_name.to_owned(),
                                arity,
                            }
                        } else {
                            TypeScriptExpr::RuntimeReference {
                                name: operation.local_name.to_owned(),
                            }
                        }
                    })
                })
                .or_else(|| {
                    runtime_numeric_operation(&name).map(|operation| {
                        let arity = core_function_arity(&type_ref);
                        if arity > 1 {
                            TypeScriptExpr::CurriedRuntimeReference {
                                name: operation.local_name.to_owned(),
                                arity,
                            }
                        } else {
                            TypeScriptExpr::RuntimeReference {
                                name: operation.local_name.to_owned(),
                            }
                        }
                    })
                })
                .or_else(|| {
                    runtime_web_html_operation(&name).map(|operation| {
                        TypeScriptExpr::RuntimeReference {
                            name: operation.local_name.to_owned(),
                        }
                    })
                })
                .or_else(|| {
                    runtime_sum_constructor(&name).map(|constructor| {
                        TypeScriptExpr::RuntimeReference {
                            name: constructor.local_name.to_owned(),
                        }
                    })
                })
                .unwrap_or_else(|| TypeScriptExpr::Identifier {
                    name: imported_values
                        .get(&name)
                        .cloned()
                        .unwrap_or_else(|| local_name(&name)),
                });
            if imported_types.is_polymorphic_binding(&name)
                && !matches!(type_ref, CoreType::Function { .. })
            {
                // The initializer is checked under its quantifiers and stored
                // once. Restore this use's checked monotype after TS erasure.
                return TypeScriptExpr::CheckedResult {
                    value: Box::new(TypeScriptExpr::CheckedResult {
                        value: Box::new(value),
                        type_ref: super::TypeScriptType::Unknown,
                    }),
                    type_ref: type_ref_from_core_type(&type_ref, imported_types),
                };
            }
            if matches!(value, TypeScriptExpr::CurriedRuntimeReference { .. }) {
                // A returned runtime function may have no surrounding TS
                // annotation. Its checked source type also types the generated
                // curried parameters without changing their runtime stages.
                return checked_operator_reference(value, &type_ref, imported_types);
            }
            if let Some(signature) = callable_signature {
                let checked = type_ref_from_core_type(&type_ref, imported_types);
                if super::types::instantiated_hkt_callee(&signature, &[], &checked).is_some() {
                    return TypeScriptExpr::CheckedResult {
                        value: Box::new(value),
                        type_ref: checked,
                    };
                }
            }
            value
        }
        CoreExpr::Call {
            callee,
            arguments,
            callee_signature,
            evidence,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            trait_dispatch,
            type_ref,
            origin,
        } => {
            // These canonical Console declarations have exactly one source
            // parameter. A zero-argument partial is the function value, not a
            // call that constructs an Effect with a missing String argument.
            if arguments.is_empty()
                && matches!(type_ref, CoreType::Function { .. })
                && matches!(
                    callee.as_str(),
                    "std/prelude::print"
                        | "std/prelude::println"
                        | "std/console::print"
                        | "std/console::println"
                )
            {
                let name = runtime_effect_operation(&callee)
                    .map(|operation| operation.local_name)
                    .or_else(|| {
                        runtime_provider_service_operation(&callee)
                            .map(|operation| operation.local_name)
                    })
                    .expect("canonical Console text operation has a runtime binding");
                return TypeScriptExpr::RuntimeReference {
                    name: name.to_owned(),
                };
            }
            if imported_types.is_erased_newtype(&callee) && arguments.len() == 1 {
                return super::types::assert_private_representation(
                    lower_core_expr_to_typescript(
                        arguments.into_iter().next().expect("newtype payload"),
                        imported_values,
                        imported_types,
                    ),
                    type_ref_from_core_type(&type_ref, imported_types),
                );
            }
            let signal_operation = runtime_signal_operation(&callee);
            let checked_argument_types = arguments
                .iter()
                .map(|argument| type_ref_from_core_expr(argument, imported_types))
                .collect::<Vec<_>>();
            let signal_type_arguments = signal_operation
                .iter()
                .flat_map(|operation| operation.type_argument_sources)
                .map(|index| {
                    type_ref_from_core_expr(
                        arguments
                            .get(*index)
                            .expect("runtime Signal type argument source must exist"),
                        imported_types,
                    )
                })
                .collect::<Vec<_>>();
            let mut arguments = lower_core_expressions(arguments, imported_values, imported_types);
            if standard_operator(&callee)
                .is_some_and(|operator| operator.kind == StandardOperatorKind::Comparison)
            {
                let dictionary = evidence
                    .first()
                    .and_then(|selected| {
                        selected_dictionary_expression(&selected, imported_values, imported_types)
                    })
                    .expect("comparison application requires materialized Ord evidence");
                lower_comparison_call(
                    &callee,
                    dictionary,
                    arguments,
                    &type_ref,
                    imported_types,
                    origin,
                )
            } else if let Some(operator) = standard_trait_operator(&callee) {
                let dispatch = trait_dispatch
                    .as_ref()
                    .expect("standard trait operator requires typed trait dispatch");
                assert_eq!(dispatch.method, operator.method_name);
                let (selected, method_evidence) = evidence
                    .split_first()
                    .expect("trait operator requires primary instance evidence");
                let selected =
                    selected_dictionary_expression(&selected, imported_values, imported_types)
                        .expect("trait operator requires materialized primary evidence");
                let method_evidence = method_evidence
                    .iter()
                    .map(|selected| {
                        selected_dictionary_expression(&selected, imported_values, imported_types)
                            .expect(
                                "trait operator method constraint requires materialized evidence",
                            )
                    })
                    .collect();
                lower_trait_operator_call(selected, operator, arguments, method_evidence)
            } else if let Some(dispatch) = trait_dispatch {
                if let Some(lowered) = lower_abi_trait_method_call(
                    &dispatch.method,
                    &evidence,
                    arguments.clone(),
                    &checked_argument_types,
                    &type_ref,
                    imported_types,
                    &origin,
                ) {
                    return lowered;
                }
                let (selected, method_evidence) = evidence
                    .split_first()
                    .expect("trait dispatch requires primary instance evidence");
                let selected =
                    selected_dictionary_expression(&selected, imported_values, imported_types)
                        .expect("local trait dispatch requires materialized primary evidence");
                let method_evidence = method_evidence
                    .iter()
                    .map(|selected| {
                        selected_dictionary_expression(&selected, imported_values, imported_types)
                            .expect("trait method constraint requires materialized evidence")
                    })
                    .collect::<Vec<_>>();
                let mut deferred = Vec::new();
                let mut result_type = type_ref.clone();
                if arguments.is_empty()
                    && dispatch
                        .method_contract
                        .as_ref()
                        .is_some_and(|contract| !contract.parameters.is_empty())
                {
                    return TypeScriptExpr::CheckedResult {
                        value: Box::new(TypeScriptExpr::DictionaryCall {
                            dictionary: Box::new(selected),
                            method: crate::method_value_factory_name(&dispatch.method),
                            arguments: method_evidence,
                        }),
                        type_ref: type_ref_from_core_type(&type_ref, imported_types),
                    };
                }
                let supplied = arguments.len();
                let index = if let Some(contract) = &dispatch.method_contract {
                    for (index, parameter) in
                        contract.parameters.iter().enumerate().skip(arguments.len())
                    {
                        let name = format!("__ssrg$method$partial${index}");
                        let parameter_type = type_ref_from_core_type(parameter, imported_types);
                        deferred.push((name, parameter_type));
                        let CoreType::Function { result, .. } = result_type else {
                            unreachable!("checked method partial has a function result")
                        };
                        result_type = *result;
                    }
                    contract.parameters.len()
                } else {
                    arguments.len()
                };
                if supplied < index {
                    let initializer = TypeScriptExpr::DictionaryCall {
                        dictionary: Box::new(selected),
                        method: dispatch.method,
                        arguments,
                    };
                    return stage_deferred_call(
                        initializer,
                        deferred,
                        method_evidence,
                        type_ref_from_core_type(&result_type, imported_types),
                        origin,
                    );
                }
                arguments.splice(index..index, method_evidence);
                // Dictionary methods can erase higher-kinded results. Restore
                // the instantiated result already proved by source evidence,
                // just as for ordinary constrained calls below.
                let value = TypeScriptExpr::CheckedResult {
                    value: Box::new(TypeScriptExpr::DictionaryCall {
                        dictionary: Box::new(selected),
                        method: dispatch.method,
                        arguments,
                    }),
                    type_ref: type_ref_from_core_type(&result_type, imported_types),
                };
                deferred
                    .into_iter()
                    .rev()
                    .fold(value, |body, (name, ty)| TypeScriptExpr::Lambda {
                        parameter: format!("{name}: {}", render_typescript_type(&ty)),
                        body: Box::new(body),
                    })
            } else if let Some(operation) = runtime_prelude_collection_operation(&callee, &evidence)
                .or_else(|| runtime_standard_operation(&callee))
            {
                // Rank-polymorphic Iterable dictionaries do not let TypeScript
                // recover the element type from C. Preserve the types already
                // selected by Seseragi instead of accepting host `unknown`.
                let type_arguments = match (callee.as_str(), evidence.first()) {
                    (name, _) if name.starts_with("std/either::") => {
                        checked_either_runtime_arguments(
                            name,
                            &checked_argument_types,
                            &type_ref,
                            imported_types,
                        )
                    }
                    ("std/array::empty" | "std/array::append", _) => {
                        checked_array_result_element(&type_ref, imported_types)
                            .into_iter()
                            .collect()
                    }
                    ("std/prelude:::", _) => {
                        vec![list_cons_element_type(&type_ref, imported_types)]
                    }
                    ("std/map::fromEntries", Some(selected)) => {
                        let [collection, CoreType::Tuple { elements }] =
                            selected.constraint.arguments.as_slice()
                        else {
                            unreachable!("Map constructor requires pair Iterable evidence")
                        };
                        std::iter::once(collection)
                            .chain(elements.iter())
                            .map(|value| type_ref_from_core_type(value, imported_types))
                            .collect()
                    }
                    (
                        "std/set::fromIterable"
                        | "std/array::fromIterable"
                        | "std/list::fromIterable",
                        Some(selected),
                    ) => selected
                        .constraint
                        .arguments
                        .iter()
                        .map(|value| type_ref_from_core_type(value, imported_types))
                        .collect(),
                    _ => Vec::new(),
                };
                // Standard functions receive their canonical where
                // dictionaries before user arguments, in interface order.
                let dictionaries = if runtime_collection_operation(&callee, &evidence).is_some() {
                    // Concrete reduce exports take only source arguments.
                    Vec::new()
                } else {
                    evidence
                        .iter()
                        .map(|selected| {
                            selected_dictionary_expression(
                                selected,
                                imported_values,
                                imported_types,
                            )
                            .expect("collection call requires materialized evidence")
                        })
                        .collect::<Vec<_>>()
                };
                let remaining_parameters = operation.source_arity.saturating_sub(arguments.len());
                let returned_arguments =
                    arguments.split_off(arguments.len().min(operation.source_arity));
                let dictionary_count = dictionaries.len();
                arguments.splice(0..0, dictionaries);
                let result = lower_uncurried_runtime_call(
                    operation.local_name.to_owned(),
                    arguments,
                    Vec::new(),
                    &type_ref,
                    imported_types,
                    "collection",
                    type_arguments,
                    Some(remaining_parameters),
                    checked_argument_types
                        .iter()
                        .take(operation.source_arity)
                        .cloned()
                        .enumerate()
                        .map(|(index, ty)| (index + dictionary_count, ty))
                        .collect(),
                    origin.clone(),
                );
                let result = if returned_arguments.is_empty() {
                    result
                } else {
                    let name = "__ssrg$collection$returned".to_owned();
                    let callable_type = checked_argument_types[operation.source_arity..]
                        .iter()
                        .rev()
                        .fold(
                            type_ref_from_core_type(&type_ref, imported_types),
                            |result, parameter| super::TypeScriptType::Function {
                                parameter: Box::new(parameter.clone()),
                                result: Box::new(result),
                            },
                        );
                    let initializer = if operation.result_erased {
                        TypeScriptExpr::CheckedResult {
                            value: Box::new(result),
                            type_ref: callable_type.clone(),
                        }
                    } else {
                        result
                    };
                    TypeScriptExpr::Sequence {
                        statements: vec![TypeScriptStatement::PureLet {
                            type_parameters: Vec::new(),
                            name: name.clone(),
                            type_ref: callable_type,
                            initializer,
                            initializer_in_outer_scope: false,
                            origin,
                        }],
                        result: Box::new(TypeScriptExpr::Call {
                            callee: name,
                            arguments: returned_arguments,
                            type_arguments: Vec::new(),
                            checked_callee_type: None,
                        }),
                    }
                };
                if operation.result_erased {
                    TypeScriptExpr::CheckedResult {
                        value: Box::new(result),
                        type_ref: type_ref_from_core_type(&type_ref, imported_types),
                    }
                } else {
                    result
                }
            } else if let Some(operation) = runtime_stream_operation(&callee) {
                let dictionaries = if callee == "std/stream::fromIterable" {
                    evidence
                        .iter()
                        .map(|selected| {
                            selected_dictionary_expression(
                                &selected,
                                imported_values,
                                imported_types,
                            )
                            .expect("Stream.fromIterable requires materialized Iterable evidence")
                        })
                        .collect()
                } else {
                    Vec::new()
                };
                lower_uncurried_runtime_call(
                    operation.local_name.to_owned(),
                    arguments,
                    dictionaries,
                    &type_ref,
                    imported_types,
                    "stream",
                    Vec::new(),
                    None,
                    checked_argument_types.iter().cloned().enumerate().collect(),
                    origin,
                )
            } else if callee == "std/prelude::reduce"
                && matches!(evidence.as_slice(), [selected] if selected.constraint.name == "Reducible")
            {
                let dictionary =
                    selected_dictionary_expression(&evidence[0], imported_values, imported_types)
                        .expect("user Reducible call requires materialized dictionary evidence");
                TypeScriptExpr::DictionaryCall {
                    dictionary: Box::new(dictionary),
                    method: "reduce".to_owned(),
                    arguments,
                }
            } else if let Some(operation) = runtime_iterator_operation(&callee) {
                let remaining = operation.source_arity.saturating_sub(arguments.len());
                lower_uncurried_runtime_call(
                    operation.local_name.to_owned(),
                    arguments,
                    Vec::new(),
                    &type_ref,
                    imported_types,
                    "iterator",
                    Vec::new(),
                    Some(remaining),
                    checked_argument_types.iter().cloned().enumerate().collect(),
                    origin,
                )
            } else if let Some(operation) = runtime_bytes_operation(&callee)
                .or_else(|| runtime_text_operation(&callee))
                .or_else(|| runtime_regex_operation(&callee))
            {
                lower_uncurried_runtime_call(
                    operation.local_name.to_owned(),
                    arguments,
                    Vec::new(),
                    &type_ref,
                    imported_types,
                    "text_bytes",
                    Vec::new(),
                    None,
                    checked_argument_types.iter().cloned().enumerate().collect(),
                    origin,
                )
            } else if let Some(operation) = runtime_json_operation(&callee) {
                let factory_arity = json_decoder_factory_arity(&callee);
                let supplied = arguments.len();
                let returned_arguments = factory_arity
                    .filter(|arity| supplied > *arity)
                    .map(|arity| arguments.split_off(arity))
                    .unwrap_or_default();
                arguments.extend(evidence.iter().map(|selected| {
                    selected_dictionary_expression(&selected, imported_values, imported_types)
                        .expect("JSON operation requires materializable codec evidence")
                }));
                if let Some(arity) = factory_arity {
                    let call = lower_uncurried_runtime_call(
                        operation.local_name.to_owned(),
                        arguments,
                        Vec::new(),
                        &type_ref,
                        imported_types,
                        "json",
                        Vec::new(),
                        Some(arity.saturating_sub(supplied)),
                        checked_argument_types
                            .iter()
                            .take(supplied.min(arity))
                            .cloned()
                            .enumerate()
                            .collect(),
                        origin.clone(),
                    );
                    if returned_arguments.is_empty() {
                        call
                    } else {
                        let name = "__ssrg$json$returned".to_owned();
                        let callable_type = checked_argument_types[arity..].iter().rev().fold(
                            type_ref_from_core_type(&type_ref, imported_types),
                            |result, parameter| super::TypeScriptType::Function {
                                parameter: Box::new(parameter.clone()),
                                result: Box::new(result),
                            },
                        );
                        TypeScriptExpr::Sequence {
                            statements: vec![TypeScriptStatement::PureLet {
                                type_parameters: Vec::new(),
                                name: name.clone(),
                                type_ref: callable_type,
                                initializer: call,
                                initializer_in_outer_scope: false,
                                origin,
                            }],
                            result: Box::new(TypeScriptExpr::Call {
                                callee: name,
                                arguments: returned_arguments,
                                type_arguments: Vec::new(),
                                checked_callee_type: None,
                            }),
                        }
                    }
                } else {
                    TypeScriptExpr::RuntimeCall {
                        callee: operation.local_name.to_owned(),
                        arguments,
                    }
                }
            } else if let Some(operation) = runtime_numeric_operation(&callee) {
                lower_uncurried_runtime_call(
                    operation.local_name.to_owned(),
                    arguments,
                    Vec::new(),
                    &type_ref,
                    imported_types,
                    "numeric",
                    Vec::new(),
                    None,
                    checked_argument_types.iter().cloned().enumerate().collect(),
                    origin,
                )
            } else if let Some(constructor) = runtime_sum_constructor(&callee) {
                TypeScriptExpr::RuntimeCall {
                    callee: constructor.local_name.to_owned(),
                    arguments,
                }
            } else if let Some(operation) = runtime_web_html_operation(&callee) {
                TypeScriptExpr::RuntimeCall {
                    callee: operation.local_name.to_owned(),
                    arguments,
                }
            } else if let Some(operation) = runtime_provider_service_operation(&callee) {
                if let Some(source_arity) = operation.source_arity {
                    // An imported Effect function may be a zero-argument
                    // partial Call. The standard declaration fixes its source
                    // stages; a returned function and the host function's
                    // implementation arity must not add or remove a stage.
                    let remaining = source_arity.saturating_sub(arguments.len());
                    lower_uncurried_runtime_call(
                        operation.local_name.to_owned(),
                        arguments,
                        Vec::new(),
                        &type_ref,
                        imported_types,
                        "provider",
                        Vec::new(),
                        Some(remaining),
                        checked_argument_types.iter().cloned().enumerate().collect(),
                        origin,
                    )
                } else {
                    TypeScriptExpr::RuntimeCall {
                        callee: operation.local_name.to_owned(),
                        arguments,
                    }
                }
            } else if let Some(operation) = runtime_effect_operation(&callee) {
                let mut dictionary_arguments = Vec::new();
                if matches!(callee.as_str(), "std/test::equal" | "std/test::notEqual") {
                    let equality = evidence
                        .iter()
                        .find(|selected| selected.constraint.name == "Eq")
                        .expect("std/test equality assertion requires selected Eq evidence");
                    dictionary_arguments.push(
                        equality_reference_from_evidence(
                            std::slice::from_ref(equality),
                            imported_values,
                            imported_types,
                        )
                        .expect("std/test equality assertion requires Eq evidence"),
                    );
                    let debug = evidence
                        .iter()
                        .find(|selected| selected.constraint.name == "Debug")
                        .and_then(|selected| {
                            selected_dictionary_expression(
                                &selected,
                                imported_values,
                                imported_types,
                            )
                        })
                        .expect("std/test equality assertion requires Debug evidence");
                    dictionary_arguments.push(debug);
                } else {
                    dictionary_arguments.extend(evidence.iter().map(|selected| {
                        selected_dictionary_expression(&selected, imported_values, imported_types)
                            .expect(
                                "constrained standard effect call requires materializable evidence",
                            )
                    }));
                }
                let mut parameters = Vec::new();
                let mut remaining = &type_ref;
                while let CoreType::Function { parameter, result } = remaining {
                    let name = format!("__ssrg$effect$partial${}", parameters.len());
                    let type_name =
                        render_typescript_type(&type_ref_from_core_type(parameter, imported_types));
                    arguments.push(TypeScriptExpr::Identifier { name: name.clone() });
                    parameters.push((name, type_name));
                    remaining = result;
                }
                arguments.extend(dictionary_arguments);
                parameters.into_iter().rev().fold(
                    TypeScriptExpr::RuntimeCall {
                        callee: operation.local_name.to_owned(),
                        arguments,
                    },
                    |body, (name, type_name)| TypeScriptExpr::Lambda {
                        parameter: format!("{name}: {type_name}"),
                        body: Box::new(body),
                    },
                )
            } else if let Some(operation) = runtime_signal_distinct_operation(&callee, &evidence) {
                let equals =
                    equality_reference_from_evidence(&evidence, imported_values, imported_types)
                        .expect("Signal.distinct requires materialized Eq evidence");
                arguments.insert(0, equals);
                TypeScriptExpr::RuntimeCall {
                    callee: operation.local_name.to_owned(),
                    arguments,
                }
            } else if let Some(operation) = signal_operation {
                if signal_type_arguments.is_empty() {
                    TypeScriptExpr::RuntimeCall {
                        callee: operation.local_name.to_owned(),
                        arguments,
                    }
                } else {
                    TypeScriptExpr::TypeApplicationCall {
                        callee: operation.local_name.to_owned(),
                        type_arguments: signal_type_arguments,
                        arguments,
                    }
                }
            } else {
                let foreign_result = imported_types.foreign_scalar_result(
                    &callee,
                    arguments.len(),
                    &type_ref,
                    &evidence,
                );
                let evidence = evidence
                    .iter()
                    .map(|selected| {
                        selected_dictionary_expression(&selected, imported_values, imported_types)
                            .expect("constrained function call requires materializable evidence")
                    })
                    .collect::<Vec<_>>();
                let callee = imported_values
                    .get(&callee)
                    .cloned()
                    .unwrap_or_else(|| local_name(&callee));
                if imported_values.contains_key(&format!("__ssrg$foreign$task:{callee}"))
                    && evidence.is_empty()
                    && deferred_evidence_parameters.is_empty()
                    && deferred_evidence_type_constructor_parameters.is_empty()
                {
                    TypeScriptExpr::ForeignTaskCall {
                        callee,
                        arguments,
                        function: imported_values
                            .get("__ssrg$foreign$function")
                            .cloned()
                            .unwrap_or_else(|| "<module>".to_owned()),
                        module: imported_values
                            .get("__ssrg$foreign$module")
                            .cloned()
                            .unwrap_or_else(|| origin.source.clone()),
                        origin,
                    }
                } else {
                    let has_evidence = !evidence.is_empty();
                    let type_arguments =
                        ordinary_call_type_arguments(callee_signature.as_ref(), imported_types);
                    let checked_result = type_ref_from_core_type_with_erasure(
                        foreign_result.as_ref().unwrap_or(&type_ref),
                        imported_types,
                        &deferred_evidence_type_constructor_parameters,
                    );
                    let check_late_callback = has_evidence
                        && type_arguments.is_empty()
                        && deferred_evidence_parameters.is_empty()
                        && (checked_argument_types.iter().skip(1).any(|argument| {
                            matches!(argument, super::TypeScriptType::Function { .. })
                        }) || checked_argument_types
                            .first()
                            .is_some_and(callback_returns_either));
                    let hkt_callee = (!has_evidence)
                        .then(|| callee_signature.as_ref())
                        .flatten()
                        .filter(|signature| signature.argument_types.len() == arguments.len())
                        .and_then(|signature| {
                            let checked = signature
                                .argument_types
                                .iter()
                                .map(|ty| type_ref_from_core_type(ty, imported_types))
                                .collect::<Vec<_>>();
                            super::types::instantiated_hkt_callee(
                                signature,
                                &checked,
                                &checked_result,
                            )
                        });
                    let evidence_count = evidence.len();
                    let factory_callee = (has_evidence
                        && deferred_evidence_parameters.is_empty()
                        && evidence_argument_index == (!arguments.is_empty()).then_some(0))
                    .then(|| callee_signature.as_ref())
                    .flatten()
                    .filter(|signature| {
                        signature.parameters.is_empty()
                            && signature.type_parameters.is_empty()
                            && signature.argument_types.len() == arguments.len()
                            && matches!(signature.result, CoreType::Function { .. })
                    })
                    .map(|signature| {
                        (0..evidence_count).fold(
                            type_ref_from_core_type(&signature.result, imported_types),
                            |result, _| super::TypeScriptType::Function {
                                parameter: Box::new(super::TypeScriptType::Unknown),
                                result: Box::new(result),
                            },
                        )
                    });
                    // Typed HIR represents an imported effect reference as a
                    // zero-argument partial Call. Preserve the callable value;
                    // emitting f() would construct it before its argument exists.
                    let mut call = if arguments.is_empty()
                        && evidence.is_empty()
                        && deferred_evidence_parameters.is_empty()
                        && matches!(type_ref, CoreType::Function { .. })
                    {
                        TypeScriptExpr::Identifier { name: callee }
                    } else {
                        lower_constrained_call(
                            callee,
                            arguments,
                            type_arguments,
                            evidence,
                            deferred_evidence_parameters,
                            deferred_evidence_type_constructor_parameters,
                            evidence_argument_index,
                            imported_types,
                            checked_result.clone(),
                            &checked_argument_types,
                            origin,
                        )
                    };
                    if check_late_callback {
                        // TypeScript fixes generic parameters at the first
                        // curried argument. The source checker used every
                        // argument and evidence; retain that instantiated
                        // callable type before applying any callback. A
                        // returned source function follows the dictionaries;
                        // its arguments must not shift that checked boundary.
                        let mut parameters = checked_argument_types;
                        let index = evidence_argument_index.unwrap_or(parameters.len());
                        parameters.splice(
                            index..index,
                            std::iter::repeat_n(super::TypeScriptType::Unknown, evidence_count),
                        );
                        let instantiated = parameters.into_iter().rev().fold(
                            checked_result.clone(),
                            |result, parameter| super::TypeScriptType::Function {
                                parameter: Box::new(parameter),
                                result: Box::new(result),
                            },
                        );
                        if let TypeScriptExpr::Call {
                            checked_callee_type,
                            ..
                        } = &mut call
                        {
                            *checked_callee_type = Some(instantiated);
                        }
                    }
                    if let Some(instantiated) = factory_callee.or(hkt_callee) {
                        match &mut call {
                            TypeScriptExpr::Call {
                                checked_callee_type,
                                ..
                            } => *checked_callee_type = Some(instantiated),
                            TypeScriptExpr::Identifier { .. } => {
                                call = TypeScriptExpr::CheckedResult {
                                    value: Box::new(call),
                                    type_ref: checked_result.clone(),
                                };
                            }
                            _ => {}
                        }
                    }
                    if has_evidence {
                        // Host inference commits generics at the first curried
                        // argument, before erased dictionary parameters arrive.
                        // The source checker has already proved this result.
                        TypeScriptExpr::CheckedResult {
                            value: Box::new(call),
                            type_ref: checked_result,
                        }
                    } else {
                        call
                    }
                }
            }
        }
        CoreExpr::Lambda {
            parameter, body, ..
        } => {
            let parameter = lower_core_parameter_to_typescript(parameter, imported_types, &[]);
            TypeScriptExpr::Lambda {
                parameter: format!("{}: {}", parameter.name, parameter.type_name),
                body: Box::new(lower_core_expr_to_typescript(
                    *body,
                    imported_values,
                    imported_types,
                )),
            }
        }
        CoreExpr::Tuple { elements, .. } => TypeScriptExpr::Tuple {
            elements: lower_core_expressions(elements, imported_values, imported_types),
        },
        CoreExpr::FieldAccess {
            receiver,
            field,
            type_ref,
            ..
        } => {
            let public_type = type_ref_from_core_expr(&receiver, imported_types);
            let private_type =
                type_ref_from_core_expr(&receiver, &imported_types.with_private_representation());
            let receiver =
                lower_core_expr_to_typescript(*receiver, imported_values, imported_types);
            if private_type != public_type {
                TypeScriptExpr::CheckedResult {
                    value: Box::new(TypeScriptExpr::FieldAccess {
                        receiver: Box::new(super::types::assert_private_representation(
                            receiver,
                            private_type,
                        )),
                        field,
                    }),
                    type_ref: type_ref_from_core_type(&type_ref, imported_types),
                }
            } else {
                TypeScriptExpr::FieldAccess {
                    receiver: Box::new(receiver),
                    field,
                }
            }
        }
        CoreExpr::OptionalFieldAccess {
            receiver,
            field,
            type_ref,
            ..
        } => TypeScriptExpr::OptionalFieldAccess {
            receiver: Box::new(lower_core_expr_to_typescript(
                *receiver,
                imported_values,
                imported_types,
            )),
            field,
            value_type: match type_ref_from_core_type(&type_ref, imported_types) {
                super::TypeScriptType::Maybe { element } => *element,
                _ => unreachable!("optional field access must return Maybe"),
            },
            just_constructor: runtime_sum_constructor("std/prelude::Just")
                .expect("standard Just constructor must exist")
                .local_name
                .to_owned(),
            nothing_constructor: runtime_sum_constructor("std/prelude::Nothing")
                .expect("standard Nothing constructor must exist")
                .local_name
                .to_owned(),
        },
        CoreExpr::Record {
            items, type_ref, ..
        } => TypeScriptExpr::Record {
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
            asserted_type: (!matches!(type_ref, crate::CoreType::Record { .. }))
                .then(|| super::types::type_ref_from_core_type(&type_ref, imported_types)),
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
            let element_type = match &type_ref {
                CoreType::Named { name, arguments } if name == "List" && arguments.len() == 1 => {
                    type_ref_from_core_type(&arguments[0], imported_types)
                }
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
        CoreExpr::Unary {
            operator,
            operand,
            type_ref,
            ..
        } => {
            let operand = lower_core_expr_to_typescript(*operand, imported_values, imported_types);
            if operator == "-" && is_int_type(&type_ref) {
                let operation =
                    runtime_int_operation("-").expect("Int subtraction runtime is registered");
                TypeScriptExpr::RuntimeCall {
                    callee: operation.local_name.to_owned(),
                    arguments: vec![
                        TypeScriptExpr::Number {
                            value: "0".to_owned(),
                        },
                        operand,
                    ],
                }
            } else {
                TypeScriptExpr::Unary {
                    operator,
                    operand: Box::new(operand),
                }
            }
        }
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
            callee,
            callee_signature,
            arguments,
            evidence,
            origin,
            success,
            ..
        } => {
            let mut arguments = lower_core_expressions(arguments, imported_values, imported_types);
            arguments.extend(evidence.iter().map(|selected| {
                selected_dictionary_expression(&selected, imported_values, imported_types)
                    .expect("constrained effect call requires materializable evidence")
            }));
            if let Some(operation) = runtime_effect_operation(&callee) {
                if callee == "std/deferred::make" {
                    let payloads = match &success {
                        CoreType::ExternalNamed {
                            canonical,
                            arguments,
                            ..
                        } if canonical == "std/deferred::Deferred" && arguments.len() == 2 => {
                            Some(arguments)
                        }
                        CoreType::Named { name, arguments }
                            if name == "std/deferred::Deferred" && arguments.len() == 2 =>
                        {
                            Some(arguments)
                        }
                        _ => None,
                    };
                    if let Some(payloads) = payloads {
                        return TypeScriptExpr::TypeApplicationCall {
                            callee: operation.local_name.to_owned(),
                            type_arguments: payloads
                                .iter()
                                .map(|ty| type_ref_from_core_type(ty, imported_types))
                                .collect(),
                            arguments,
                        };
                    }
                }
                TypeScriptExpr::RuntimeCall {
                    callee: operation.local_name.to_owned(),
                    arguments,
                }
            } else if let Some(operation) = runtime_stream_operation(&callee) {
                TypeScriptExpr::RuntimeCall {
                    callee: operation.local_name.to_owned(),
                    arguments,
                }
            } else if let Some(operation) = runtime_provider_service_operation(&callee) {
                TypeScriptExpr::RuntimeCall {
                    callee: operation.local_name.to_owned(),
                    arguments,
                }
            } else {
                let callee_name = imported_values
                    .get(&callee)
                    .cloned()
                    .unwrap_or_else(|| local_name(&callee));
                if imported_values.contains_key(&format!("__ssrg$foreign$task:{callee_name}")) {
                    TypeScriptExpr::ForeignTaskCall {
                        callee: callee_name,
                        arguments,
                        function: imported_values
                            .get("__ssrg$foreign$function")
                            .cloned()
                            .unwrap_or_else(|| "<module>".to_owned()),
                        module: imported_values
                            .get("__ssrg$foreign$module")
                            .cloned()
                            .unwrap_or_else(|| origin.source.clone()),
                        origin,
                    }
                } else {
                    TypeScriptExpr::Call {
                        callee: callee_name,
                        arguments,
                        type_arguments: ordinary_call_type_arguments(
                            callee_signature.as_ref(),
                            imported_types,
                        ),
                        checked_callee_type: None,
                    }
                }
            }
        }
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
                selected_dictionary_expression(&evidence, imported_values, imported_types)
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

fn lower_abi_trait_method_call(
    method: &str,
    evidence: &[crate::CoreCallEvidence],
    arguments: Vec<TypeScriptExpr>,
    argument_types: &[super::TypeScriptType],
    result_type: &CoreType,
    imported_types: &TypeScriptTypeContext,
    origin: &crate::SourceSpan,
) -> Option<TypeScriptExpr> {
    let operation = runtime_trait_method_call_operation(method, evidence, arguments.len())?;
    if let RuntimeTraitMethodOperation::Equality(dictionary) = operation {
        // The runtime method already has typed curried stages. Applying its
        // first operand now also preserves strict partial application.
        return Some(TypeScriptExpr::DictionaryCall {
            dictionary: Box::new(TypeScriptExpr::RuntimeReference {
                name: dictionary.local_name.to_owned(),
            }),
            method: method.to_owned(),
            arguments,
        });
    }
    let binary = |left, right| match operation {
        RuntimeTraitMethodOperation::Int(operation) => TypeScriptExpr::RuntimeCall {
            callee: operation.local_name.to_owned(),
            arguments: vec![left, right],
        },
        RuntimeTraitMethodOperation::Native(operator) => TypeScriptExpr::Binary {
            operator: operator.to_owned(),
            left: Box::new(left),
            right: Box::new(right),
        },
        RuntimeTraitMethodOperation::Equality(_) => {
            unreachable!("typed equality was lowered through its selected dictionary")
        }
    };
    match arguments.as_slice() {
        [left, right] => Some(binary(left.clone(), right.clone())),
        [left] => {
            let CoreType::Function { parameter, .. } = result_type else {
                unreachable!("checked operator partial has one remaining source stage")
            };
            let saved = "__ssrg$operator$saved0".to_owned();
            let right = "_argument1".to_owned();
            Some(TypeScriptExpr::Sequence {
                statements: vec![TypeScriptStatement::PureLet {
                    type_parameters: vec![],
                    name: saved.clone(),
                    type_ref: argument_types[0].clone(),
                    initializer: left.clone(),
                    initializer_in_outer_scope: false,
                    origin: origin.clone(),
                }],
                result: Box::new(TypeScriptExpr::Lambda {
                    parameter: format!(
                        "{right}: {}",
                        render_typescript_type(&type_ref_from_core_type(parameter, imported_types))
                    ),
                    body: Box::new(binary(
                        TypeScriptExpr::Identifier { name: saved },
                        TypeScriptExpr::Identifier { name: right },
                    )),
                }),
            })
        }
        _ => None,
    }
}

fn lower_template(
    parts: Vec<CoreTemplatePart>,
    imported_values: &BTreeMap<String, String>,
    imported_types: &TypeScriptTypeContext,
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
                let dictionary =
                    selected_dictionary_expression(&evidence, imported_values, imported_types)
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

fn callback_returns_either(type_ref: &super::TypeScriptType) -> bool {
    let mut result = type_ref;
    while let super::TypeScriptType::Function { result: next, .. } = result {
        result = next;
    }
    matches!(type_ref, super::TypeScriptType::Function { .. })
        && matches!(result, super::TypeScriptType::Either { .. })
}

fn lower_constrained_call(
    callee: String,
    mut arguments: Vec<TypeScriptExpr>,
    type_arguments: Vec<super::TypeScriptType>,
    evidence: Vec<TypeScriptExpr>,
    deferred_parameters: Vec<CoreType>,
    deferred_type_constructor_parameters: Vec<String>,
    evidence_argument_index: Option<usize>,
    imported_types: &TypeScriptTypeContext,
    mut result_type: super::TypeScriptType,
    supplied_types: &[super::TypeScriptType],
    origin: crate::SourceSpan,
) -> TypeScriptExpr {
    if deferred_parameters.is_empty() {
        let index = evidence_argument_index.unwrap_or(arguments.len());
        arguments.splice(index..index, evidence);
        return TypeScriptExpr::Call {
            callee,
            arguments,
            type_arguments,
            checked_callee_type: None,
        };
    }

    let parameters = deferred_parameters
        .into_iter()
        .enumerate()
        .map(|(index, type_ref)| {
            let name = format!("__ssrg$partial${index}");
            let parameter_type = type_ref_from_core_type_with_erasure(
                &type_ref,
                imported_types,
                &deferred_type_constructor_parameters,
            );
            let super::TypeScriptType::Function { result, .. } = &result_type else {
                unreachable!("checked partial source arity");
            };
            result_type = (**result).clone();
            (name, parameter_type)
        })
        .collect::<Vec<_>>();
    let initializer = if arguments.is_empty() {
        TypeScriptExpr::Identifier { name: callee }
    } else {
        // Check the supplied prefix before TS infers its generic parameters
        // from a callback's Either union. The remaining source parameters still
        // precede dictionaries, which precede any returned function's stages.
        let checked_callee_type = (type_arguments.is_empty()
            && supplied_types.iter().any(callback_returns_either))
        .then(|| {
            supplied_types
                .iter()
                .cloned()
                .chain(parameters.iter().map(|(_, ty)| ty.clone()))
                .chain(std::iter::repeat_n(
                    super::TypeScriptType::Unknown,
                    evidence.len(),
                ))
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .fold(result_type.clone(), |result, parameter| {
                    super::TypeScriptType::Function {
                        parameter: Box::new(parameter),
                        result: Box::new(result),
                    }
                })
        });
        TypeScriptExpr::Call {
            callee,
            arguments,
            type_arguments,
            checked_callee_type,
        }
    };
    stage_deferred_call(initializer, parameters, evidence, result_type, origin)
}

/// Evaluate the callee and the supplied source prefix now. Only the remaining
/// source stages and their following dictionaries belong inside the closure.
/// The stage type is the checked instantiation, never host inference from a
/// later argument or an extra source arrow inferred from a returned function.
fn stage_deferred_call(
    initializer: TypeScriptExpr,
    parameters: Vec<(String, super::TypeScriptType)>,
    evidence: Vec<TypeScriptExpr>,
    result_type: super::TypeScriptType,
    origin: crate::SourceSpan,
) -> TypeScriptExpr {
    let name = "__ssrg$partial$stage".to_owned();
    let mut stage_type = result_type;
    for _ in &evidence {
        stage_type = super::TypeScriptType::Function {
            parameter: Box::new(super::TypeScriptType::Unknown),
            result: Box::new(stage_type),
        };
    }
    for (_, parameter) in parameters.iter().rev() {
        stage_type = super::TypeScriptType::Function {
            parameter: Box::new(parameter.clone()),
            result: Box::new(stage_type),
        };
    }
    let arguments = parameters
        .iter()
        .map(|(name, _)| TypeScriptExpr::Identifier { name: name.clone() })
        .chain(evidence)
        .collect();
    let result = parameters.into_iter().rev().fold(
        TypeScriptExpr::Call {
            callee: name.clone(),
            arguments,
            type_arguments: Vec::new(),
            checked_callee_type: None,
        },
        |body, (parameter, ty)| TypeScriptExpr::Lambda {
            parameter: format!("{parameter}: {}", render_typescript_type(&ty)),
            body: Box::new(body),
        },
    );
    TypeScriptExpr::Sequence {
        statements: vec![TypeScriptStatement::PureLet {
            type_parameters: vec![],
            name,
            type_ref: stage_type.clone(),
            initializer: TypeScriptExpr::CheckedResult {
                value: Box::new(initializer),
                type_ref: stage_type,
            },
            initializer_in_outer_scope: false,
            origin,
        }],
        result: Box::new(result),
    }
}

fn core_function_arity(type_ref: &CoreType) -> usize {
    match type_ref {
        CoreType::Function { result, .. } => 1 + core_function_arity(result),
        _ => 0,
    }
}

fn lower_uncurried_runtime_call(
    callee: String,
    mut arguments: Vec<TypeScriptExpr>,
    trailing_arguments: Vec<TypeScriptExpr>,
    result_type: &CoreType,
    imported_types: &TypeScriptTypeContext,
    parameter_scope: &str,
    type_arguments: Vec<super::TypeScriptType>,
    remaining_parameters: Option<usize>,
    supplied_types: Vec<(usize, super::TypeScriptType)>,
    origin: crate::SourceSpan,
) -> TypeScriptExpr {
    let supplied = arguments.len();
    let mut parameters = Vec::new();
    let mut remaining = result_type;
    while let CoreType::Function { parameter, result } = remaining {
        if remaining_parameters.is_some_and(|count| parameters.len() == count) {
            break;
        }
        let index = parameters.len();
        let name = format!("__ssrg${parameter_scope}$partial${index}");
        let type_name = render_typescript_type(&type_ref_from_core_type(parameter, imported_types));
        arguments.push(TypeScriptExpr::Identifier { name: name.clone() });
        parameters.push((name, type_name));
        remaining = result;
    }
    // A runtime export is uncurried, but its source partial is strict. Capture
    // only the supplied source values now, using their checked CoreIr types.
    // Implicit dictionaries keep the runtime ABI's existing placement.
    let mut statements = Vec::new();
    if !parameters.is_empty() {
        for (index, type_ref) in supplied_types {
            assert!(index < supplied, "checked supplied runtime argument");
            let name = format!("__ssrg${parameter_scope}$saved{index}");
            let initializer = std::mem::replace(
                &mut arguments[index],
                TypeScriptExpr::Identifier { name: name.clone() },
            );
            statements.push(TypeScriptStatement::PureLet {
                type_parameters: vec![],
                name,
                type_ref,
                initializer,
                initializer_in_outer_scope: false,
                origin: origin.clone(),
            });
        }
    }
    arguments.extend(trailing_arguments);
    let call = if type_arguments.is_empty() {
        TypeScriptExpr::RuntimeCall { callee, arguments }
    } else {
        TypeScriptExpr::TypeApplicationCall {
            callee,
            type_arguments,
            arguments,
        }
    };
    let result = parameters
        .into_iter()
        .rev()
        .fold(call, |body, (name, type_name)| TypeScriptExpr::Lambda {
            parameter: format!("{name}: {type_name}"),
            body: Box::new(body),
        });
    if statements.is_empty() {
        result
    } else {
        TypeScriptExpr::Sequence {
            statements,
            result: Box::new(result),
        }
    }
}

pub(super) fn typescript_expr_contains_await(expr: &TypeScriptExpr) -> bool {
    match expr {
        TypeScriptExpr::Await { .. } => true,
        TypeScriptExpr::CheckedResult { value, .. } => typescript_expr_contains_await(value),
        TypeScriptExpr::Tuple { elements } | TypeScriptExpr::Array { elements, .. } => {
            elements.iter().any(typescript_expr_contains_await)
        }
        TypeScriptExpr::FieldAccess { receiver, .. }
        | TypeScriptExpr::OptionalFieldAccess { receiver, .. } => {
            typescript_expr_contains_await(receiver)
        }
        TypeScriptExpr::Record { items, .. } => items
            .iter()
            .any(|item| typescript_expr_contains_await(item.value())),
        TypeScriptExpr::Lambda { body, .. } => typescript_expr_contains_await(body),
        TypeScriptExpr::Binary { left, right, .. } => {
            typescript_expr_contains_await(left) || typescript_expr_contains_await(right)
        }
        TypeScriptExpr::Unary { operand, .. } => typescript_expr_contains_await(operand),
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
        | TypeScriptExpr::ForeignTaskCall { arguments, .. }
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
        | TypeScriptExpr::EmptyDictionary
        | TypeScriptExpr::Bigint { .. }
        | TypeScriptExpr::Number { .. }
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
    imported_types: &TypeScriptTypeContext,
) -> Vec<TypeScriptExpr> {
    expressions
        .into_iter()
        .map(|expression| {
            lower_core_expr_to_typescript(expression, imported_values, imported_types)
        })
        .collect()
}

// Empty inputs cannot supply a host generic argument. Keep the element
// already selected by the source checker before TypeScript infers unknown.
fn checked_either_runtime_arguments(
    name: &str,
    supplied: &[super::TypeScriptType],
    result: &CoreType,
    imported_types: &TypeScriptTypeContext,
) -> Vec<super::TypeScriptType> {
    use super::TypeScriptType as Ty;
    let arity = runtime_standard_operation(name)
        .expect("checked Either operation")
        .source_arity;
    let full = supplied.iter().rev().fold(
        type_ref_from_core_type(result, imported_types),
        |result, parameter| Ty::Function {
            parameter: Box::new(parameter.clone()),
            result: Box::new(result),
        },
    );
    let mut remaining = &full;
    let mut parameters = Vec::new();
    for _ in 0..arity {
        let Ty::Function { parameter, result } = remaining else {
            return Vec::new();
        };
        parameters.push(parameter.as_ref());
        remaining = result;
    }
    let arrow = |value: &Ty| match value {
        Ty::Function { parameter, result } => {
            Some((parameter.as_ref().clone(), result.as_ref().clone()))
        }
        _ => None,
    };
    let sum = |value: &Ty| match value {
        Ty::Either { error, value } => Some((error.as_ref().clone(), value.as_ref().clone())),
        _ => None,
    };
    let selected = (|| {
        Some(match name {
            "std/either::mapLeft" => {
                let (error, value) = sum(parameters[1])?;
                let (_, mapped) = arrow(parameters[0])?;
                vec![error, mapped, value]
            }
            "std/either::mapRight" => {
                let (error, value) = sum(parameters[1])?;
                let (_, mapped) = arrow(parameters[0])?;
                vec![error, value, mapped]
            }
            "std/either::bimap" => {
                let (error, value) = sum(parameters[2])?;
                let (_, mapped_error) = arrow(parameters[0])?;
                let (_, mapped_value) = arrow(parameters[1])?;
                vec![error, mapped_error, value, mapped_value]
            }
            "std/either::fold" => {
                let (error, value) = sum(parameters[2])?;
                vec![error, value, remaining.clone()]
            }
            "std/either::swap" => {
                let (error, value) = sum(parameters[0])?;
                vec![error, value]
            }
            "std/either::traverse" => {
                let (input, output) = arrow(parameters[0])?;
                let (error, value) = sum(&output)?;
                vec![error, input, value]
            }
            _ => return None,
        })
    })();
    selected.unwrap_or_default()
}

fn checked_array_result_element(
    type_ref: &CoreType,
    imported_types: &TypeScriptTypeContext,
) -> Option<super::TypeScriptType> {
    let mut result = type_ref;
    while let CoreType::Function { result: next, .. } = result {
        result = next;
    }
    let arguments = match result {
        CoreType::Named { name, arguments }
            if matches!(name.as_str(), "Array" | "std/prelude::Array") =>
        {
            arguments
        }
        CoreType::ExternalNamed {
            canonical,
            arguments,
            ..
        } if canonical == "std/prelude::Array" => arguments,
        _ => return None,
    };
    let [element] = arguments.as_slice() else {
        return None;
    };
    (!matches!(element, CoreType::Hole)).then(|| type_ref_from_core_type(element, imported_types))
}

fn list_cons_element_type(
    type_ref: &CoreType,
    imported_types: &TypeScriptTypeContext,
) -> super::TypeScriptType {
    let mut result = type_ref;
    while let CoreType::Function { result: next, .. } = result {
        result = next;
    }
    match result {
        CoreType::Named { name, arguments } if name == "List" && arguments.len() == 1 => {
            type_ref_from_core_type(&arguments[0], imported_types)
        }
        _ => unreachable!("checked List cons must return a List"),
    }
}

fn lower_binary(
    operator: String,
    left: CoreExpr,
    right: CoreExpr,
    evidence: Vec<crate::CoreCallEvidence>,
    type_ref: CoreType,
    imported_values: &BTreeMap<String, String>,
    imported_types: &TypeScriptTypeContext,
) -> TypeScriptExpr {
    let left = lower_core_expr_to_typescript(left, imported_values, imported_types);
    let right = lower_core_expr_to_typescript(right, imported_values, imported_types);
    if operator == ":" {
        return TypeScriptExpr::TypeApplicationCall {
            callee: crate::list_ops::runtime_list_cons_operation()
                .local_name
                .to_owned(),
            type_arguments: vec![list_cons_element_type(&type_ref, imported_types)],
            arguments: vec![left, right],
        };
    }
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
    if standard_operator(&operator)
        .is_some_and(|operator| operator.kind == StandardOperatorKind::Comparison)
    {
        let dictionary = evidence
            .first()
            .and_then(|selected| {
                selected_dictionary_expression(&selected, imported_values, imported_types)
            })
            .expect("comparison requires materialized Ord evidence");
        return comparison_result(&operator, dictionary, left, right);
    }
    if matches!(operator.as_str(), "==" | "!=") {
        if let Some(selected) = evidence.first().and_then(|selected| {
            selected_dictionary_expression(&selected, imported_values, imported_types)
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
        if let Some(operator) = strict_equality_operator_with_evidence(&operator, &evidence) {
            return TypeScriptExpr::Binary {
                operator: operator.to_owned(),
                left: Box::new(left),
                right: Box::new(right),
            };
        }
    }
    if let Some(method) = operator_trait_method(&operator) {
        if let Some(selected) = evidence.first().and_then(|selected| {
            selected_dictionary_expression(&selected, imported_values, imported_types)
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

// Applying the first operand invokes the selected method's first stage now.
// Its body may run before the second operand, including for a saved section.
fn lower_comparison_call(
    operator: &str,
    dictionary: TypeScriptExpr,
    mut arguments: Vec<TypeScriptExpr>,
    result_type: &CoreType,
    imported_types: &TypeScriptTypeContext,
    origin: crate::SourceSpan,
) -> TypeScriptExpr {
    if arguments.len() == 2 {
        let right = arguments.pop().unwrap();
        return comparison_result(operator, dictionary, arguments.pop().unwrap(), right);
    }
    let CoreType::Function { parameter, result } = result_type else {
        unreachable!("checked comparison partial type")
    };
    let parameter_type = type_ref_from_core_type(parameter, imported_types);
    if arguments.is_empty() {
        let left = "__ssrg$comparison$left".to_owned();
        return TypeScriptExpr::Lambda {
            parameter: format!("{left}: {}", render_typescript_type(&parameter_type)),
            body: Box::new(lower_comparison_call(
                operator,
                dictionary,
                vec![TypeScriptExpr::Identifier { name: left }],
                result,
                imported_types,
                origin,
            )),
        };
    }
    let name = "__ssrg$comparison$stage".to_owned();
    let right = "__ssrg$comparison$right".to_owned();
    let stage_type = super::TypeScriptType::Function {
        parameter: Box::new(parameter_type.clone()),
        result: Box::new(super::TypeScriptType::Ordering),
    };
    TypeScriptExpr::Sequence {
        statements: vec![TypeScriptStatement::PureLet {
            type_parameters: vec![],
            name: name.clone(),
            type_ref: stage_type,
            initializer: TypeScriptExpr::DictionaryCall {
                dictionary: Box::new(dictionary),
                method: "compare".to_owned(),
                arguments,
            },
            initializer_in_outer_scope: false,
            origin,
        }],
        result: Box::new(TypeScriptExpr::Lambda {
            parameter: format!("{right}: {}", render_typescript_type(&parameter_type)),
            body: Box::new(comparison_ordering_result(
                operator,
                TypeScriptExpr::Call {
                    callee: name,
                    arguments: vec![TypeScriptExpr::Identifier { name: right }],
                    type_arguments: Vec::new(),
                    checked_callee_type: None,
                },
            )),
        }),
    }
}

/// Inspect the canonical Ordering tag once; operands and compare each execute once.
fn comparison_result(
    operator: &str,
    dictionary: TypeScriptExpr,
    left: TypeScriptExpr,
    right: TypeScriptExpr,
) -> TypeScriptExpr {
    comparison_ordering_result(
        operator,
        TypeScriptExpr::DictionaryCall {
            dictionary: Box::new(dictionary),
            method: "compare".to_owned(),
            arguments: vec![left, right],
        },
    )
}

fn comparison_ordering_result(operator: &str, ordering: TypeScriptExpr) -> TypeScriptExpr {
    let (test, tag) = match operator {
        "<" => ("===", "Less"),
        "<=" => ("!==", "Greater"),
        ">" => ("===", "Greater"),
        ">=" => ("!==", "Less"),
        _ => unreachable!("comparison metadata must identify an Ord operator"),
    };
    TypeScriptExpr::Binary {
        operator: test.to_owned(),
        left: Box::new(TypeScriptExpr::FieldAccess {
            receiver: Box::new(ordering),
            field: "tag".to_owned(),
        }),
        right: Box::new(TypeScriptExpr::String {
            value: tag.to_owned(),
        }),
    }
}

// Preserve source-selected operand types when TypeScript has no contextual
// function type (returned sections) or would infer generics too early.
fn checked_operator_reference(
    value: TypeScriptExpr,
    type_ref: &CoreType,
    imported_types: &TypeScriptTypeContext,
) -> TypeScriptExpr {
    TypeScriptExpr::CheckedResult {
        value: Box::new(value),
        type_ref: type_ref_from_core_type(type_ref, imported_types),
    }
}

fn operator_trait_method(operator: &str) -> Option<&'static str> {
    standard_operator(operator)
        .filter(|operator| operator.kind == StandardOperatorKind::Arithmetic)
        .map(|operator| operator.method_name)
}

fn curried_dictionary_method_reference(dictionary: TypeScriptExpr, method: &str) -> TypeScriptExpr {
    // The selected method already has the language's curried stages. An
    // eta-expanded two-argument wrapper would delay a one-parameter method's
    // body until its returned function receives the second source operand.
    TypeScriptExpr::DictionaryCall {
        dictionary: Box::new(dictionary),
        method: method.to_owned(),
        arguments: vec![],
    }
}

fn equality_reference_from_evidence(
    evidence: &[crate::CoreCallEvidence],
    imported_values: &BTreeMap<String, String>,
    imported_types: &TypeScriptTypeContext,
) -> Option<TypeScriptExpr> {
    let [selected] = evidence else {
        return None;
    };
    if selected.constraint.name != "Eq" {
        return None;
    }
    if let Some(dictionary) =
        selected_dictionary_expression(&selected, imported_values, imported_types)
    {
        return Some(curried_dictionary_method_reference(dictionary, "eq"));
    }
    strict_equality_operator_with_evidence("==", evidence).map(curried_binary_reference)
}

fn lower_trait_operator_call(
    dictionary: TypeScriptExpr,
    operator: &StandardTraitOperator,
    supplied_source_arguments: Vec<TypeScriptExpr>,
    method_evidence: Vec<TypeScriptExpr>,
) -> TypeScriptExpr {
    assert!(supplied_source_arguments.len() <= operator.method_operand_sources.len());
    let supplied_count = supplied_source_arguments.len();
    let mut source_arguments = [None, None];
    for (source_index, argument) in supplied_source_arguments.into_iter().enumerate() {
        source_arguments[source_index] = Some(argument);
    }
    for source_index in supplied_count..source_arguments.len() {
        source_arguments[source_index] = Some(TypeScriptExpr::Identifier {
            name: format!("__ssrg$operator$argument${source_index}"),
        });
    }
    let mut method_arguments = operator
        .method_operand_sources
        .map(|source_index| {
            source_arguments[source_index]
                .take()
                .expect("trait operator order must be a permutation")
        })
        .to_vec();
    method_arguments.extend(method_evidence);
    let mut result = TypeScriptExpr::DictionaryCall {
        dictionary: Box::new(dictionary),
        method: operator.method_name.to_owned(),
        arguments: method_arguments,
    };
    for source_index in (supplied_count..source_arguments.len()).rev() {
        result = TypeScriptExpr::Lambda {
            parameter: format!("__ssrg$operator$argument${source_index}"),
            body: Box::new(result),
        };
    }
    result
}

fn negated_equality_reference(
    dictionary: TypeScriptExpr,
    type_ref: &CoreType,
    imported_types: &TypeScriptTypeContext,
    origin: crate::SourceSpan,
) -> TypeScriptExpr {
    let CoreType::Function {
        parameter: left_type,
        result: stage_type,
    } = type_ref
    else {
        unreachable!("checked equality reference must have two source operands")
    };
    let CoreType::Function {
        parameter: right_type,
        ..
    } = stage_type.as_ref()
    else {
        unreachable!("checked equality reference must return its right source stage")
    };
    let method = "__ssrg$equality$method".to_owned();
    let stage = "__ssrg$equality$stage".to_owned();
    let left = "__ssrg$equality$left".to_owned();
    let right = "__ssrg$equality$right".to_owned();
    TypeScriptExpr::Sequence {
        statements: vec![TypeScriptStatement::PureLet {
            type_parameters: vec![],
            name: method.clone(),
            type_ref: type_ref_from_core_type(type_ref, imported_types),
            initializer: curried_dictionary_method_reference(dictionary, "eq"),
            initializer_in_outer_scope: false,
            origin: origin.clone(),
        }],
        result: Box::new(TypeScriptExpr::Lambda {
            parameter: format!(
                "{left}: {}",
                render_typescript_type(&type_ref_from_core_type(left_type, imported_types))
            ),
            body: Box::new(TypeScriptExpr::Sequence {
                statements: vec![TypeScriptStatement::PureLet {
                    type_parameters: vec![],
                    name: stage.clone(),
                    type_ref: type_ref_from_core_type(stage_type, imported_types),
                    initializer: TypeScriptExpr::Call {
                        callee: method,
                        arguments: vec![TypeScriptExpr::Identifier { name: left }],
                        type_arguments: Vec::new(),
                        checked_callee_type: None,
                    },
                    initializer_in_outer_scope: false,
                    origin,
                }],
                result: Box::new(TypeScriptExpr::Lambda {
                    parameter: format!(
                        "{right}: {}",
                        render_typescript_type(&type_ref_from_core_type(
                            right_type,
                            imported_types
                        ))
                    ),
                    body: Box::new(TypeScriptExpr::Binary {
                        operator: "===".to_owned(),
                        left: Box::new(TypeScriptExpr::Call {
                            callee: stage,
                            arguments: vec![TypeScriptExpr::Identifier { name: right }],
                            type_arguments: Vec::new(),
                            checked_callee_type: None,
                        }),
                        right: Box::new(TypeScriptExpr::Boolean { value: false }),
                    }),
                }),
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
    imported_types: &TypeScriptTypeContext,
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
    imported_types: &TypeScriptTypeContext,
) -> TypeScriptStatement {
    match statement {
        CoreStatement::Effect { value } => TypeScriptStatement::Effect {
            value: lower_core_expr_to_typescript(value, imported_values, imported_types),
        },
        CoreStatement::PureLet {
            type_parameters,
            name,
            type_ref,
            value,
            origin,
        } => TypeScriptStatement::PureLet {
            // Non-recursive let initializers resolve before introducing the
            // new binding. A JS const would otherwise capture outer references
            // with the same name in its temporal dead zone, including lambdas.
            initializer_in_outer_scope: crate::core_free_variables(&value).map_or(true, |free| {
                free.names.iter().any(|reference| {
                    imported_values
                        .get(reference)
                        .cloned()
                        .unwrap_or_else(|| local_name(reference))
                        == safe_identifier(&name)
                })
            }),
            type_ref: type_ref_from_core_type(
                &type_ref,
                &imported_types.with_parameters(&type_parameters),
            ),
            initializer: lower_core_expr_to_typescript(
                value,
                imported_values,
                &imported_types.with_parameters(&type_parameters),
            ),
            type_parameters,
            name: safe_identifier(&name),
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
        CoreStatement::LocalFunction {
            initializer_in_outer_scope,
            rec_group,
            return_type,
            name,
            type_parameters,
            constraints,
            parameters,
            body,
            origin,
        } => {
            let scoped_types = imported_types.with_parameters(&type_parameters);
            let imported_types = &scoped_types;
            let type_constructor_parameters = type_parameters
                .iter()
                .filter(|parameter| parameter.is_constructor())
                .map(|parameter| parameter.name.clone())
                .collect::<Vec<_>>();
            TypeScriptStatement::LocalFunction {
                initializer_in_outer_scope,
                rec_group,
                return_type: return_type
                    .as_ref()
                    .map(|ty| type_ref_from_core_type(ty, imported_types)),
                name: safe_identifier(&name),
                type_parameters,
                constraints: constraints
                    .into_iter()
                    .map(|constraint| super::TypeScriptInstanceConstraint {
                        name: constraint.name,
                        trait_identity: constraint.trait_identity,
                        arguments: constraint
                            .arguments
                            .iter()
                            .map(|argument| type_ref_from_core_type(argument, imported_types))
                            .collect(),
                    })
                    .collect(),
                parameters: parameters
                    .into_iter()
                    .map(|parameter| {
                        lower_core_parameter_to_typescript(
                            parameter,
                            imported_types,
                            &type_constructor_parameters,
                        )
                    })
                    .collect(),
                body: lower_core_expr_to_typescript(body, imported_values, imported_types),
                origin,
            }
        }
    }
}

fn lower_monad_do_statement(
    statement: CoreMonadDoStatement,
    imported_values: &BTreeMap<String, String>,
    imported_types: &TypeScriptTypeContext,
) -> TypeScriptStatement {
    match statement {
        CoreMonadDoStatement::Expression { value } => TypeScriptStatement::Effect {
            value: lower_core_expr_to_typescript(value, imported_values, imported_types),
        },
        CoreMonadDoStatement::PureLet {
            type_parameters,
            constraints,
            name,
            type_ref,
            mut value,
            origin,
        } => {
            let statement = if constraints.is_empty() {
                CoreStatement::PureLet {
                    type_parameters,
                    name,
                    type_ref,
                    value,
                    origin,
                }
            } else {
                let mut parameters = Vec::new();
                while let CoreExpr::Lambda {
                    parameter, body, ..
                } = value
                {
                    parameters.push(parameter);
                    value = *body;
                }
                CoreStatement::LocalFunction {
                    initializer_in_outer_scope: true,
                    rec_group: None,
                    return_type: None,
                    name,
                    type_parameters,
                    constraints,
                    parameters,
                    body: value,
                    origin,
                }
            };
            lower_core_statement_to_typescript(statement, imported_values, imported_types)
        }
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
        TypeScriptStatement::LocalFunction { body, .. } => typescript_expr_contains_await(body),
    }
}

// Reify a complete ordinary source instantiation at the first curried stage.
// TypeScript otherwise fixes parameters before later arguments and dictionaries
// can infer them. Higher-kinded calls retain their checked erasure path.
fn ordinary_call_type_arguments(
    signature: Option<&crate::CoreCallSignature>,
    imported_types: &TypeScriptTypeContext,
) -> Vec<super::TypeScriptType> {
    signature
        .filter(|signature| {
            !signature.type_parameters.is_empty()
                && signature
                    .type_parameters
                    .iter()
                    .all(|p| !p.is_constructor())
                && signature.instantiated_type_arguments.len() == signature.type_parameters.len()
        })
        .map(|signature| {
            signature
                .instantiated_type_arguments
                .iter()
                .map(|ty| type_ref_from_core_type(ty, imported_types))
                .collect()
        })
        .unwrap_or_default()
}
