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
        // Type parameters remain in scope through the declaration, including
        // nested methods and local declarations (spec 2.11). A fresh inner
        // binder cannot reuse an enclosing parameter's spelling. Keep this
        // separate from ordinary nominal/value shadowing and sibling scopes.
        if resolver
            .lookup(scope, SymbolNamespace::Type, &parameter.name)
            .is_some_and(|id| {
                let existing = &resolver.symbols[id.0 as usize];
                existing.kind == SymbolKind::TypeParameter && existing.scope != scope
            })
        {
            resolver.issues.push(crate::ResolveIssue {
                code: "SES-N0002".into(),
                message_key: "name.duplicate-definition".into(),
                primary: origin,
            });
        }
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
        match requirement {
            SurfaceRequirement::Shorthand { name, span } => {
                // Shorthand service requirements may be supplied by provider metadata.
                // Retain any actual lexical owner without adding a new unresolved-name
                // rule for those existing implicit service requirements.
                resolver.reference(scope, SymbolNamespace::Type, name, *span, false);
            }
            SurfaceRequirement::Field { type_ref, .. } => {
                resolve_type_ref(resolver, scope, type_ref);
            }
        }
    }
}

pub(in crate::resolve::body) fn resolve_type_ref(
    resolver: &mut Resolver,
    scope: ScopeId,
    type_ref: &TypeRef,
) {
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
        TypeRef::RequirementMerge { operands, .. } => {
            for operand in operands {
                resolve_type_ref(resolver, scope, operand);
            }
        }
        TypeRef::Hole { .. } => {}
    }
}
