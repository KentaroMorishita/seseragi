use crate::{unit_type, TypedDecl, TypedExpr, TypedParameter, TypedScheme, TypedType};
use seseragi_syntax::{SurfaceDecl, SurfaceParameter, Token};
use std::collections::BTreeMap;

use super::effect::typed_effect_from_surface;
use super::expr::{
    find_effect_body, find_value_token, find_value_tokens, typed_expr_from_value_token,
    typed_fn_body_from_tokens,
};
use super::type_ref::{inferred_type_from_expr, typed_type_from_type_ref};

pub(crate) fn typed_decl_from_surface(
    module: &str,
    declaration: SurfaceDecl,
    tokens: &[Token],
    top_level_values: &BTreeMap<String, TypedType>,
) -> Option<TypedDecl> {
    match declaration {
        SurfaceDecl::Let {
            visibility,
            name,
            type_ref,
            span,
            ..
        } => {
            let value = find_value_token(tokens, span)
                .map(typed_expr_from_value_token)
                .unwrap_or_else(|| TypedExpr::Integer {
                    value: "0".to_owned(),
                    type_ref: TypedType::Named {
                        name: "Int".to_owned(),
                        arguments: Vec::new(),
                    },
                    origin: span,
                });
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
            let typed_parameters = parameters
                .iter()
                .map(typed_parameter_from_surface)
                .collect::<Vec<_>>();
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
            span,
            ..
        } => {
            let typed_parameters = parameters
                .iter()
                .map(typed_parameter_from_surface)
                .collect::<Vec<_>>();
            let body = typed_fn_body_from_tokens(
                &find_value_tokens(tokens, span),
                &typed_parameters,
                top_level_values,
            )
            .unwrap_or_else(|| TypedExpr::Variable {
                name: String::new(),
                type_ref: TypedType::Hole,
                origin: span,
            });
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

fn typed_parameter_from_surface(parameter: &SurfaceParameter) -> TypedParameter {
    TypedParameter::Named {
        name: parameter.name.clone(),
        type_ref: typed_type_from_type_ref(&parameter.type_ref),
        origin: parameter.name_span,
    }
}
