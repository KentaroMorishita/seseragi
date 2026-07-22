use crate::{
    effect_ops::{known_effect_operation_by_surface, KnownEffectOperation},
    unit_type, SymbolKind, SymbolNamespace, TypedDoStatement, TypedExpr, TypedParameter,
};
use seseragi_syntax::{ByteSpan, SurfaceDoItem, SurfaceExpr, SurfacePattern};
use std::collections::BTreeMap;

use super::pure_issues::{ArrayIssue, PureCallIssue, RangeIssue, RecordIssue};
use super::semantic_types::SemanticValueType;
use super::surface_expr::{
    analyze_resolved_expression, application, ensure_recovery_hole_issue, PureExpressionContext,
    SurfaceExpressionAnalysis,
};
use super::type_ref::{
    application_argument_type_from_expr, effect_success_type_from_expr, inferred_type_from_expr,
};
use super::TypedResolution;

mod imported;
mod operation_contract;

use operation_contract::operation_effect;

pub(crate) struct EffectBodyAnalysis {
    pub(crate) value: TypedExpr,
    pub(crate) call_issues: Vec<PureCallIssue>,
    pub(crate) array_issues: Vec<ArrayIssue>,
    pub(crate) record_issues: Vec<RecordIssue>,
    pub(crate) range_issues: Vec<RangeIssue>,
}

pub(super) struct EffectBodyIssues<'a> {
    pub(super) calls: &'a mut Vec<PureCallIssue>,
    pub(super) arrays: &'a mut Vec<ArrayIssue>,
    pub(super) records: &'a mut Vec<RecordIssue>,
    pub(super) ranges: &'a mut Vec<RangeIssue>,
}

pub(crate) fn analyze_effect_body(
    body: &SurfaceExpr,
    parameters: &[TypedParameter],
    resolution: &TypedResolution<'_>,
) -> EffectBodyAnalysis {
    let context = PureExpressionContext::new(parameters, resolution);
    let mut call_issues = Vec::new();
    let mut array_issues = Vec::new();
    let mut record_issues = Vec::new();
    let mut range_issues = Vec::new();
    let value = type_effect_expression(
        body,
        &context,
        resolution,
        &mut EffectBodyIssues {
            calls: &mut call_issues,
            arrays: &mut array_issues,
            records: &mut record_issues,
            ranges: &mut range_issues,
        },
    );
    let mut final_analysis = SurfaceExpressionAnalysis::valid(value);
    if call_issues.is_empty()
        && array_issues.is_empty()
        && record_issues.is_empty()
        && range_issues.is_empty()
    {
        ensure_recovery_hole_issue(&mut final_analysis);
        if let Some(issue) = final_analysis.pure_call_issue {
            call_issues.push(issue);
        }
    }
    EffectBodyAnalysis {
        value: final_analysis.value,
        call_issues,
        array_issues,
        record_issues,
        range_issues,
    }
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
    issues: &mut EffectBodyIssues<'_>,
) -> TypedExpr {
    if let SurfaceExpr::Grouped { value, .. } = expression {
        return type_effect_expression(value, context, resolution, issues);
    }

    if let Some((operation, arguments)) =
        effect_application(expression, context, resolution, issues)
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
        imported::type_imported_effect_application(expression, context, resolution, issues)
    {
        return value;
    }

    if let SurfaceExpr::Do {
        items,
        result,
        span,
    } = expression
    {
        return type_do_block(items, result.as_deref(), *span, context, resolution, issues);
    }

    if matches!(expression, SurfaceExpr::Application { .. }) {
        let analysis = application::type_application_with(
            expression,
            context,
            |argument, argument_context| {
                let value = type_effect_expression(argument, argument_context, resolution, issues);
                let semantic_type = argument_context
                    .semantic_value_from_typed_type(&application_argument_type_from_expr(&value))
                    .key;
                SurfaceExpressionAnalysis::valid_with_semantic_type(value, semantic_type)
            },
        );
        return finish_expression_analysis(analysis, issues);
    }

    type_pure_expression(expression, context, issues)
}

fn effect_application(
    expression: &SurfaceExpr,
    context: &PureExpressionContext<'_>,
    resolution: &TypedResolution<'_>,
    issues: &mut EffectBodyIssues<'_>,
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
            type_pure_expression(argument_nodes[0], context, issues),
            type_effect_expression(argument_nodes[1], context, resolution, issues),
        ]
    } else {
        argument_nodes
            .into_iter()
            .map(|argument| type_pure_expression(argument, context, issues))
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
    issues: &mut EffectBodyIssues<'_>,
) -> TypedExpr {
    let mut locals = BTreeMap::new();
    let mut statements = Vec::new();

    for item in items {
        let context = base_context.with_locals(locals.clone());
        match item {
            SurfaceDoItem::Expression { value, .. } => {
                statements.push(TypedDoStatement::Effect {
                    value: type_effect_expression(value, &context, resolution, issues),
                });
            }
            SurfaceDoItem::Bind {
                pattern,
                value,
                span,
            } => {
                let value = type_effect_expression(value, &context, resolution, issues);
                let type_ref = effect_success_type_from_expr(&value);
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
                if let Some(issue) = analysis.array_issue.clone() {
                    issues.arrays.push(issue);
                }
                if let Some(issue) = analysis.record_issue.clone() {
                    issues.records.push(issue);
                }
                if let Some(issue) = analysis.range_issue.clone() {
                    issues.ranges.push(issue);
                }
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
        .map(|result| type_effect_expression(result, &context, resolution, issues))
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

fn type_pure_expression(
    expression: &SurfaceExpr,
    context: &PureExpressionContext<'_>,
    issues: &mut EffectBodyIssues<'_>,
) -> TypedExpr {
    finish_expression_analysis(analyze_resolved_expression(expression, context), issues)
}

fn finish_expression_analysis(
    analysis: SurfaceExpressionAnalysis,
    issues: &mut EffectBodyIssues<'_>,
) -> TypedExpr {
    if let Some(issue) = analysis.array_issue {
        issues.arrays.push(issue);
    }
    if let Some(issue) = analysis.record_issue {
        issues.records.push(issue);
    }
    if let Some(issue) = analysis.range_issue {
        issues.ranges.push(issue);
    }
    if let Some(issue) = analysis.pure_call_issue {
        issues.calls.push(issue);
    }
    analysis.value
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
