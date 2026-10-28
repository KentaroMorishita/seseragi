use super::*;
use seseragi_syntax::{
    SurfaceBlockItem, SurfaceComprehensionClause, SurfaceDoItem, SurfaceExpr, SurfaceImplMember,
    SurfaceTemplatePart,
};

/// Source provenance belongs to analysis, not the typed function contract. The
/// exact expression span joins this table to a selected typed call. Generated
/// calls without a corresponding source callee acquire no invented location.
pub(super) fn callee_ranges(resolved: &ResolvedModule) -> BTreeMap<(usize, usize), ByteSpan> {
    let mut result = BTreeMap::new();
    for declaration in &resolved.declarations {
        let bodies = match declaration {
            SurfaceDecl::Let { body, .. }
            | SurfaceDecl::Fn { body, .. }
            | SurfaceDecl::EffectFn { body, .. } => body.iter().collect::<Vec<_>>(),
            SurfaceDecl::Instance { methods, .. } | SurfaceDecl::Trait { methods, .. } => methods
                .iter()
                .filter_map(|method| method.body.as_ref())
                .collect(),
            SurfaceDecl::Impl { members, .. } => members
                .iter()
                .filter_map(|member| match member {
                    SurfaceImplMember::Method { method, .. } => method.body.as_ref(),
                    SurfaceImplMember::Operator { body, .. } => body.as_ref(),
                })
                .collect(),
            _ => Vec::new(),
        };
        for body in bodies {
            visit(body, &mut |expression| {
                if let Some(callee) = callee(expression) {
                    let span = expression.span();
                    result.insert((span.start, span.end), callee);
                }
            });
        }
    }
    result
}

fn callee(expression: &SurfaceExpr) -> Option<ByteSpan> {
    match expression {
        SurfaceExpr::Application { function, .. }
        | SurfaceExpr::Grouped {
            value: function, ..
        } => callee(function),
        SurfaceExpr::Name { span, .. } => Some(*span),
        SurfaceExpr::Member { field_span, .. } => Some(*field_span),
        SurfaceExpr::Binary { operator_span, .. } | SurfaceExpr::Prefix { operator_span, .. } => {
            Some(*operator_span)
        }
        _ => None,
    }
}

fn visit(expression: &SurfaceExpr, action: &mut impl FnMut(&SurfaceExpr)) {
    action(expression);
    match expression {
        SurfaceExpr::Template { parts, .. } => {
            for part in parts {
                if let SurfaceTemplatePart::Interpolation { value, .. } = part {
                    visit(value, action);
                }
            }
        }
        SurfaceExpr::Member { receiver, .. }
        | SurfaceExpr::Prefix {
            operand: receiver, ..
        }
        | SurfaceExpr::Grouped {
            value: receiver, ..
        } => visit(receiver, action),
        SurfaceExpr::Lambda { body, .. } => visit(body, action),
        SurfaceExpr::Application {
            function, argument, ..
        }
        | SurfaceExpr::Assignment {
            target: function,
            value: argument,
            ..
        }
        | SurfaceExpr::Index {
            receiver: function,
            index: argument,
            ..
        }
        | SurfaceExpr::Binary {
            left: function,
            right: argument,
            ..
        } => {
            visit(function, action);
            visit(argument, action);
        }
        SurfaceExpr::EffectfulFor { source, body, .. } => {
            visit(source, action);
            visit(body, action);
        }
        SurfaceExpr::Tuple { elements, .. }
        | SurfaceExpr::Array { elements, .. }
        | SurfaceExpr::List { elements, .. } => {
            for element in elements {
                visit(element, action);
            }
        }
        SurfaceExpr::Record { items, .. } | SurfaceExpr::Struct { items, .. } => {
            for item in items {
                visit(item.value(), action);
            }
        }
        SurfaceExpr::ArrayComprehension {
            element, clauses, ..
        }
        | SurfaceExpr::ListComprehension {
            element, clauses, ..
        } => {
            visit(element, action);
            for clause in clauses {
                match clause {
                    SurfaceComprehensionClause::Generator { source, .. } => visit(source, action),
                    SurfaceComprehensionClause::Guard { condition, .. } => visit(condition, action),
                }
            }
        }
        SurfaceExpr::InfixChain { first, steps, .. } => {
            visit(first, action);
            for step in steps {
                visit(&step.operand, action);
            }
        }
        SurfaceExpr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            visit(condition, action);
            visit(then_branch, action);
            visit(else_branch, action);
        }
        SurfaceExpr::Match {
            scrutinee, arms, ..
        } => {
            visit(scrutinee, action);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    visit(guard, action);
                }
                visit(&arm.body, action);
            }
        }
        SurfaceExpr::Block { items, result, .. } => {
            for item in items {
                match item {
                    SurfaceBlockItem::Let { value, .. }
                    | SurfaceBlockItem::Function { value, .. } => visit(value, action),
                }
            }
            visit(result, action);
        }
        SurfaceExpr::Do { items, result, .. } => {
            for item in items {
                match item {
                    SurfaceDoItem::Bind { value, .. }
                    | SurfaceDoItem::Let { value, .. }
                    | SurfaceDoItem::Expression { value, .. } => visit(value, action),
                }
            }
            if let Some(result) = result {
                visit(result, action);
            }
        }
        SurfaceExpr::Unit { .. }
        | SurfaceExpr::Integer { .. }
        | SurfaceExpr::Float { .. }
        | SurfaceExpr::String { .. }
        | SurfaceExpr::Char { .. }
        | SurfaceExpr::Boolean { .. }
        | SurfaceExpr::Name { .. }
        | SurfaceExpr::Error { .. } => {}
    }
}
