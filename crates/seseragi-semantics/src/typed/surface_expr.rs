use crate::{TypedExpr, TypedParameter, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceExpr};
use std::collections::{BTreeMap, BTreeSet};

use super::functions::{application_result_type, TopLevelPureFunction};
use super::pure_issues::{ConditionalIssue, PureCallIssue, UnresolvedNameIssue};

mod application;
mod binary;
mod conditional;
mod tuple;

pub(crate) struct PureExpressionContext<'a> {
    parameters: BTreeMap<&'a str, TypedType>,
    declared_values: &'a BTreeSet<String>,
    top_level_values: &'a BTreeMap<String, TypedType>,
    top_level_functions: &'a BTreeMap<String, TopLevelPureFunction>,
}

impl<'a> PureExpressionContext<'a> {
    pub(crate) fn new(
        parameters: &'a [TypedParameter],
        declared_values: &'a BTreeSet<String>,
        top_level_values: &'a BTreeMap<String, TypedType>,
        top_level_functions: &'a BTreeMap<String, TopLevelPureFunction>,
    ) -> Self {
        let parameters = parameters
            .iter()
            .filter_map(|parameter| match parameter {
                TypedParameter::Named { name, type_ref, .. } => {
                    Some((name.as_str(), type_ref.clone()))
                }
                TypedParameter::ImplicitUnit { .. } => None,
            })
            .collect();
        Self {
            parameters,
            declared_values,
            top_level_values,
            top_level_functions,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SurfaceExpressionAnalysis {
    pub(crate) value: TypedExpr,
    pub(crate) conditional_issue: Option<ConditionalIssue>,
    pub(crate) pure_call_issue: Option<PureCallIssue>,
    pub(crate) unresolved_names: Vec<UnresolvedNameIssue>,
}

impl SurfaceExpressionAnalysis {
    pub(super) fn valid(value: TypedExpr) -> Self {
        Self {
            value,
            conditional_issue: None,
            pure_call_issue: None,
            unresolved_names: Vec::new(),
        }
    }

    pub(super) fn merge_issues_from(&mut self, child: Self) {
        self.conditional_issue = self.conditional_issue.take().or(child.conditional_issue);
        self.pure_call_issue = self.pure_call_issue.take().or(child.pure_call_issue);
        self.unresolved_names.extend(child.unresolved_names);
    }
}

pub(crate) fn analyze_surface_expression(
    expression: &SurfaceExpr,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    type_surface_expression(expression, context)
}

pub(crate) fn surface_expression_type_hint(expression: &SurfaceExpr) -> Option<TypedType> {
    match expression {
        SurfaceExpr::Unit { .. } => Some(named_type("Unit")),
        SurfaceExpr::Integer { .. } => Some(named_type("Int")),
        SurfaceExpr::String { .. } => Some(named_type("String")),
        SurfaceExpr::Boolean { .. } => Some(named_type("Bool")),
        SurfaceExpr::Tuple { elements, .. } => Some(TypedType::Tuple {
            elements: elements
                .iter()
                .map(surface_expression_type_hint)
                .collect::<Option<Vec<_>>>()?,
        }),
        SurfaceExpr::Grouped { value, .. } => surface_expression_type_hint(value),
        SurfaceExpr::If {
            then_branch,
            else_branch,
            ..
        } => {
            let then_type = surface_expression_type_hint(then_branch)?;
            (surface_expression_type_hint(else_branch)? == then_type).then_some(then_type)
        }
        SurfaceExpr::Name { .. }
        | SurfaceExpr::Application { .. }
        | SurfaceExpr::Binary { .. }
        | SurfaceExpr::Do { .. }
        | SurfaceExpr::Error { .. } => None,
    }
}

pub(super) fn type_surface_expression(
    expression: &SurfaceExpr,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    match expression {
        SurfaceExpr::Unit { span } => SurfaceExpressionAnalysis::valid(TypedExpr::Unit {
            type_ref: named_type("Unit"),
            origin: *span,
        }),
        SurfaceExpr::Integer { raw, span } => {
            SurfaceExpressionAnalysis::valid(TypedExpr::Integer {
                value: raw.clone(),
                type_ref: named_type("Int"),
                origin: *span,
            })
        }
        SurfaceExpr::String { raw, span } => SurfaceExpressionAnalysis::valid(TypedExpr::String {
            value: unquote_string(raw),
            type_ref: named_type("String"),
            origin: *span,
        }),
        SurfaceExpr::Boolean { value, span } => {
            SurfaceExpressionAnalysis::valid(TypedExpr::Boolean {
                value: *value,
                type_ref: named_type("Bool"),
                origin: *span,
            })
        }
        SurfaceExpr::Name { name, span } => type_name(name, *span, context),
        SurfaceExpr::Grouped { value, .. } => type_surface_expression(value, context),
        SurfaceExpr::Application { .. } => application::type_application(expression, context),
        SurfaceExpr::Tuple { elements, span } => tuple::type_tuple(elements, *span, context),
        SurfaceExpr::Binary {
            operator,
            left,
            right,
            span,
            ..
        } => binary::type_binary(operator, left, right, *span, context),
        SurfaceExpr::If {
            condition,
            then_branch,
            else_branch,
            span,
        } => conditional::type_if(condition, then_branch, else_branch, *span, context),
        SurfaceExpr::Do { span, .. } | SurfaceExpr::Error { span } => {
            SurfaceExpressionAnalysis::valid(TypedExpr::Variable {
                name: String::new(),
                type_ref: TypedType::Hole,
                origin: *span,
            })
        }
    }
}

fn type_name(
    name: &str,
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    if let Some(type_ref) = context.parameters.get(name) {
        return SurfaceExpressionAnalysis::valid(TypedExpr::Variable {
            name: name.to_owned(),
            type_ref: type_ref.clone(),
            origin: span,
        });
    }
    if let Some(type_ref) = context.top_level_values.get(name) {
        return SurfaceExpressionAnalysis::valid(TypedExpr::Variable {
            name: name.to_owned(),
            type_ref: type_ref.clone(),
            origin: span,
        });
    }
    if let Some(function) = context.top_level_functions.get(name) {
        return SurfaceExpressionAnalysis::valid(TypedExpr::Variable {
            name: function.symbol.clone(),
            type_ref: application_result_type(function, 0),
            origin: span,
        });
    }

    let mut analysis = SurfaceExpressionAnalysis::valid(TypedExpr::Variable {
        name: name.to_owned(),
        type_ref: TypedType::Hole,
        origin: span,
    });
    if !context.declared_values.contains(name) {
        analysis
            .unresolved_names
            .push(UnresolvedNameIssue { origin: span });
    }
    analysis
}

pub(super) fn named_type(name: &str) -> TypedType {
    TypedType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}

pub(super) fn named_type_is(type_ref: &TypedType, expected: &str) -> bool {
    matches!(type_ref, TypedType::Named { name, .. } if name == expected)
}

fn unquote_string(raw: &str) -> String {
    raw.strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(raw)
        .to_owned()
}
