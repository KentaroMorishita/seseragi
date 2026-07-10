use crate::{unit_type, SymbolKind, TypedDecl, TypedExpr, TypedParameter, TypedScheme, TypedType};
use seseragi_syntax::{SurfaceDecl, Token};

use super::adt::{typed_adt_decl, AdtDeclInput};
use super::effect::typed_effect_from_surface;
use super::expr::find_effect_body;
use super::functions::typed_parameters_from_surface;
use super::surface_expr::{analyze_resolved_expression, PureExpressionContext};
use super::type_ref::{inferred_type_from_expr, typed_type_from_type_ref};
use super::TypedResolution;

pub(crate) fn typed_decl_from_surface(
    declaration: SurfaceDecl,
    tokens: &[Token],
    resolution: &TypedResolution<'_>,
) -> Option<TypedDecl> {
    match declaration {
        SurfaceDecl::Let {
            visibility,
            name,
            name_span,
            type_ref,
            body,
            span,
            ..
        } => {
            let no_parameters = Vec::new();
            let context = PureExpressionContext::new(&no_parameters, resolution);
            let value = body
                .as_ref()
                .map(|body| analyze_resolved_expression(body, &context).value)
                .unwrap_or_else(|| hole_expression(span));
            Some(TypedDecl::Let {
                symbol: declaration_symbol(resolution, name_span, SymbolKind::Let, &name),
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
            name_span,
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
                symbol: declaration_symbol(
                    resolution,
                    name_span,
                    SymbolKind::EffectFunction,
                    &name,
                ),
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
            name_span,
            type_parameters,
            parameters,
            return_type,
            constraints,
            body,
            span,
            ..
        } => {
            let typed_parameters = typed_parameters_from_surface(&parameters);
            let context = PureExpressionContext::new(&typed_parameters, resolution);
            let body = body
                .as_ref()
                .map(|body| analyze_resolved_expression(body, &context).value)
                .unwrap_or_else(|| hole_expression(span));
            Some(TypedDecl::Fn {
                symbol: declaration_symbol(resolution, name_span, SymbolKind::Function, &name),
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
        SurfaceDecl::Type {
            visibility,
            opaque,
            name,
            name_span,
            type_parameters,
            variants,
            span,
            ..
        } => typed_adt_decl(
            resolution,
            AdtDeclInput {
                visibility,
                opaque,
                name,
                name_span,
                type_parameters,
                variants,
                origin: span,
            },
        ),
        SurfaceDecl::Newtype { .. }
        | SurfaceDecl::Alias { .. }
        | SurfaceDecl::Struct { .. }
        | SurfaceDecl::Trait { .. }
        | SurfaceDecl::Operator { .. }
        | SurfaceDecl::Instance { .. } => None,
    }
}

fn declaration_symbol(
    resolution: &TypedResolution<'_>,
    origin: seseragi_syntax::ByteSpan,
    kind: SymbolKind,
    fallback: &str,
) -> String {
    resolution
        .declaration_symbol(origin, kind)
        .and_then(|symbol| symbol.canonical.clone())
        .unwrap_or_else(|| fallback.to_owned())
}

fn hole_expression(origin: seseragi_syntax::ByteSpan) -> TypedExpr {
    TypedExpr::Variable {
        name: String::new(),
        type_ref: TypedType::Hole,
        origin,
    }
}
