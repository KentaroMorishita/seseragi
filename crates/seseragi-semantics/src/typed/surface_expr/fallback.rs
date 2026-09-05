use super::{type_surface_expression, PureExpressionContext, SurfaceExpressionAnalysis};
use crate::typed::pure_issues::PureCallIssue;
use crate::typed::semantic_types::{
    semantic_values_are_compatible, SemanticTypeKey, SemanticValueType,
};
use crate::typed::type_ref::{inferred_type_from_expr, typed_type_contains_hole};
use crate::{TypedExpr, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceExpr};

fn maybe_type(
    element: SemanticValueType,
    context: &PureExpressionContext<'_>,
) -> SemanticValueType {
    let type_ref = TypedType::Named {
        name: "Maybe".to_owned(),
        arguments: vec![element.type_ref.clone()],
    };
    let key = context
        .resolution
        .resolved()
        .symbols
        .iter()
        .find(|symbol| symbol.canonical.as_deref() == Some("std/prelude::Maybe"))
        .map(|symbol| SemanticTypeKey::Adt {
            owner: symbol.id,
            arguments: vec![element.clone()],
        })
        .unwrap_or_else(|| SemanticTypeKey::NamedGeneric {
            name: "Maybe".to_owned(),
            arguments: vec![element],
        });
    SemanticValueType { type_ref, key }
}

fn maybe_element(
    key: &SemanticTypeKey,
    context: &PureExpressionContext<'_>,
) -> Option<SemanticValueType> {
    let arguments = match key {
        SemanticTypeKey::NamedGeneric { name, arguments } if name == "Maybe" => arguments,
        SemanticTypeKey::ExternalNominal {
            canonical,
            arguments,
        } if canonical == "std/prelude::Maybe" => arguments,
        SemanticTypeKey::Adt { owner, arguments }
            if context.resolution.resolved().symbols.iter().any(|symbol| {
                symbol.id == *owner && symbol.canonical.as_deref() == Some("std/prelude::Maybe")
            }) =>
        {
            arguments
        }
        _ => return None,
    };
    match arguments.as_slice() {
        [element] => Some(element.clone()),
        _ => None,
    }
}

pub(super) fn type_fallback(
    left: &SurfaceExpr,
    right: &SurfaceExpr,
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    let left_context = context.with_expected(
        context
            .expected()
            .cloned()
            .map(|element| maybe_type(element, context)),
    );
    let mut left_analysis = type_surface_expression(left, &left_context);
    let mut element = maybe_element(&left_analysis.semantic_type, context);
    let expected = element
        .clone()
        .filter(|element| !typed_type_contains_hole(&element.type_ref))
        .or_else(|| context.expected().cloned());
    let right_analysis = type_surface_expression(right, &context.with_expected(expected));
    let right_type = SemanticValueType {
        type_ref: inferred_type_from_expr(&right_analysis.value),
        key: right_analysis.semantic_type.clone(),
    };
    // A nullary Nothing constructor carries an uninstantiated scheme parameter,
    // not a Hole. Recheck the left under the concrete fallback expectation.
    // Refine nullary constructor parameters from the fallback, but preserve
    // concrete left types and their operand diagnostics when refinement fails.
    if element.is_some() && !typed_type_contains_hole(&right_type.type_ref) {
        let candidate = type_surface_expression(
            left,
            &context.with_expected(Some(maybe_type(right_type.clone(), context))),
        );
        let candidate_element = maybe_element(&candidate.semantic_type, context);
        let clean = candidate.pure_call_issue.is_none()
            && candidate.conditional_issue.is_none()
            && candidate.array_issue.is_none()
            && candidate.record_issue.is_none()
            && candidate.range_issue.is_none()
            && candidate.monad_do_issue.is_none()
            && candidate.match_issues.is_empty();
        if clean
            && candidate_element.as_ref().is_some_and(|value| {
                semantic_values_are_compatible(value, &right_type)
                    && context
                        .expected()
                        .is_none_or(|expected| semantic_values_are_compatible(expected, value))
            })
        {
            left_analysis = candidate;
            element = candidate_element;
        }
    }
    let issue = match &element {
        None => Some(PureCallIssue::ArgumentType {
            argument: left.span(),
            index: 0,
            expected: maybe_type(right_type.clone(), context).type_ref,
            actual: inferred_type_from_expr(&left_analysis.value),
        }),
        Some(element) if !semantic_values_are_compatible(element, &right_type) => {
            Some(PureCallIssue::ArgumentType {
                argument: right.span(),
                index: 1,
                expected: element.type_ref.clone(),
                actual: right_type.type_ref.clone(),
            })
        }
        _ => None,
    };
    let result_type = element.unwrap_or(SemanticValueType {
        type_ref: TypedType::Hole,
        key: SemanticTypeKey::Invalid,
    });
    let mut result = SurfaceExpressionAnalysis::valid_with_semantic_type(
        TypedExpr::Binary {
            operator: "??".to_owned(),
            left: Box::new(left_analysis.value.clone()),
            right: Box::new(right_analysis.value.clone()),
            evidence: vec![],
            type_ref: result_type.type_ref,
            origin: span,
        },
        result_type.key,
    );
    result.merge_issues_from(left_analysis);
    result.merge_issues_from(right_analysis);
    result.pure_call_issue = result.pure_call_issue.or(issue);
    result
}
