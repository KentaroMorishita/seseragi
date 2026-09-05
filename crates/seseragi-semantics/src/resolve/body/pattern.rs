use super::Resolver;
use crate::{ScopeId, SymbolKind, SymbolNamespace};
use seseragi_syntax::SurfacePattern;

pub(super) fn resolve_pattern(resolver: &mut Resolver, scope: ScopeId, pattern: &SurfacePattern) {
    resolve_pattern_with(resolver, scope, pattern, SymbolKind::PatternBinding);
}

pub(super) fn register_module_pattern(resolver: &mut Resolver, pattern: &SurfacePattern) {
    match pattern {
        SurfacePattern::Name {
            name, name_span, ..
        } => {
            resolver.register_module(SymbolNamespace::Value, SymbolKind::Let, name, *name_span);
        }
        SurfacePattern::Constructor { argument, .. } => {
            if let Some(argument) = argument {
                register_module_pattern(resolver, argument);
            }
        }
        SurfacePattern::Tuple { elements, .. } => {
            for element in elements {
                register_module_pattern(resolver, element);
            }
        }
        SurfacePattern::Array { elements, rest, .. }
        | SurfacePattern::List { elements, rest, .. } => {
            for element in elements {
                register_module_pattern(resolver, element);
            }
            if let Some(rest) = rest {
                register_module_pattern(resolver, rest);
            }
        }
        SurfacePattern::Record { fields, .. } | SurfacePattern::Struct { fields, .. } => {
            for field in fields {
                register_module_pattern(resolver, &field.pattern);
            }
        }
        SurfacePattern::Integer { .. }
        | SurfacePattern::String { .. }
        | SurfacePattern::Char { .. }
        | SurfacePattern::Boolean { .. }
        | SurfacePattern::Wildcard { .. }
        | SurfacePattern::Error { .. } => {}
    }
}

pub(super) fn resolve_pattern_references(
    resolver: &mut Resolver,
    scope: ScopeId,
    pattern: &SurfacePattern,
) {
    match pattern {
        SurfacePattern::Constructor {
            name,
            name_span,
            argument,
            ..
        } => {
            resolver.reference(scope, SymbolNamespace::Value, name, *name_span, true);
            if let Some(argument) = argument {
                resolve_pattern_references(resolver, scope, argument);
            }
        }
        SurfacePattern::Struct {
            name,
            name_span,
            fields,
            ..
        } => {
            resolver.reference(scope, SymbolNamespace::Type, name, *name_span, true);
            for field in fields {
                resolve_pattern_references(resolver, scope, &field.pattern);
            }
        }
        SurfacePattern::Tuple { elements, .. } => {
            for element in elements {
                resolve_pattern_references(resolver, scope, element);
            }
        }
        SurfacePattern::Array { elements, rest, .. }
        | SurfacePattern::List { elements, rest, .. } => {
            for element in elements {
                resolve_pattern_references(resolver, scope, element);
            }
            if let Some(rest) = rest {
                resolve_pattern_references(resolver, scope, rest);
            }
        }
        SurfacePattern::Record { fields, .. } => {
            for field in fields {
                resolve_pattern_references(resolver, scope, &field.pattern);
            }
        }
        SurfacePattern::Integer { .. }
        | SurfacePattern::String { .. }
        | SurfacePattern::Char { .. }
        | SurfacePattern::Boolean { .. }
        | SurfacePattern::Name { .. }
        | SurfacePattern::Wildcard { .. }
        | SurfacePattern::Error { .. } => {}
    }
}

fn resolve_pattern_with(
    resolver: &mut Resolver,
    scope: ScopeId,
    pattern: &SurfacePattern,
    binding_kind: SymbolKind,
) {
    match pattern {
        SurfacePattern::Name {
            name, name_span, ..
        } => {
            resolver.register(
                scope,
                SymbolNamespace::Value,
                binding_kind,
                name,
                None,
                *name_span,
            );
        }
        SurfacePattern::Constructor {
            name,
            name_span,
            argument,
            ..
        } => {
            resolver.reference(scope, SymbolNamespace::Value, name, *name_span, true);
            if let Some(argument) = argument {
                resolve_pattern_with(resolver, scope, argument, binding_kind);
            }
        }
        SurfacePattern::Tuple { elements, .. } => {
            for element in elements {
                resolve_pattern_with(resolver, scope, element, binding_kind);
            }
        }
        SurfacePattern::Array { elements, rest, .. }
        | SurfacePattern::List { elements, rest, .. } => {
            for element in elements {
                resolve_pattern_with(resolver, scope, element, binding_kind);
            }
            if let Some(rest) = rest {
                resolve_pattern_with(resolver, scope, rest, binding_kind);
            }
        }
        SurfacePattern::Record { fields, .. } => {
            for field in fields {
                resolve_pattern_with(resolver, scope, &field.pattern, binding_kind);
            }
        }
        SurfacePattern::Struct {
            name,
            name_span,
            fields,
            ..
        } => {
            resolver.reference(scope, SymbolNamespace::Type, name, *name_span, true);
            for field in fields {
                resolve_pattern_with(resolver, scope, &field.pattern, binding_kind);
            }
        }
        SurfacePattern::Integer { .. }
        | SurfacePattern::String { .. }
        | SurfacePattern::Char { .. }
        | SurfacePattern::Boolean { .. }
        | SurfacePattern::Wildcard { .. }
        | SurfacePattern::Error { .. } => {}
    }
}
