use crate::{
    unit_type, TypedDecl, TypedEffect, TypedExpr, TypedParameter, TypedRecordField, TypedScheme,
    TypedType,
};
use seseragi_syntax::{ByteSpan, SurfaceDecl, SurfaceParameter, Token, TokenKind};

use super::expr::{
    find_effect_body, find_type_name_after, find_value_token, lower_first,
    typed_expr_from_value_token,
};
use super::type_ref::{inferred_type_from_expr, typed_type_from_type_ref};

pub(crate) fn typed_decl_from_surface(
    module: &str,
    declaration: SurfaceDecl,
    tokens: &[Token],
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
            return_type,
            span,
            ..
        } => {
            let with_type = find_type_name_after(tokens, span, TokenKind::KeywordWith);
            let failure = find_type_name_after(tokens, span, TokenKind::KeywordFails)
                .unwrap_or_else(|| "Never".to_owned());
            let success = return_type
                .as_ref()
                .map(typed_type_from_type_ref)
                .unwrap_or_else(unit_type);
            Some(TypedDecl::EffectFn {
                symbol: format!("{module}::{name}"),
                visibility,
                origin: span,
                parameters: vec![TypedParameter::ImplicitUnit {
                    type_ref: unit_type(),
                }],
                effect: TypedEffect {
                    environment: TypedType::Record {
                        closed: true,
                        fields: with_type
                            .map(|name| TypedRecordField {
                                name: lower_first(&name),
                                optional: false,
                                type_ref: TypedType::Named {
                                    name,
                                    arguments: Vec::new(),
                                },
                            })
                            .into_iter()
                            .collect(),
                    },
                    failure: TypedType::Named {
                        name: failure,
                        arguments: Vec::new(),
                    },
                    success,
                },
                body: find_effect_body(tokens, span).unwrap_or_else(|| TypedExpr::EffectCall {
                    operation: "std/prelude::unit".to_owned(),
                    arguments: Vec::new(),
                    origin: span,
                }),
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
            let body = find_value_token(tokens, span)
                .map(|token| typed_fn_body_from_token(token, &typed_parameters))
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

fn typed_fn_body_from_token(token: &Token, parameters: &[TypedParameter]) -> TypedExpr {
    if token.kind == TokenKind::IdentifierLower {
        if let Some((name, type_ref)) = find_parameter(token, parameters) {
            return TypedExpr::Variable {
                name,
                type_ref,
                origin: ByteSpan {
                    start: token.start,
                    end: token.end,
                },
            };
        }
    }
    typed_expr_from_value_token(token)
}

fn find_parameter(token: &Token, parameters: &[TypedParameter]) -> Option<(String, TypedType)> {
    parameters.iter().find_map(|parameter| match parameter {
        TypedParameter::Named { name, type_ref, .. } if name == &token.raw => {
            Some((name.clone(), type_ref.clone()))
        }
        _ => None,
    })
}
