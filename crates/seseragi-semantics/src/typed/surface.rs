use crate::{unit_type, TypedDecl, TypedExpr, TypedParameter, TypedScheme, TypedType};
use seseragi_syntax::{SurfaceDecl, Token};
use std::collections::{BTreeMap, BTreeSet};

use super::effect::typed_effect_from_surface;
use super::expr::find_effect_body;
use super::functions::{typed_parameters_from_surface, TopLevelPureFunction};
use super::surface_expr::{analyze_surface_expression, PureExpressionContext};
use super::type_ref::{inferred_type_from_expr, typed_type_from_type_ref};

pub(crate) fn typed_decl_from_surface(
    module: &str,
    declaration: SurfaceDecl,
    tokens: &[Token],
    declared_values: &BTreeSet<String>,
    top_level_values: &BTreeMap<String, TypedType>,
    top_level_functions: &BTreeMap<String, TopLevelPureFunction>,
) -> Option<TypedDecl> {
    match declaration {
        SurfaceDecl::Let {
            visibility,
            name,
            type_ref,
            body,
            span,
            ..
        } => {
            let no_parameters = Vec::new();
            let context = PureExpressionContext::new(
                &no_parameters,
                declared_values,
                top_level_values,
                top_level_functions,
            );
            let value = body
                .as_ref()
                .map(|body| analyze_surface_expression(body, &context).value)
                .unwrap_or_else(|| hole_expression(span));
            Some(TypedDecl::Let {
                symbol: format!("{module}::{name}"),
                visibility,
                origin: span,
                scheme: TypedScheme {
                    type_parameters: Vec::new(),
                    constraints: Vec::new(),
                    type_ref: type_ref
                        .as_ref()
                        .map(typed_type_from_type_ref)
                        .unwrap_or_else(|| inferred_type_from_expr(&value)),
                },
                value,
            })
        }
        SurfaceDecl::EffectFn {
            visibility,
            name,
            parameters,
            return_type,
            inferred_contract,
            span,
            ..
        } => {
            let typed_parameters = typed_parameters_from_surface(&parameters);
            let body = find_effect_body(tokens, span).unwrap_or_else(|| TypedExpr::EffectCall {
                operation: "std/prelude::unit".to_owned(),
                arguments: Vec::new(),
                origin: span,
            });
            let effect =
                typed_effect_from_surface(&return_type, inferred_contract, tokens, span, &body);
            Some(TypedDecl::EffectFn {
                symbol: format!("{module}::{name}"),
                visibility,
                origin: span,
                inferred_contract,
                parameters: if typed_parameters.is_empty() {
                    vec![TypedParameter::ImplicitUnit {
                        type_ref: unit_type(),
                    }]
                } else {
                    typed_parameters
                },
                effect,
                body,
            })
        }
        SurfaceDecl::Fn {
            visibility,
            name,
            type_parameters,
            parameters,
            return_type,
            constraints,
            body,
            span,
            ..
        } => {
            let typed_parameters = typed_parameters_from_surface(&parameters);
            let context = PureExpressionContext::new(
                &typed_parameters,
                declared_values,
                top_level_values,
                top_level_functions,
            );
            let body = body
                .as_ref()
                .map(|body| analyze_surface_expression(body, &context).value)
                .unwrap_or_else(|| hole_expression(span));
            Some(TypedDecl::Fn {
                symbol: format!("{module}::{name}"),
                visibility,
                origin: span,
                scheme: TypedScheme {
                    type_parameters,
                    constraints: constraints
                        .into_iter()
                        .map(|name| crate::TypedConstraint { name })
                        .collect(),
                    type_ref: typed_type_from_type_ref(&return_type),
                },
                parameters: typed_parameters,
                body,
            })
        }
        SurfaceDecl::Newtype { .. }
        | SurfaceDecl::Alias { .. }
        | SurfaceDecl::Type { .. }
        | SurfaceDecl::Struct { .. }
        | SurfaceDecl::Trait { .. }
        | SurfaceDecl::Operator { .. }
        | SurfaceDecl::Instance { .. } => None,
    }
}

fn hole_expression(origin: seseragi_syntax::ByteSpan) -> TypedExpr {
    TypedExpr::Variable {
        name: String::new(),
        type_ref: TypedType::Hole,
        origin,
    }
}
