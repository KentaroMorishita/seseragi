use crate::{
    unit_type, TypedDecl, TypedEffect, TypedExpr, TypedParameter, TypedRecordField, TypedScheme,
    TypedType,
};
use seseragi_syntax::{SurfaceDecl, Token, TokenKind};

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
        SurfaceDecl::Fn { .. }
        | SurfaceDecl::Newtype { .. }
        | SurfaceDecl::Alias { .. }
        | SurfaceDecl::Trait { .. }
        | SurfaceDecl::Operator { .. }
        | SurfaceDecl::Instance { .. } => None,
    }
}
