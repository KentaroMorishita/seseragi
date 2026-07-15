use super::super::Resolver;
use crate::{ScopeId, ScopeKind, SymbolKind, SymbolNamespace};
use seseragi_syntax::{ByteSpan, SurfaceParameter, SurfaceRequirement, TypeParameter, TypeRef};

pub(super) fn declaration_scope(
    resolver: &mut Resolver,
    parent: ScopeId,
    type_parameters: &[TypeParameter],
    origin: ByteSpan,
) -> ScopeId {
    let scope = resolver.new_scope(parent, ScopeKind::Declaration, origin);
    register_type_parameters(resolver, scope, type_parameters, origin);
    scope
}

pub(super) fn register_type_parameters(
    resolver: &mut Resolver,
    scope: ScopeId,
    parameters: &[TypeParameter],
    origin: ByteSpan,
) {
    for parameter in parameters {
        resolver.register(
            scope,
            SymbolNamespace::Type,
            SymbolKind::TypeParameter,
            &parameter.name,
            None,
            origin,
        );
    }
}

pub(super) fn register_parameters(
    resolver: &mut Resolver,
    scope: ScopeId,
    parameters: &[SurfaceParameter],
) {
    for parameter in parameters {
        resolve_type_ref(resolver, scope, &parameter.type_ref);
        resolver.register(
            scope,
            SymbolNamespace::Value,
            SymbolKind::Parameter,
            &parameter.name,
            None,
            parameter.name_span,
        );
    }
}

pub(super) fn resolve_requirements(
    resolver: &mut Resolver,
    scope: ScopeId,
    requirements: &[SurfaceRequirement],
) {
    for requirement in requirements {
        if let SurfaceRequirement::Field { type_ref, .. } = requirement {
            resolve_type_ref(resolver, scope, type_ref);
        }
    }
}

pub(super) fn resolve_type_ref(resolver: &mut Resolver, scope: ScopeId, type_ref: &TypeRef) {
    match type_ref {
        TypeRef::Named {
            name,
            arguments,
            span,
        } => {
            resolver.reference(scope, SymbolNamespace::Type, name, *span, true);
            for argument in arguments {
                resolve_type_ref(resolver, scope, argument);
            }
        }
        TypeRef::Record { fields, .. } => {
            for field in fields {
                resolve_type_ref(resolver, scope, &field.type_ref);
            }
        }
        TypeRef::Tuple { elements, .. } => {
            for element in elements {
                resolve_type_ref(resolver, scope, element);
            }
        }
        TypeRef::Function {
            parameter, result, ..
        } => {
            resolve_type_ref(resolver, scope, parameter);
            resolve_type_ref(resolver, scope, result);
        }
        TypeRef::Hole { .. } => {}
    }
}
