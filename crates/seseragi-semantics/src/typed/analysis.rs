use super::TypedResolution;
use seseragi_syntax::{SurfaceExpr, SurfaceParameter, TypeRef};

use super::function_body::{function_body_issue, FunctionBodyIssue};
use super::functions::typed_parameters_from_surface;
use super::pure_issues::{
    ArrayIssue, ConditionalIssue, MatchIssue, MonadDoIssue, PureCallIssue, RangeIssue, RecordIssue,
};
use super::semantic_types::{semantic_values_are_compatible, SemanticValueType};
use super::surface_expr::{analyze_resolved_expression, PureExpressionContext};
use super::type_ref::inferred_type_from_expr;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PureFunctionAnalysis {
    pub(crate) conditional_issue: Option<ConditionalIssue>,
    pub(crate) array_issue: Option<ArrayIssue>,
    pub(crate) record_issue: Option<RecordIssue>,
    pub(crate) range_issue: Option<RangeIssue>,
    pub(crate) function_body_issue: Option<FunctionBodyIssue>,
    pub(crate) pure_call_issue: Option<PureCallIssue>,
    pub(crate) monad_do_issue: Option<MonadDoIssue>,
    pub(crate) match_issues: Vec<MatchIssue>,
}

pub(crate) fn analyze_pure_function(
    body: Option<&SurfaceExpr>,
    parameters: &[SurfaceParameter],
    return_type: &TypeRef,
    resolution: &TypedResolution<'_>,
    scoped_evidence: &[super::ScopedCallEvidence],
) -> PureFunctionAnalysis {
    let Some(body) = body else {
        return PureFunctionAnalysis {
            conditional_issue: None,
            array_issue: None,
            record_issue: None,
            range_issue: None,
            function_body_issue: None,
            pure_call_issue: None,
            monad_do_issue: None,
            match_issues: Vec::new(),
        };
    };
    let typed_parameters = typed_parameters_from_surface(parameters);
    let context = PureExpressionContext::new(&typed_parameters, resolution)
        .with_evidence_parameters(scoped_evidence.to_vec())
        .with_expected(Some(resolution.semantic_value_from_type_ref(return_type)));
    let expression = analyze_resolved_expression(body, &context);
    let expected = resolution.semantic_value_from_type_ref(return_type);
    let actual = SemanticValueType {
        type_ref: inferred_type_from_expr(&expression.value),
        key: expression.semantic_type.clone(),
    };
    let semantically_compatible = semantic_values_are_compatible(&expected, &actual);
    let function_body_issue = (expression.conditional_issue.is_none()
        && expression.array_issue.is_none()
        && expression.record_issue.is_none()
        && expression.range_issue.is_none()
        && expression.pure_call_issue.is_none()
        && expression.monad_do_issue.is_none()
        && expression.match_issues.is_empty())
    .then(|| {
        function_body_issue(
            Some(&expression.value),
            Some(body.span()),
            return_type,
            semantically_compatible,
        )
    })
    .flatten();

    PureFunctionAnalysis {
        conditional_issue: expression.conditional_issue,
        array_issue: expression.array_issue,
        record_issue: expression.record_issue,
        range_issue: expression.range_issue,
        function_body_issue,
        pure_call_issue: expression.pure_call_issue,
        monad_do_issue: expression.monad_do_issue,
        match_issues: expression.match_issues,
    }
}
