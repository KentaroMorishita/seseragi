use crate::typed::pure_issues::ArrayIssue;
use crate::typed::semantic_types::{
    semantic_values_are_compatible, SemanticTypeKey, SemanticValueType,
};
use crate::typed::type_ref::{inferred_type_from_expr, typed_type_contains_hole};
use crate::{TypedComprehensionClause, TypedExpr, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceComprehensionClause, SurfaceExpr};
use std::collections::BTreeMap;

use super::match_expression::pattern::type_pattern;
use super::{
    named_type_is, type_surface_expression, PureExpressionContext, SurfaceExpressionAnalysis,
};

pub(super) fn type_array_comprehension(
    element: &SurfaceExpr,
    clauses: &[SurfaceComprehensionClause],
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    let mut current_context = context.without_expected();
    let mut typed_clauses = Vec::with_capacity(clauses.len());
    let mut children = Vec::new();
    let mut match_issues = Vec::new();
    let mut missing_instance = None;
    let mut locals = BTreeMap::new();
    let mut generator_count = 0usize;

    if !matches!(
        clauses.first(),
        Some(SurfaceComprehensionClause::Generator { .. })
    ) {
        match_issues.push(crate::typed::pure_issues::MatchIssue::PatternMismatch {
            pattern: span,
            message: "comprehension clauses must begin with a generator".to_owned(),
        });
    }

    for clause in clauses {
        match clause {
            SurfaceComprehensionClause::Generator {
                pattern,
                source,
                span,
            } => {
                generator_count += 1;
                let source_analysis = type_surface_expression(source, &current_context);
                let source_type = inferred_type_from_expr(&source_analysis.value);
                let (element_type, evidence) =
                    match current_context.select_iterable_evidence(source_type.clone()) {
                        Ok(selected) => selected,
                        Err(constraint) => {
                            missing_instance =
                                Some(crate::typed::pure_issues::PureCallIssue::MissingInstance {
                                    callee: source.span(),
                                    constraint,
                                });
                            children.push(source_analysis);
                            continue;
                        }
                    };
                let expected = current_context.semantic_value_from_typed_type(&element_type);
                let pattern_analysis = type_pattern(pattern, &expected, &current_context);
                match_issues.extend(pattern_analysis.issues);
                locals.extend(pattern_analysis.locals);
                current_context = context.with_locals(locals.clone()).without_expected();
                typed_clauses.push(TypedComprehensionClause::Generator {
                    pattern: pattern_analysis.typed,
                    source: source_analysis.value.clone(),
                    evidence,
                    origin: *span,
                });
                children.push(source_analysis);
            }
            SurfaceComprehensionClause::Guard { condition, span } => {
                let guard = type_surface_expression(condition, &current_context);
                let actual = inferred_type_from_expr(&guard.value);
                if !typed_type_contains_hole(&actual) && !named_type_is(&actual, "Bool") {
                    match_issues.push(crate::typed::pure_issues::MatchIssue::GuardNotBool {
                        guard: condition.span(),
                        actual,
                    });
                }
                typed_clauses.push(TypedComprehensionClause::Guard {
                    condition: guard.value.clone(),
                    origin: *span,
                });
                children.push(guard);
            }
        }
    }

    if generator_count == 0 {
        match_issues.push(crate::typed::pure_issues::MatchIssue::PatternMismatch {
            pattern: span,
            message: "comprehension requires at least one generator".to_owned(),
        });
    }

    let expected_element = expected_array_element(context);
    let element_analysis = type_surface_expression(
        element,
        &current_context.with_expected(expected_element.clone()),
    );
    let inferred_element = inferred_type_from_expr(&element_analysis.value);
    let actual_element = SemanticValueType {
        type_ref: inferred_element.clone(),
        key: element_analysis.semantic_type.clone(),
    };
    let expected_element = expected_element.unwrap_or_else(|| actual_element.clone());
    let array_issue =
        (!semantic_values_are_compatible(&expected_element, &actual_element)).then(|| {
            ArrayIssue::ElementTypeMismatch {
                element: element.span(),
                index: 0,
                expected: expected_element.type_ref.clone(),
                actual: actual_element.type_ref,
            }
        });
    let element_type = expected_element.type_ref;
    let invalid = typed_type_contains_hole(&element_type)
        || missing_instance.is_some()
        || !match_issues.is_empty()
        || array_issue.is_some();
    let type_ref = TypedType::Named {
        name: "Array".to_owned(),
        arguments: vec![if invalid {
            TypedType::Hole
        } else {
            element_type
        }],
    };
    let mut result = SurfaceExpressionAnalysis::valid_with_semantic_type(
        TypedExpr::ArrayComprehension {
            element: Box::new(element_analysis.value.clone()),
            clauses: typed_clauses,
            type_ref,
            origin: span,
        },
        if invalid {
            SemanticTypeKey::Invalid
        } else {
            SemanticTypeKey::Other
        },
    );
    result.pure_call_issue = missing_instance;
    result.match_issues = match_issues;
    result.array_issue = array_issue;
    for child in children {
        result.merge_issues_from(child);
    }
    result.merge_issues_from(element_analysis);
    result
}

fn expected_array_element(context: &PureExpressionContext<'_>) -> Option<SemanticValueType> {
    let TypedType::Named { name, arguments } = &context.expected()?.type_ref else {
        return None;
    };
    (name == "Array" && arguments.len() == 1)
        .then(|| context.semantic_value_from_typed_type(&arguments[0]))
}
