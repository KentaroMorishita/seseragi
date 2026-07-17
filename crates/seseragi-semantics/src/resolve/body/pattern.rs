use super::Resolver;
use crate::{ScopeId, SymbolKind, SymbolNamespace};
use seseragi_syntax::SurfacePattern;

pub(super) fn resolve_pattern(resolver: &mut Resolver, scope: ScopeId, pattern: &SurfacePattern) {
    match pattern {
        SurfacePattern::Name {
            name, name_span, ..
        } => {
            resolver.register(
                scope,
                SymbolNamespace::Value,
                SymbolKind::PatternBinding,
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
                resolve_pattern(resolver, scope, argument);
            }
        }
        SurfacePattern::Tuple { elements, .. } => {
            for element in elements {
                resolve_pattern(resolver, scope, element);
            }
        }
        SurfacePattern::Record { fields, .. } => {
            for field in fields {
                resolve_pattern(resolver, scope, &field.pattern);
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
                resolve_pattern(resolver, scope, &field.pattern);
            }
        }
        SurfacePattern::Integer { .. }
        | SurfacePattern::String { .. }
        | SurfacePattern::Boolean { .. }
        | SurfacePattern::Wildcard { .. }
        | SurfacePattern::Error { .. } => {}
    }
}
