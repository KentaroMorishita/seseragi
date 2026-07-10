use super::Resolver;
use crate::{ScopeId, ScopeKind, SymbolKind, SymbolNamespace};
use seseragi_syntax::{SurfaceDoItem, SurfaceExpr, SurfacePattern};

pub(super) fn resolve_expression(
    resolver: &mut Resolver,
    scope: ScopeId,
    expression: &SurfaceExpr,
) {
    match expression {
        SurfaceExpr::Name { name, span } => {
            resolver.reference(scope, SymbolNamespace::Value, name, *span, true);
        }
        SurfaceExpr::Application {
            function, argument, ..
        } => {
            resolve_expression(resolver, scope, function);
            resolve_expression(resolver, scope, argument);
        }
        SurfaceExpr::Tuple { elements, .. } => {
            for element in elements {
                resolve_expression(resolver, scope, element);
            }
        }
        SurfaceExpr::Binary {
            operator,
            operator_span,
            left,
            right,
            ..
        } => {
            resolve_expression(resolver, scope, left);
            resolver.reference(
                scope,
                SymbolNamespace::Operator,
                operator,
                *operator_span,
                false,
            );
            resolve_expression(resolver, scope, right);
        }
        SurfaceExpr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            resolve_expression(resolver, scope, condition);
            resolve_expression(resolver, scope, then_branch);
            resolve_expression(resolver, scope, else_branch);
        }
        SurfaceExpr::Do {
            items,
            result,
            span,
        } => {
            let block_scope = resolver.new_scope(scope, ScopeKind::DoBlock, *span);
            for item in items {
                resolve_do_item(resolver, block_scope, item);
            }
            if let Some(result) = result {
                resolve_expression(resolver, block_scope, result);
            }
        }
        SurfaceExpr::Grouped { value, .. } => resolve_expression(resolver, scope, value),
        SurfaceExpr::Unit { .. }
        | SurfaceExpr::Integer { .. }
        | SurfaceExpr::String { .. }
        | SurfaceExpr::Boolean { .. }
        | SurfaceExpr::Error { .. } => {}
    }
}

fn resolve_do_item(resolver: &mut Resolver, scope: ScopeId, item: &SurfaceDoItem) {
    match item {
        SurfaceDoItem::Bind { pattern, value, .. } | SurfaceDoItem::Let { pattern, value, .. } => {
            resolve_expression(resolver, scope, value);
            resolve_pattern(resolver, scope, pattern);
        }
        SurfaceDoItem::Expression { value, .. } => {
            resolve_expression(resolver, scope, value);
        }
    }
}

fn resolve_pattern(resolver: &mut Resolver, scope: ScopeId, pattern: &SurfacePattern) {
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
        SurfacePattern::Wildcard { .. } | SurfacePattern::Error { .. } => {}
    }
}
