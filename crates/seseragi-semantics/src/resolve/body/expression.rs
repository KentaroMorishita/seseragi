use super::Resolver;
use crate::{ScopeId, ScopeKind, SymbolNamespace};
use seseragi_syntax::{SurfaceComprehensionClause, SurfaceDoItem, SurfaceExpr};

use super::pattern::resolve_pattern;

pub(super) fn resolve_expression(
    resolver: &mut Resolver,
    scope: ScopeId,
    expression: &SurfaceExpr,
) {
    match expression {
        SurfaceExpr::Name { name, span } => {
            let namespace = if is_operator_reference(name) {
                SymbolNamespace::Operator
            } else {
                SymbolNamespace::Value
            };
            resolver.reference(scope, namespace, name, *span, true);
        }
        SurfaceExpr::Application {
            function, argument, ..
        } => {
            resolve_expression(resolver, scope, function);
            resolve_expression(resolver, scope, argument);
        }
        SurfaceExpr::Tuple { elements, .. } | SurfaceExpr::Array { elements, .. } => {
            for element in elements {
                resolve_expression(resolver, scope, element);
            }
        }
        SurfaceExpr::ArrayComprehension {
            element,
            clauses,
            span,
        } => {
            let mut clause_scope = scope;
            for clause in clauses {
                match clause {
                    SurfaceComprehensionClause::Generator {
                        pattern,
                        source,
                        span,
                    } => {
                        resolve_expression(resolver, clause_scope, source);
                        clause_scope =
                            resolver.new_scope(clause_scope, ScopeKind::Comprehension, *span);
                        resolve_pattern(resolver, clause_scope, pattern);
                    }
                    SurfaceComprehensionClause::Guard { condition, .. } => {
                        resolve_expression(resolver, clause_scope, condition);
                    }
                }
            }
            if clauses.is_empty() {
                clause_scope = resolver.new_scope(scope, ScopeKind::Comprehension, *span);
            }
            resolve_expression(resolver, clause_scope, element);
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
        SurfaceExpr::Match {
            scrutinee, arms, ..
        } => {
            resolve_expression(resolver, scope, scrutinee);
            for arm in arms {
                let arm_scope = resolver.new_scope(scope, ScopeKind::MatchArm, arm.span);
                resolve_pattern(resolver, arm_scope, &arm.pattern);
                if let Some(guard) = &arm.guard {
                    resolve_expression(resolver, arm_scope, guard);
                }
                resolve_expression(resolver, arm_scope, &arm.body);
            }
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

fn is_operator_reference(name: &str) -> bool {
    matches!(name, "+" | "-" | "*" | "/" | "%" | "**")
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
