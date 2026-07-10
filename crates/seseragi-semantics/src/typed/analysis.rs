use crate::TypedType;
use seseragi_syntax::{SurfaceExpr, SurfaceParameter, TypeRef};
use std::collections::{BTreeMap, BTreeSet};

use super::function_body::{function_body_issue, FunctionBodyIssue};
use super::functions::{typed_parameters_from_surface, TopLevelPureFunction};
use super::pure_issues::{ConditionalIssue, PureCallIssue, UnresolvedNameIssue};
use super::surface_expr::{analyze_surface_expression, PureExpressionContext};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PureFunctionAnalysis {
    pub(crate) conditional_issue: Option<ConditionalIssue>,
    pub(crate) function_body_issue: Option<FunctionBodyIssue>,
    pub(crate) pure_call_issue: Option<PureCallIssue>,
    pub(crate) unresolved_names: Vec<UnresolvedNameIssue>,
}

pub(crate) fn analyze_pure_function(
    body: Option<&SurfaceExpr>,
    parameters: &[SurfaceParameter],
    return_type: &TypeRef,
    declared_values: &BTreeSet<String>,
    top_level_values: &BTreeMap<String, TypedType>,
    top_level_functions: &BTreeMap<String, TopLevelPureFunction>,
) -> PureFunctionAnalysis {
    let Some(body) = body else {
        return PureFunctionAnalysis {
            conditional_issue: None,
            function_body_issue: None,
            pure_call_issue: None,
            unresolved_names: Vec::new(),
        };
    };
    let typed_parameters = typed_parameters_from_surface(parameters);
    let context = PureExpressionContext::new(
        &typed_parameters,
        declared_values,
        top_level_values,
        top_level_functions,
    );
    let expression = analyze_surface_expression(body, &context);
    let function_body_issue = (expression.conditional_issue.is_none()
        && expression.pure_call_issue.is_none())
    .then(|| function_body_issue(Some(&expression.value), Some(body.span()), return_type))
    .flatten();

    PureFunctionAnalysis {
        conditional_issue: expression.conditional_issue,
        function_body_issue,
        pure_call_issue: expression.pure_call_issue,
        unresolved_names: expression.unresolved_names,
    }
}
