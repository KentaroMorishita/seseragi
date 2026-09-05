use crate::{
    effect_ops::{known_effect_operation_by_surface, KnownEffectOperation},
    unit_type, SymbolKind, SymbolNamespace, TypedDoStatement, TypedExpr, TypedParameter,
    TypedTemplatePart,
};
use seseragi_syntax::{ByteSpan, SurfaceDoItem, SurfaceExpr};
use std::collections::BTreeMap;

use super::pure_issues::{
    ArrayIssue, ConditionalIssue, MatchIssue, PureCallIssue, RangeIssue, RecordIssue,
};
use super::surface_expr::{
    analyze_resolved_expression, application, ensure_recovery_hole_issue, named_type,
    PureExpressionContext, SurfaceExpressionAnalysis,
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
    pub(crate) conditional_issues: Vec<ConditionalIssue>,
    pub(crate) call_issues: Vec<PureCallIssue>,
    pub(crate) array_issues: Vec<ArrayIssue>,
    pub(crate) record_issues: Vec<RecordIssue>,
    pub(crate) range_issues: Vec<RangeIssue>,
    pub(crate) pattern_issues: Vec<MatchIssue>,
}

pub(super) struct EffectBodyIssues<'a> {
    pub(super) conditionals: &'a mut Vec<ConditionalIssue>,
    pub(super) calls: &'a mut Vec<PureCallIssue>,
    pub(super) arrays: &'a mut Vec<ArrayIssue>,
    pub(super) records: &'a mut Vec<RecordIssue>,
    pub(super) ranges: &'a mut Vec<RangeIssue>,
    pub(super) patterns: &'a mut Vec<MatchIssue>,
}

pub(crate) fn analyze_effect_body(
    body: &SurfaceExpr,
    parameters: &[TypedParameter],
    resolution: &TypedResolution<'_>,
    evidence_parameters: Vec<super::call_evidence::ScopedCallEvidence>,
) -> EffectBodyAnalysis {
    let context = PureExpressionContext::new(parameters, resolution)
        .with_evidence_parameters(evidence_parameters);
    let mut conditional_issues = Vec::new();
    let mut call_issues = Vec::new();
    let mut array_issues = Vec::new();
    let mut record_issues = Vec::new();
    let mut range_issues = Vec::new();
    let mut pattern_issues = Vec::new();
    let value = type_effect_expression(
        body,
        &context,
        resolution,
        &mut EffectBodyIssues {
            conditionals: &mut conditional_issues,
            calls: &mut call_issues,
            arrays: &mut array_issues,
            records: &mut record_issues,
            ranges: &mut range_issues,
            patterns: &mut pattern_issues,
        },
    );
    let mut final_analysis = SurfaceExpressionAnalysis::valid(value);
    if conditional_issues.is_empty()
        && call_issues.is_empty()
        && array_issues.is_empty()
        && record_issues.is_empty()
        && range_issues.is_empty()
        && pattern_issues.is_empty()
    {
        ensure_recovery_hole_issue(&mut final_analysis);
        if let Some(issue) = final_analysis.pure_call_issue {
            call_issues.push(issue);
        }
    }
    EffectBodyAnalysis {
        value: final_analysis.value,
        conditional_issues,
        call_issues,
        array_issues,
        record_issues,
        range_issues,
        pattern_issues,
    }
}

pub(crate) fn typed_effect_body(
    body: &SurfaceExpr,
    parameters: &[TypedParameter],
    resolution: &TypedResolution<'_>,
    evidence_parameters: Vec<super::call_evidence::ScopedCallEvidence>,
) -> TypedExpr {
    analyze_effect_body(body, parameters, resolution, evidence_parameters).value
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

    if let SurfaceExpr::Lambda {
        parameter,
        body,
        span,
    } = expression
    {
        let analysis = super::surface_expr::lambda::type_lambda_with(
            parameter,
            body,
            *span,
            context,
            |body, body_context| {
                let value = type_effect_expression(body, body_context, resolution, issues);
                let semantic_type = body_context
                    .semantic_value_from_typed_type(&application_argument_type_from_expr(&value))
                    .key;
                SurfaceExpressionAnalysis::valid_with_semantic_type(value, semantic_type)
            },
        );
        return finish_expression_analysis(analysis, issues);
    }

    if let SurfaceExpr::EffectfulFor {
        pattern,
        source,
        body,
        span,
    } = expression
    {
        let analysis = super::surface_expr::effectful_for::type_effectful_for_with(
            pattern,
            source,
            body,
            *span,
            context,
            |body, body_context| {
                let value = type_effect_expression(body, body_context, resolution, issues);
                let semantic_type = body_context
                    .semantic_value_from_typed_type(&application_argument_type_from_expr(&value))
                    .key;
                SurfaceExpressionAnalysis::valid_with_semantic_type(value, semantic_type)
            },
        );
        return finish_expression_analysis(analysis, issues);
    }

    let mut type_branch = |body: &SurfaceExpr, branch_context: &PureExpressionContext<'_>| {
        let value = type_effect_expression(body, branch_context, resolution, issues);
        let semantic_type = branch_context
            .semantic_value_from_typed_type(&application_argument_type_from_expr(&value))
            .key;
        SurfaceExpressionAnalysis::valid_with_semantic_type(value, semantic_type)
    };
    let branch_analysis = match expression {
        SurfaceExpr::Match {
            scrutinee,
            arms,
            span,
        } => Some(super::surface_expr::match_expression::type_match_with(
            scrutinee,
            arms,
            *span,
            context,
            &mut type_branch,
        )),
        SurfaceExpr::If {
            condition,
            then_branch,
            else_branch,
            span,
        } => Some(super::surface_expr::conditional::type_if_with(
            condition,
            then_branch,
            else_branch,
            *span,
            context,
            &mut type_branch,
        )),
        SurfaceExpr::Block {
            items,
            result,
            span,
        } => Some(super::surface_expr::block::type_block_with(
            items,
            result,
            *span,
            context,
            &mut type_branch,
        )),
        _ => None,
    };
    if let Some(analysis) = branch_analysis {
        return finish_expression_analysis(analysis, issues);
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

    if let Some(analysis) =
        imported::type_imported_effect_application(expression, context, resolution, issues)
    {
        return finish_expression_analysis(analysis, issues);
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
    let mut operation = known_effect_operation_by_surface(&symbol.spelling)?;
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
    if operation.surface_name == "printValue" {
        if let [value] = arguments.as_slice() {
            let type_ref = inferred_type_from_expr(value);
            let (trait_identity, evidence) = match context.select_show_evidence(type_ref) {
                Ok((trait_identity, evidence)) => (trait_identity, Some(evidence)),
                Err(constraint) => {
                    issues.calls.push(PureCallIssue::MissingInstance {
                        callee: *span,
                        constraint,
                    });
                    ("std/prelude::Show".to_owned(), None)
                }
            };
            arguments = vec![TypedExpr::Template {
                parts: vec![TypedTemplatePart::Interpolation {
                    value: Box::new(value.clone()),
                    evidence,
                    trait_identity,
                    origin: super::effect_analysis::expression_origin(value),
                }],
                type_ref: named_type("String"),
                origin: expression.span(),
            }];
            operation = known_effect_operation_by_surface("print")
                .expect("printValue desugaring requires the standard print operation");
        }
    }
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
                let input = resolution.semantic_value_from_typed_type(&type_ref);
                let pattern_analysis =
                    super::surface_expr::pattern::type_pattern(pattern, &input, &context);
                if pattern_analysis.is_refutable() {
                    issues.calls.push(PureCallIssue::RefutableBindingPattern {
                        pattern: pattern.span(),
                        surface: "do bind",
                    });
                }
                locals.extend(pattern_analysis.locals.clone());
                issues.patterns.extend(pattern_analysis.issues);
                statements.push(TypedDoStatement::Bind {
                    pattern: pattern_analysis.typed,
                    value,
                    origin: *span,
                });
            }
            SurfaceDoItem::Let {
                pattern,
                type_ref,
                value,
                span,
            } => {
                let binding = super::surface_expr::pattern::type_pattern_binding(
                    pattern,
                    type_ref.as_ref(),
                    value,
                    &context,
                    |value, value_context| {
                        let value =
                            type_effect_expression(value, value_context, resolution, issues);
                        let semantic_type = value_context
                            .semantic_value_from_typed_type(&application_argument_type_from_expr(
                                &value,
                            ))
                            .key;
                        SurfaceExpressionAnalysis::valid_with_semantic_type(value, semantic_type)
                    },
                );
                if let Some(issue) = binding.mismatch {
                    issues.calls.push(issue);
                }
                if let Some(issue) = binding.expression.array_issue.clone() {
                    issues.arrays.push(issue);
                }
                if let Some(issue) = binding.expression.record_issue.clone() {
                    issues.records.push(issue);
                }
                if let Some(issue) = binding.expression.range_issue.clone() {
                    issues.ranges.push(issue);
                }
                if binding.pattern.is_refutable() {
                    issues.calls.push(PureCallIssue::RefutableBindingPattern {
                        pattern: pattern.span(),
                        surface: "do let",
                    });
                }
                locals.extend(binding.pattern.locals.clone());
                issues.patterns.extend(binding.pattern.issues);
                statements.push(TypedDoStatement::PureLet {
                    pattern: binding.pattern.typed,
                    value: binding.expression.value,
                    origin: *span,
                });
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
    if let Some(issue) = analysis.conditional_issue {
        issues.conditionals.push(issue);
    }
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
    issues.patterns.extend(analysis.match_issues);
    analysis.value
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
