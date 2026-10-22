//! One synchronous parse owns its limits. Recursive helper parsers share the
//! same thread-local scope; constructing a new range parser cannot reset it.
use crate::{
    ByteSpan, SurfaceBlockItem, SurfaceComprehensionClause, SurfaceDoItem, SurfaceExpr,
    SurfaceTemplatePart,
};
use std::cell::RefCell;

pub(crate) const RECURSION_LIMIT: usize = 64;
pub(crate) const EXPRESSION_DEPTH_LIMIT: usize = 128;
const WORK_LIMIT: usize = 1_000_000;

#[derive(Default)]
struct Budget {
    depth: usize,
    work: usize,
    exceeded: Option<ByteSpan>,
}
thread_local! { static BUDGET: RefCell<Option<Budget>> = const { RefCell::new(None) }; }

pub(crate) struct ParseScope(bool);
impl ParseScope {
    pub(crate) fn new() -> Self {
        Self(BUDGET.with_borrow_mut(|current| {
            if current.is_some() {
                false
            } else {
                *current = Some(Budget::default());
                true
            }
        }))
    }
}
impl Drop for ParseScope {
    fn drop(&mut self) {
        if self.0 {
            BUDGET.with_borrow_mut(|current| *current = None);
        }
    }
}

pub(crate) struct Frame(bool);
pub(crate) fn enter(span: ByteSpan) -> Option<Frame> {
    BUDGET.with_borrow_mut(|current| {
        let Some(budget) = current else {
            return Some(Frame(false));
        };
        if budget.exceeded.is_some() {
            return None;
        }
        budget.work += 1;
        if budget.depth >= RECURSION_LIMIT || budget.work > WORK_LIMIT {
            budget.exceeded = Some(span);
            return None;
        }
        budget.depth += 1;
        Some(Frame(true))
    })
}
impl Drop for Frame {
    fn drop(&mut self) {
        if self.0 {
            BUDGET.with_borrow_mut(|current| current.as_mut().unwrap().depth -= 1);
        }
    }
}
pub(crate) fn exceeded() -> Option<ByteSpan> {
    BUDGET.with_borrow(|current| current.as_ref().and_then(|budget| budget.exceeded))
}
#[cfg(test)]
pub(crate) fn spend_all_but_one() {
    BUDGET.with_borrow_mut(|current| current.as_mut().unwrap().work = WORK_LIMIT - 1);
}

pub(crate) fn exhaust(span: ByteSpan) {
    BUDGET.with_borrow_mut(|current| {
        if let Some(budget) = current {
            budget.exceeded.get_or_insert(span);
        }
    });
}

/// Use an explicit stack for the check itself. Flat applications and operator
/// chains are capped during construction, before a deeply recursive drop can
/// happen even on a syntax-error path.
pub(crate) fn check_expression(root: &SurfaceExpr) -> bool {
    let mut pending = vec![(root, 1)];
    while let Some((expression, depth)) = pending.pop() {
        if depth > EXPRESSION_DEPTH_LIMIT {
            exhaust(expression.span());
            return false;
        }
        let spent = BUDGET.with_borrow_mut(|current| {
            let Some(budget) = current else {
                return true;
            };
            budget.work += 1;
            if budget.work > WORK_LIMIT {
                budget.exceeded.get_or_insert(expression.span());
                false
            } else {
                true
            }
        });
        if !spent {
            return false;
        }
        let next = depth + 1;
        match expression {
            SurfaceExpr::Template { parts, .. } => {
                for part in parts {
                    if let SurfaceTemplatePart::Interpolation { value, .. } = part {
                        pending.push((value.as_ref(), next));
                    }
                }
            }
            SurfaceExpr::Index {
                receiver, index, ..
            } => {
                pending.push((receiver.as_ref(), next));
                pending.push((index.as_ref(), next));
            }
            SurfaceExpr::Member { receiver, .. } => pending.push((receiver.as_ref(), next)),
            SurfaceExpr::Application {
                function, argument, ..
            } => {
                pending.push((function.as_ref(), next));
                pending.push((argument.as_ref(), next));
            }
            SurfaceExpr::Prefix { operand, .. } => pending.push((operand.as_ref(), next)),
            SurfaceExpr::Assignment { target, value, .. } => {
                pending.push((target.as_ref(), next));
                pending.push((value.as_ref(), next));
            }
            SurfaceExpr::Lambda { body, .. } => pending.push((body.as_ref(), next)),
            SurfaceExpr::EffectfulFor { source, body, .. } => {
                pending.push((source.as_ref(), next));
                pending.push((body.as_ref(), next));
            }
            SurfaceExpr::Tuple { elements, .. }
            | SurfaceExpr::Array { elements, .. }
            | SurfaceExpr::List { elements, .. } => {
                pending.extend(elements.iter().map(|value| (value, next)))
            }
            SurfaceExpr::Record { items, .. } | SurfaceExpr::Struct { items, .. } => {
                pending.extend(items.iter().map(|item| (item.value(), next)))
            }
            SurfaceExpr::ArrayComprehension {
                element, clauses, ..
            }
            | SurfaceExpr::ListComprehension {
                element, clauses, ..
            } => {
                pending.push((element.as_ref(), next));
                pending.extend(clauses.iter().map(|clause| match clause {
                    SurfaceComprehensionClause::Generator { source, .. } => (source, next),
                    SurfaceComprehensionClause::Guard { condition, .. } => (condition, next),
                }));
            }
            SurfaceExpr::Binary { left, right, .. } => {
                pending.push((left.as_ref(), next));
                pending.push((right.as_ref(), next));
            }
            SurfaceExpr::InfixChain { first, steps, .. } => {
                // Fixity resolution later replaces each custom operator with
                // two curried Application nodes. Charge its worst association
                // depth now, including chains nested inside grouped operands.
                // The flat parser representation alone would hide that depth.
                let projected = depth + 2 * steps.len();
                pending.push((first.as_ref(), projected));
                pending.extend(steps.iter().map(|step| (&step.operand, projected)));
            }
            SurfaceExpr::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                pending.push((condition.as_ref(), next));
                pending.push((then_branch.as_ref(), next));
                pending.push((else_branch.as_ref(), next));
            }
            SurfaceExpr::Match {
                scrutinee, arms, ..
            } => {
                pending.push((scrutinee.as_ref(), next));
                for arm in arms {
                    pending.push((&arm.body, next));
                    if let Some(guard) = &arm.guard {
                        pending.push((guard, next));
                    }
                }
            }
            SurfaceExpr::Block { items, result, .. } => {
                pending.push((result.as_ref(), next));
                pending.extend(items.iter().map(|item| match item {
                    SurfaceBlockItem::Let { value, .. }
                    | SurfaceBlockItem::Function { value, .. } => (value, next),
                }));
            }
            SurfaceExpr::Do { items, result, .. } => {
                if let Some(result) = result {
                    pending.push((result.as_ref(), next));
                }
                pending.extend(items.iter().map(|item| match item {
                    SurfaceDoItem::Bind { value, .. }
                    | SurfaceDoItem::Let { value, .. }
                    | SurfaceDoItem::Expression { value, .. } => (value, next),
                }));
            }
            SurfaceExpr::Grouped { value, .. } => pending.push((value.as_ref(), next)),
            SurfaceExpr::Unit { .. }
            | SurfaceExpr::Integer { .. }
            | SurfaceExpr::Float { .. }
            | SurfaceExpr::Char { .. }
            | SurfaceExpr::String { .. }
            | SurfaceExpr::Boolean { .. }
            | SurfaceExpr::Name { .. }
            | SurfaceExpr::Error { .. } => {}
        }
    }
    true
}
