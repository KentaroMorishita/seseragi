use std::collections::BTreeMap;

use crate::{SymbolId, SymbolKind, TypedBlockStatement, TypedConstraint, TypedExpr, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceBlockItem, SurfaceExpr};

use super::{type_surface_expression, PureExpressionContext, SurfaceExpressionAnalysis};
use crate::typed::functions::typed_parameters_from_surface;
use crate::typed::pure_issues::PureCallIssue;
use crate::typed::semantic_types::{
    semantic_values_are_compatible, SemanticTypeKey, SemanticValueType,
};
use crate::typed::type_ref::{inferred_type_from_expr, typed_type_contains_hole};

pub(super) fn type_block(
    items: &[SurfaceBlockItem],
    result: &SurfaceExpr,
    origin: ByteSpan,
    base_context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    let mut locals = BTreeMap::<SymbolId, SemanticValueType>::new();
    let mut statements = Vec::new();
    let mut merged = SurfaceExpressionAnalysis::valid_with_semantic_type(
        TypedExpr::Unit {
            type_ref: named_type("Unit"),
            origin,
        },
        SemanticTypeKey::Other,
    );

    for item in items {
        match item {
            SurfaceBlockItem::Let {
                name,
                name_span,
                type_ref,
                value,
                span,
            } => {
                let expected = type_ref
                    .as_ref()
                    .map(|type_ref| base_context.semantic_value_from_type_ref(type_ref));
                let context = base_context
                    .with_locals(locals.clone())
                    .with_expected(expected.clone());
                let analysis = type_surface_expression(value, &context);
                let actual = SemanticValueType {
                    type_ref: inferred_type_from_expr(&analysis.value),
                    key: analysis.semantic_type.clone(),
                };
                if let Some(expected) = expected {
                    if !typed_type_contains_hole(&expected.type_ref)
                        && !typed_type_contains_hole(&actual.type_ref)
                        && !semantic_values_are_compatible(&expected, &actual)
                    {
                        merged.pure_call_issue = merged.pure_call_issue.take().or(Some(
                            PureCallIssue::LocalBindingTypeMismatch {
                                binding: *name_span,
                                expected: expected.type_ref,
                                actual: actual.type_ref.clone(),
                            },
                        ));
                    }
                }
                if let Some(symbol) = base_context.declaration_symbol(*name_span, SymbolKind::Let) {
                    locals.insert(symbol, base_context.hydrate_semantic_value(actual.clone()));
                }
                statements.push(TypedBlockStatement::Let {
                    name: name.clone(),
                    type_ref: actual.type_ref,
                    value: analysis.value.clone(),
                    origin: *span,
                });
                merged.merge_issues_from(analysis);
            }
            SurfaceBlockItem::Function {
                name,
                name_span,
                type_parameters,
                parameters,
                return_type,
                constraints,
                value,
                span,
            } => {
                let typed_parameters =
                    typed_parameters_from_surface(parameters, base_context.resolution);
                let expected = base_context.semantic_value_from_type_ref(return_type);
                let function_type = function_type(&typed_parameters, expected.type_ref.clone());
                let mut function_locals = locals.clone();
                if let Some(symbol) =
                    base_context.declaration_symbol(*name_span, SymbolKind::Function)
                {
                    function_locals.insert(
                        symbol,
                        SemanticValueType {
                            type_ref: function_type.clone(),
                            key: SemanticTypeKey::Other,
                        },
                    );
                    locals.insert(
                        symbol,
                        SemanticValueType {
                            type_ref: function_type,
                            key: SemanticTypeKey::Other,
                        },
                    );
                }
                function_locals.extend(base_context.parameter_locals(&typed_parameters));
                let scoped_evidence =
                    crate::typed::scoped_call_evidence(constraints, base_context.resolution);
                let mut context = base_context
                    .with_locals(function_locals)
                    .with_expected(Some(expected.clone()));
                if !constraints.is_empty() {
                    context = context.with_evidence_parameters(scoped_evidence);
                }
                let analysis = type_surface_expression(value, &context);
                let actual = SemanticValueType {
                    type_ref: inferred_type_from_expr(&analysis.value),
                    key: analysis.semantic_type.clone(),
                };
                if !typed_type_contains_hole(&expected.type_ref)
                    && !typed_type_contains_hole(&actual.type_ref)
                    && !semantic_values_are_compatible(&expected, &actual)
                {
                    merged.pure_call_issue = merged.pure_call_issue.take().or(Some(
                        PureCallIssue::LocalFunctionBodyTypeMismatch {
                            body: value.span(),
                            expected: expected.type_ref,
                            actual: actual.type_ref,
                        },
                    ));
                }
                statements.push(TypedBlockStatement::Function {
                    name: name.clone(),
                    type_parameters: type_parameters
                        .iter()
                        .map(|parameter| parameter.name.clone())
                        .collect(),
                    type_constructor_parameters: type_parameters
                        .iter()
                        .filter(|parameter| parameter.is_constructor())
                        .map(|parameter| parameter.name.clone())
                        .collect(),
                    constraints: constraints
                        .iter()
                        .map(|constraint| TypedConstraint {
                            name: constraint.name.clone(),
                            arguments: constraint
                                .arguments
                                .iter()
                                .map(|argument| {
                                    base_context.semantic_value_from_type_ref(argument).type_ref
                                })
                                .collect(),
                        })
                        .collect(),
                    parameters: typed_parameters,
                    body: analysis.value.clone(),
                    origin: *span,
                });
                merged.merge_issues_from(analysis);
            }
        }
    }

    let result_analysis = type_surface_expression(
        result,
        &base_context
            .with_locals(locals)
            .with_expected(base_context.expected().cloned()),
    );
    let type_ref = inferred_type_from_expr(&result_analysis.value);
    let semantic_type = result_analysis.semantic_type.clone();
    let result_value = result_analysis.value.clone();
    merged.merge_issues_from(result_analysis);
    merged.value = TypedExpr::Block {
        statements,
        result: Box::new(result_value),
        type_ref,
        origin,
    };
    merged.semantic_type = semantic_type;
    merged
}

fn function_type(parameters: &[crate::TypedParameter], result: TypedType) -> TypedType {
    parameters.iter().rev().fold(result, |result, parameter| {
        let parameter = match parameter {
            crate::TypedParameter::ImplicitUnit { type_ref }
            | crate::TypedParameter::Named { type_ref, .. } => type_ref.clone(),
        };
        TypedType::Function {
            parameter: Box::new(parameter),
            result: Box::new(result),
        }
    })
}

fn named_type(name: &str) -> TypedType {
    TypedType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}
