use crate::typed::pure_issues::PureCallIssue;
use crate::typed::semantic_types::{SemanticTypeKey, SemanticValueType};
use crate::typed::type_ref::{
    application_argument_type_from_expr, effect_from_value_type, inferred_type_from_expr,
    typed_type_contains_hole,
};
use crate::{TypedExpr, TypedMatchArm, TypedParameter, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceExpr, SurfacePattern};

use super::match_expression::pattern::type_pattern;
use super::{type_surface_expression, PureExpressionContext, SurfaceExpressionAnalysis};

pub(super) fn type_effectful_for(
    pattern: &SurfacePattern,
    source: &SurfaceExpr,
    body: &SurfaceExpr,
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    type_effectful_for_with(
        pattern,
        source,
        body,
        span,
        context,
        type_surface_expression,
    )
}

pub(crate) fn type_effectful_for_with(
    pattern: &SurfacePattern,
    source: &SurfaceExpr,
    body: &SurfaceExpr,
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
    mut type_body: impl FnMut(&SurfaceExpr, &PureExpressionContext<'_>) -> SurfaceExpressionAnalysis,
) -> SurfaceExpressionAnalysis {
    let source_analysis = type_surface_expression(source, &context.without_expected());
    let source_type = inferred_type_from_expr(&source_analysis.value);
    let (element_type, evidence, mut issue) =
        match context.select_iterable_evidence(source_type.clone()) {
            Ok((element_type, evidence)) => (element_type, Some(evidence), None),
            Err(constraint) => (
                TypedType::Hole,
                None,
                Some(PureCallIssue::MissingInstance {
                    callee: source.span(),
                    constraint,
                }),
            ),
        };
    let expected_element = if typed_type_contains_hole(&element_type) {
        SemanticValueType {
            type_ref: element_type.clone(),
            key: SemanticTypeKey::Other,
        }
    } else {
        context.semantic_value_from_typed_type(&element_type)
    };
    let pattern_analysis = type_pattern(pattern, &expected_element, context);
    let irrefutable = pattern_analysis.is_irrefutable();
    if issue.is_none() && !irrefutable {
        issue = Some(PureCallIssue::EffectfulForRefutablePattern {
            pattern: pattern.span(),
        });
    }

    let body_analysis = type_body(
        body,
        &context
            .with_locals(pattern_analysis.locals.clone())
            .without_expected(),
    );
    let body_type = application_argument_type_from_expr(&body_analysis.value);
    let body_effect = effect_from_value_type(&body_type);
    if issue.is_none() {
        issue = match &body_effect {
            None if !typed_type_contains_hole(&body_type) => {
                Some(PureCallIssue::EffectfulForBodyNotEffect {
                    body: body.span(),
                    actual: body_type.clone(),
                })
            }
            Some(effect)
                if !typed_type_contains_hole(&effect.success)
                    && !named_type_is(&effect.success, "Unit") =>
            {
                Some(PureCallIssue::EffectfulForBodyNotUnit {
                    body: body.span(),
                    actual: effect.success.clone(),
                })
            }
            _ => None,
        };
    }

    let action_parameter = "$ssrg_for_value".to_owned();
    let action_body = TypedExpr::Match {
        scrutinee: Box::new(TypedExpr::Variable {
            name: action_parameter.clone(),
            evidence: Vec::new(),
            type_ref: element_type.clone(),
            origin: pattern.span(),
        }),
        arms: vec![TypedMatchArm {
            pattern: pattern_analysis.typed,
            guard: None,
            body: body_analysis.value.clone(),
            origin: ByteSpan {
                start: pattern.span().start,
                end: body.span().end,
            },
        }],
        exhaustive: irrefutable,
        type_ref: body_type.clone(),
        origin: ByteSpan {
            start: pattern.span().start,
            end: body.span().end,
        },
    };
    let action = TypedExpr::Lambda {
        parameter: TypedParameter::Named {
            name: action_parameter,
            type_ref: element_type.clone(),
            origin: pattern.span(),
        },
        body: Box::new(action_body),
        type_ref: TypedType::Function {
            parameter: Box::new(element_type),
            result: Box::new(body_type.clone()),
        },
        origin: ByteSpan {
            start: pattern.span().start,
            end: body.span().end,
        },
    };
    let invalid = issue.is_some()
        || !pattern_analysis.issues.is_empty()
        || body_effect.is_none()
        || typed_type_contains_hole(&body_type);
    let mut result = SurfaceExpressionAnalysis::valid_with_semantic_type(
        TypedExpr::Call {
            callee: "std/prelude::forEach".to_owned(),
            arguments: vec![action, source_analysis.value.clone()],
            evidence: evidence.into_iter().collect(),
            deferred_evidence_parameters: Vec::new(),
            deferred_evidence_type_constructor_parameters: Vec::new(),
            trait_dispatch: None,
            type_ref: if invalid { TypedType::Hole } else { body_type },
            origin: span,
        },
        if invalid {
            SemanticTypeKey::Invalid
        } else {
            SemanticTypeKey::Other
        },
    );
    result.pure_call_issue = issue;
    result.match_issues = pattern_analysis.issues;
    result.merge_issues_from(source_analysis);
    result.merge_issues_from(body_analysis);
    result
}

fn named_type_is(type_ref: &TypedType, expected: &str) -> bool {
    matches!(
        type_ref,
        TypedType::Named { name, arguments }
            if name == expected && arguments.is_empty()
    )
}
