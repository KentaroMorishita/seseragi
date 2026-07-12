use crate::{
    effect_ops::{known_effect_operation_by_surface, KnownEffectOperation},
    unit_type, SymbolKind, SymbolNamespace, TypedDoStatement, TypedExpr, TypedParameter,
};
use seseragi_syntax::{ByteSpan, SurfaceDoItem, SurfaceExpr, SurfacePattern};
use std::collections::BTreeMap;

use super::pure_issues::PureCallIssue;
use super::semantic_types::SemanticValueType;
use super::surface_expr::{analyze_resolved_expression, PureExpressionContext};
use super::type_ref::inferred_type_from_expr;
use super::TypedResolution;

mod imported;
mod operation_contract;

use operation_contract::operation_effect;

pub(crate) struct EffectBodyAnalysis {
    pub(crate) value: TypedExpr,
    pub(crate) call_issues: Vec<PureCallIssue>,
}

pub(crate) fn analyze_effect_body(
    body: &SurfaceExpr,
    parameters: &[TypedParameter],
    resolution: &TypedResolution<'_>,
) -> EffectBodyAnalysis {
    let context = PureExpressionContext::new(parameters, resolution);
    let mut call_issues = Vec::new();
    let value = type_effect_expression(body, &context, resolution, &mut call_issues);
    EffectBodyAnalysis { value, call_issues }
}

pub(crate) fn typed_effect_body(
    body: &SurfaceExpr,
    parameters: &[TypedParameter],
    resolution: &TypedResolution<'_>,
) -> TypedExpr {
    analyze_effect_body(body, parameters, resolution).value
}

fn type_effect_expression(
    expression: &SurfaceExpr,
    context: &PureExpressionContext<'_>,
    resolution: &TypedResolution<'_>,
    call_issues: &mut Vec<PureCallIssue>,
) -> TypedExpr {
    if let SurfaceExpr::Grouped { value, .. } = expression {
        return type_effect_expression(value, context, resolution, call_issues);
    }

    if let Some((operation, arguments)) =
        effect_application(expression, context, resolution, call_issues)
    {
        let effect = operation_effect(operation, &arguments);
        return TypedExpr::EffectCall {
            operation: operation.semantic_name.to_owned(),
            effect,
            arguments,
            origin: expression.span(),
        };
    }

    if let Some(value) =
        imported::type_imported_effect_application(expression, context, resolution, call_issues)
    {
        return value;
    }

    if let SurfaceExpr::Do {
        items,
        result,
        span,
    } = expression
    {
        return type_do_block(
            items,
            result.as_deref(),
            *span,
            context,
            resolution,
            call_issues,
        );
    }

    analyze_resolved_expression(expression, context).value
}

fn effect_application(
    expression: &SurfaceExpr,
    context: &PureExpressionContext<'_>,
    resolution: &TypedResolution<'_>,
    call_issues: &mut Vec<PureCallIssue>,
) -> Option<(KnownEffectOperation, Vec<TypedExpr>)> {
    let (callee, argument_nodes) = flatten_application(expression);
    let SurfaceExpr::Name { span, .. } = callee else {
        return None;
    };
    let target = resolution.target(*span, SymbolNamespace::Value)?;
    let symbol = resolution.symbol(target)?;
    if symbol.kind != SymbolKind::Prelude {
        return None;
    }
    let operation = known_effect_operation_by_surface(&symbol.spelling)?;
    if operation.surface_name == "mapError" && argument_nodes.len() != 2 {
        return None;
    }
    let mut arguments = if operation.surface_name == "mapError" {
        vec![
            analyze_resolved_expression(argument_nodes[0], context).value,
            type_effect_expression(argument_nodes[1], context, resolution, call_issues),
        ]
    } else {
        argument_nodes
            .into_iter()
            .map(|argument| analyze_resolved_expression(argument, context).value)
            .collect::<Vec<_>>()
    };
    if matches!(operation.surface_name, "readLine" | "succeed")
        && matches!(arguments.as_slice(), [TypedExpr::Unit { .. }])
    {
        arguments.clear();
    }
    Some((operation, arguments))
}

fn type_do_block(
    items: &[SurfaceDoItem],
    result: Option<&SurfaceExpr>,
    origin: ByteSpan,
    base_context: &PureExpressionContext<'_>,
    resolution: &TypedResolution<'_>,
    call_issues: &mut Vec<PureCallIssue>,
) -> TypedExpr {
    let mut locals = BTreeMap::new();
    let mut statements = Vec::new();

    for item in items {
        let context = base_context.with_locals(locals.clone());
        match item {
            SurfaceDoItem::Expression { value, .. } => {
                statements.push(TypedDoStatement::Effect {
                    value: type_effect_expression(value, &context, resolution, call_issues),
                });
            }
            SurfaceDoItem::Bind {
                pattern,
                value,
                span,
            } => {
                let value = type_effect_expression(value, &context, resolution, call_issues);
                let type_ref = inferred_type_from_expr(&value);
                if let Some((symbol, name)) = binding(pattern, resolution) {
                    locals.insert(symbol, resolution.semantic_value_from_typed_type(&type_ref));
                    statements.push(TypedDoStatement::Bind {
                        name,
                        type_ref,
                        value,
                        origin: *span,
                    });
                } else {
                    statements.push(TypedDoStatement::Effect { value });
                }
            }
            SurfaceDoItem::Let {
                pattern,
                value,
                span,
            } => {
                let analysis = analyze_resolved_expression(value, &context);
                let type_ref = inferred_type_from_expr(&analysis.value);
                if let Some((symbol, name)) = binding(pattern, resolution) {
                    locals.insert(
                        symbol,
                        SemanticValueType {
                            type_ref: type_ref.clone(),
                            key: analysis.semantic_type,
                        },
                    );
                    statements.push(TypedDoStatement::PureLet {
                        name,
                        type_ref,
                        value: analysis.value,
                        origin: *span,
                    });
                }
            }
        }
    }

    let context = base_context.with_locals(locals);
    let result = result
        .map(|result| type_effect_expression(result, &context, resolution, call_issues))
        .unwrap_or_else(|| TypedExpr::Unit {
            type_ref: unit_type(),
            origin: insertion_point(origin),
        });
    TypedExpr::DoBlock {
        statements,
        result: Box::new(result),
        origin,
    }
}

fn binding(
    pattern: &SurfacePattern,
    resolution: &TypedResolution<'_>,
) -> Option<(crate::SymbolId, String)> {
    let SurfacePattern::Name {
        name, name_span, ..
    } = pattern
    else {
        return None;
    };
    let symbol = resolution.declaration_symbol(*name_span, SymbolKind::PatternBinding)?;
    Some((symbol.id, name.clone()))
}

fn flatten_application(expression: &SurfaceExpr) -> (&SurfaceExpr, Vec<&SurfaceExpr>) {
    let mut callee = expression;
    let mut arguments = Vec::new();
    while let SurfaceExpr::Application {
        function, argument, ..
    } = callee
    {
        arguments.push(argument.as_ref());
        callee = function.as_ref();
    }
    arguments.reverse();
    (callee, arguments)
}

fn insertion_point(origin: ByteSpan) -> ByteSpan {
    let point = origin.end.saturating_sub(1);
    ByteSpan {
        start: point,
        end: point,
    }
}
