use crate::{SymbolKind, TypedDecl, TypedExpr, TypedScheme, TypedStructField, TypedType};
use seseragi_syntax::{SurfaceDecl, SurfaceImplMember, SurfaceVariant};

use super::adt::{typed_adt_decl, AdtDeclInput};
use super::effect::typed_effect_from_surface;
use super::effect_body::typed_effect_body;
use super::functions::typed_parameters_from_surface;
use super::surface_expr::{analyze_resolved_expression, PureExpressionContext};
use super::type_ref::{inferred_type_from_expr, typed_type_from_type_ref};
use super::TypedResolution;

pub(crate) fn typed_decls_from_surface(
    declaration: SurfaceDecl,
    resolution: &TypedResolution<'_>,
) -> Vec<TypedDecl> {
    match declaration {
        SurfaceDecl::Impl {
            type_parameters,
            target,
            constraints,
            members,
            ..
        } => members
            .into_iter()
            .filter_map(|member| {
                let SurfaceImplMember::Method { visibility, method } = member else {
                    return None;
                };
                let symbol = resolution.inherent_method_symbol(&target, &method.name)?;
                let mut combined_type_parameters = type_parameters.clone();
                combined_type_parameters.extend(method.type_parameters.clone());
                let mut combined_constraints = constraints.clone();
                combined_constraints.extend(method.constraints.clone());
                let typed_parameters = typed_parameters_from_surface(&method.parameters);
                let scoped_evidence =
                    crate::typed::scoped_call_evidence(&combined_constraints, resolution);
                let context = PureExpressionContext::new(&typed_parameters, resolution)
                    .with_evidence_parameters(scoped_evidence)
                    .with_expected(Some(
                        resolution.semantic_value_from_type_ref(&method.return_type),
                    ));
                let body = method
                    .body
                    .as_ref()
                    .map(|body| analyze_resolved_expression(body, &context).value)
                    .unwrap_or_else(|| hole_expression(method.span));
                Some(TypedDecl::Fn {
                    symbol,
                    visibility,
                    origin: method.span,
                    type_constructor_parameters: combined_type_parameters
                        .iter()
                        .filter(|parameter| parameter.is_constructor())
                        .cloned()
                        .collect(),
                    scheme: TypedScheme {
                        type_parameters: combined_type_parameters
                            .into_iter()
                            .map(|parameter| parameter.name)
                            .collect(),
                        constraints: combined_constraints
                            .into_iter()
                            .map(|constraint| crate::TypedConstraint {
                                name: constraint.name,
                                arguments: constraint
                                    .arguments
                                    .iter()
                                    .map(typed_type_from_type_ref)
                                    .collect(),
                            })
                            .collect(),
                        type_ref: typed_type_from_type_ref(&method.return_type),
                    },
                    parameters: typed_parameters,
                    body,
                })
            })
            .collect(),
        declaration => typed_decl_from_surface(declaration, resolution)
            .into_iter()
            .collect(),
    }
}

pub(crate) fn typed_decl_from_surface(
    declaration: SurfaceDecl,
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
            let context = PureExpressionContext::new(&no_parameters, resolution).with_expected(
                type_ref
                    .as_ref()
                    .map(|type_ref| resolution.semantic_value_from_type_ref(type_ref)),
            );
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
            requirements,
            failure,
            inferred_contract,
            body,
            span,
            ..
        } => {
            let typed_parameters = typed_parameters_from_surface(&parameters);
            let body = body
                .as_ref()
                .map(|body| typed_effect_body(body, &typed_parameters, resolution))
                .unwrap_or_else(|| hole_expression(span));
            let effect = typed_effect_from_surface(
                &return_type,
                &requirements,
                failure.as_ref(),
                inferred_contract,
                &body,
            );
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
                parameters: typed_parameters,
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
            let scoped_evidence = crate::typed::scoped_call_evidence(&constraints, resolution);
            let context = PureExpressionContext::new(&typed_parameters, resolution)
                .with_evidence_parameters(scoped_evidence)
                .with_expected(Some(resolution.semantic_value_from_type_ref(&return_type)));
            let body = body
                .as_ref()
                .map(|body| analyze_resolved_expression(body, &context).value)
                .unwrap_or_else(|| hole_expression(span));
            let type_constructor_parameters = type_parameters
                .iter()
                .filter(|parameter| parameter.is_constructor())
                .cloned()
                .collect();
            Some(TypedDecl::Fn {
                symbol: declaration_symbol(resolution, name_span, SymbolKind::Function, &name),
                visibility,
                origin: span,
                type_constructor_parameters,
                scheme: TypedScheme {
                    type_parameters: type_parameters
                        .into_iter()
                        .map(|parameter| parameter.name)
                        .collect(),
                    constraints: constraints
                        .into_iter()
                        .map(|constraint| crate::TypedConstraint {
                            name: constraint.name,
                            arguments: constraint
                                .arguments
                                .iter()
                                .map(typed_type_from_type_ref)
                                .collect(),
                        })
                        .collect(),
                    type_ref: typed_type_from_type_ref(&return_type),
                },
                parameters: typed_parameters,
                body,
            })
        }
        SurfaceDecl::Operator {
            visibility,
            spelling,
            spelling_span,
            type_parameters,
            parameters,
            return_type,
            constraints,
            body,
            span,
            ..
        } => {
            let typed_parameters = typed_parameters_from_surface(&parameters);
            let scoped_evidence = crate::typed::scoped_call_evidence(&constraints, resolution);
            let context = PureExpressionContext::new(&typed_parameters, resolution)
                .with_evidence_parameters(scoped_evidence)
                .with_expected(Some(resolution.semantic_value_from_type_ref(&return_type)));
            let body = body
                .as_ref()
                .map(|body| analyze_resolved_expression(body, &context).value)
                .unwrap_or_else(|| hole_expression(span));
            let type_constructor_parameters = type_parameters
                .iter()
                .filter(|parameter| parameter.is_constructor())
                .cloned()
                .collect();
            Some(TypedDecl::Fn {
                symbol: declaration_symbol(
                    resolution,
                    spelling_span,
                    SymbolKind::Operator,
                    &spelling,
                ),
                visibility,
                origin: span,
                type_constructor_parameters,
                scheme: TypedScheme {
                    type_parameters: type_parameters
                        .into_iter()
                        .map(|parameter| parameter.name)
                        .collect(),
                    constraints: constraints
                        .into_iter()
                        .map(|constraint| crate::TypedConstraint {
                            name: constraint.name,
                            arguments: constraint
                                .arguments
                                .iter()
                                .map(typed_type_from_type_ref)
                                .collect(),
                        })
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
                type_parameters: type_parameters
                    .into_iter()
                    .map(|parameter| parameter.name)
                    .collect(),
                variants,
                origin: span,
            },
        ),
        SurfaceDecl::Newtype {
            visibility,
            opaque,
            name,
            name_span,
            type_parameters,
            representation,
            span,
            ..
        } => typed_adt_decl(
            resolution,
            AdtDeclInput {
                visibility,
                opaque,
                name: name.clone(),
                name_span,
                type_parameters: type_parameters
                    .into_iter()
                    .map(|parameter| parameter.name)
                    .collect(),
                variants: vec![SurfaceVariant {
                    name,
                    name_span,
                    payload: Some(representation),
                    span: name_span,
                }],
                origin: span,
            },
        ),
        SurfaceDecl::Struct {
            visibility,
            opaque,
            name,
            name_span,
            type_parameters,
            fields,
            span,
            ..
        } => Some(TypedDecl::Struct {
            symbol: declaration_symbol(resolution, name_span, SymbolKind::Type, &name),
            name,
            visibility,
            opaque,
            type_parameters: type_parameters
                .into_iter()
                .map(|parameter| parameter.name)
                .collect(),
            fields: fields
                .into_iter()
                .map(|field| TypedStructField {
                    name: field.name,
                    type_ref: typed_type_from_type_ref(&field.type_ref),
                    origin: field.name_span,
                })
                .collect(),
            origin: span,
        }),
        SurfaceDecl::Alias { .. } | SurfaceDecl::Trait { .. } | SurfaceDecl::Instance { .. } => {
            None
        }
        SurfaceDecl::Impl { .. } => unreachable!("impl declarations expand before this point"),
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
        evidence: Vec::new(),
        type_ref: TypedType::Hole,
        origin,
    }
}
