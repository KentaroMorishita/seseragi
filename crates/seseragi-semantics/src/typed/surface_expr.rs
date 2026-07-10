use crate::{SymbolId, SymbolNamespace, TypedExpr, TypedParameter, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceExpr};
use std::collections::BTreeMap;

use super::functions::{application_result_type, TopLevelPureFunction};
use super::pure_issues::{ConditionalIssue, PureCallIssue};
use super::TypedResolution;

mod application;
mod binary;
mod conditional;
mod tuple;

pub(crate) struct PureExpressionContext<'a> {
    parameters: BTreeMap<SymbolId, TypedType>,
    resolution: &'a TypedResolution<'a>,
}

impl<'a> PureExpressionContext<'a> {
    pub(crate) fn new(parameters: &[TypedParameter], resolution: &'a TypedResolution<'a>) -> Self {
        Self {
            parameters: resolution.parameter_types(parameters),
            resolution,
        }
    }

    pub(super) fn target(&self, origin: ByteSpan) -> Option<SymbolId> {
        self.resolution.target(origin, SymbolNamespace::Value)
    }

    pub(super) fn callable(&self, target: SymbolId) -> Option<&TopLevelPureFunction> {
        self.resolution.callable(target)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SurfaceExpressionAnalysis {
    pub(crate) value: TypedExpr,
    pub(crate) conditional_issue: Option<ConditionalIssue>,
    pub(crate) pure_call_issue: Option<PureCallIssue>,
}

impl SurfaceExpressionAnalysis {
    pub(super) fn valid(value: TypedExpr) -> Self {
        Self {
            value,
            conditional_issue: None,
            pure_call_issue: None,
        }
    }

    pub(super) fn merge_issues_from(&mut self, child: Self) {
        self.conditional_issue = self.conditional_issue.take().or(child.conditional_issue);
        self.pure_call_issue = self.pure_call_issue.take().or(child.pure_call_issue);
    }
}

pub(crate) fn analyze_resolved_expression(
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
    let Some(target) = context.target(span) else {
        return SurfaceExpressionAnalysis::valid(TypedExpr::Variable {
            name: name.to_owned(),
            type_ref: TypedType::Hole,
            origin: span,
        });
    };
    if let Some(type_ref) = context.parameters.get(&target) {
        return SurfaceExpressionAnalysis::valid(TypedExpr::Variable {
            name: name.to_owned(),
            type_ref: type_ref.clone(),
            origin: span,
        });
    }
    if let Some(type_ref) = context.resolution.top_level_value_type(target) {
        let resolved_name = context
            .resolution
            .symbol(target)
            .map(|symbol| symbol.spelling.clone())
            .unwrap_or_else(|| name.to_owned());
        return SurfaceExpressionAnalysis::valid(TypedExpr::Variable {
            name: resolved_name,
            type_ref: type_ref.clone(),
            origin: span,
        });
    }
    if let Some(function) = context.callable(target) {
        return SurfaceExpressionAnalysis::valid(TypedExpr::Variable {
            name: function.symbol.clone(),
            type_ref: application_result_type(function, 0),
            origin: span,
        });
    }

    SurfaceExpressionAnalysis::valid(TypedExpr::Variable {
        name: name.to_owned(),
        type_ref: TypedType::Hole,
        origin: span,
    })
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
