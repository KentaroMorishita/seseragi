use crate::{SymbolKind, SymbolNamespace, TypedAdtVariant, TypedDecl, TypedScheme, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceVariant, TypeRef, Visibility};

use super::type_ref::typed_type_from_type_ref;
use super::TypedResolution;

pub(super) struct AdtDeclInput {
    pub(super) visibility: Visibility,
    pub(super) opaque: bool,
    pub(super) name: String,
    pub(super) name_span: ByteSpan,
    pub(super) type_parameters: Vec<String>,
    pub(super) variants: Vec<SurfaceVariant>,
    pub(super) origin: ByteSpan,
}

pub(super) fn typed_adt_decl(
    resolution: &TypedResolution<'_>,
    input: AdtDeclInput,
) -> Option<TypedDecl> {
    let symbol = resolution
        .declaration_symbol(input.name_span, SymbolKind::Type)
        .and_then(|symbol| symbol.canonical.clone())?;
    let result = TypedType::Named {
        name: input.name.clone(),
        arguments: input
            .type_parameters
            .iter()
            .map(|parameter| TypedType::Named {
                name: parameter.clone(),
                arguments: Vec::new(),
            })
            .collect(),
    };
    let variants = input
        .variants
        .into_iter()
        .map(|variant| typed_variant(resolution, &input.type_parameters, &result, variant))
        .collect::<Option<Vec<_>>>()?;
    Some(TypedDecl::Adt {
        symbol,
        name: input.name,
        visibility: input.visibility,
        opaque: input.opaque,
        type_parameters: input.type_parameters,
        variants,
        origin: input.origin,
    })
}

fn typed_variant(
    resolution: &TypedResolution<'_>,
    type_parameters: &[String],
    result: &TypedType,
    variant: SurfaceVariant,
) -> Option<TypedAdtVariant> {
    let symbol = resolution
        .declaration_symbol(variant.name_span, SymbolKind::Constructor)
        .and_then(|symbol| symbol.canonical.clone())?;
    if variant
        .payload
        .as_ref()
        .is_some_and(|payload| !type_ref_is_resolved(resolution, payload))
    {
        return None;
    }
    let payload = variant.payload.as_ref().map(typed_type_from_type_ref);
    let type_ref = payload.as_ref().map_or_else(
        || result.clone(),
        |payload| TypedType::Function {
            parameter: Box::new(payload.clone()),
            result: Box::new(result.clone()),
        },
    );
    Some(TypedAdtVariant {
        symbol,
        name: variant.name,
        payload,
        scheme: TypedScheme {
            type_parameters: type_parameters.to_vec(),
            constraints: Vec::new(),
            type_ref,
        },
        origin: variant.name_span,
    })
}

fn type_ref_is_resolved(resolution: &TypedResolution<'_>, type_ref: &TypeRef) -> bool {
    match type_ref {
        TypeRef::Named {
            name,
            arguments,
            span,
        } => {
            (name.contains('.') || resolution.target(*span, SymbolNamespace::Type).is_some())
                && arguments
                    .iter()
                    .all(|argument| type_ref_is_resolved(resolution, argument))
        }
        TypeRef::Record { fields, .. } => fields
            .iter()
            .all(|field| type_ref_is_resolved(resolution, &field.type_ref)),
        TypeRef::Tuple { elements, .. } => elements
            .iter()
            .all(|element| type_ref_is_resolved(resolution, element)),
        TypeRef::Function {
            parameter, result, ..
        } => {
            type_ref_is_resolved(resolution, parameter) && type_ref_is_resolved(resolution, result)
        }
        TypeRef::Hole { .. } => false,
    }
}
