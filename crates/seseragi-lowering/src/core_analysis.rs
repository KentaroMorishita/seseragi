//! Conservative target-independent facts, without rewriting CoreIr or ABI.
use crate::{CoreExpr, CoreInstanceEvidence, CoreInstanceImplementation, CoreModule, SourceSpan};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreCallShape {
    Partial,
    Saturated,
    ReturnedApplication,
}

/// Uses declaration parameters, never the arrows in the return type.
/// Callers must first exclude lexical shadowing and staged/deferred evidence.
pub fn known_pure_call_shape(
    module: &CoreModule,
    callee: &str,
    arguments: usize,
) -> Option<CoreCallShape> {
    let function = module
        .functions
        .iter()
        .find(|f| f.symbol == callee && !f.is_effect)?;
    Some(match arguments.cmp(&function.parameters.len()) {
        std::cmp::Ordering::Less => CoreCallShape::Partial,
        std::cmp::Ordering::Equal => CoreCallShape::Saturated,
        std::cmp::Ordering::Greater => CoreCallShape::ReturnedApplication,
    })
}

/// A local method dictionary whose dependencies are also closed local trees.
/// Imported/standard implementations need their own purity/ABI proof. This
/// fact alone does not authorize moving evaluation: a backend must prove its
/// factory only constructs method closures, and must preserve initialization.
pub fn is_closed_local_evidence(module: &CoreModule, evidence: &CoreInstanceEvidence) -> bool {
    let CoreInstanceEvidence::Local {
        identity,
        evidence_arguments,
        ..
    } = evidence
    else {
        return false;
    };
    let Some(instance) = module.instances.iter().find(|i| &i.identity == identity) else {
        return false;
    };
    let CoreInstanceImplementation::UserDefined { methods } = &instance.implementation else {
        return false;
    };
    instance.supertrait_count == 0
        && instance.constraints.len() == evidence_arguments.len()
        && methods
            .iter()
            .all(|m| !m.parameters.is_empty() && m.constraints.is_empty())
        && evidence_arguments
            .iter()
            .all(|e| is_closed_local_evidence(module, &e.evidence))
}

pub fn core_expr_origin(expr: &CoreExpr) -> &SourceSpan {
    match expr {
        CoreExpr::Unit { origin, .. }
        | CoreExpr::Integer { origin, .. }
        | CoreExpr::Float64 { origin, .. }
        | CoreExpr::Char { origin, .. }
        | CoreExpr::String { origin, .. }
        | CoreExpr::Template { origin, .. }
        | CoreExpr::Boolean { origin, .. }
        | CoreExpr::Variable { origin, .. }
        | CoreExpr::Call { origin, .. }
        | CoreExpr::Lambda { origin, .. }
        | CoreExpr::Tuple { origin, .. }
        | CoreExpr::FieldAccess { origin, .. }
        | CoreExpr::OptionalFieldAccess { origin, .. }
        | CoreExpr::Record { origin, .. }
        | CoreExpr::Array { origin, .. }
        | CoreExpr::List { origin, .. }
        | CoreExpr::ArrayComprehension { origin, .. }
        | CoreExpr::ListComprehension { origin, .. }
        | CoreExpr::Binary { origin, .. }
        | CoreExpr::Unary { origin, .. }
        | CoreExpr::If { origin, .. }
        | CoreExpr::Decision { origin, .. }
        | CoreExpr::EffectOperation { origin, .. }
        | CoreExpr::EffectInvoke { origin, .. }
        | CoreExpr::Sequence { origin, .. }
        | CoreExpr::MonadDo { origin, .. } => origin,
    }
}

/// Free term names and hidden evidence parameters for the pure expression
/// slice. None means the caller must conservatively retain its whole scope.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CoreFreeVariables {
    pub names: std::collections::BTreeSet<String>,
    pub evidence_parameters: std::collections::BTreeSet<usize>,
}
impl CoreFreeVariables {
    fn merge(&mut self, other: Self) {
        self.names.extend(other.names);
        self.evidence_parameters.extend(other.evidence_parameters);
    }
    fn evidence(&mut self, evidence: &[crate::CoreCallEvidence]) {
        for selected in evidence {
            match &selected.evidence {
                CoreInstanceEvidence::Parameter { index } => {
                    self.evidence_parameters.insert(*index);
                }
                CoreInstanceEvidence::Local {
                    evidence_arguments, ..
                }
                | CoreInstanceEvidence::Imported {
                    evidence_arguments, ..
                }
                | CoreInstanceEvidence::Standard {
                    evidence_arguments, ..
                } => self.evidence(evidence_arguments),
            }
        }
    }
}
pub fn core_free_variables(expr: &CoreExpr) -> Option<CoreFreeVariables> {
    let mut free = CoreFreeVariables::default();
    match expr {
        CoreExpr::Unit { .. }
        | CoreExpr::Integer { .. }
        | CoreExpr::Float64 { .. }
        | CoreExpr::Char { .. }
        | CoreExpr::String { .. }
        | CoreExpr::Boolean { .. } => {}
        CoreExpr::Variable { name, evidence, .. } => {
            free.names.insert(name.clone());
            free.evidence(evidence);
        }
        CoreExpr::Call {
            callee,
            arguments,
            evidence,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            ..
        } => {
            if !deferred_evidence_parameters.is_empty()
                || !deferred_evidence_type_constructor_parameters.is_empty()
                || (evidence_argument_index.is_some() && !evidence.is_empty())
            {
                return None;
            }
            free.names.insert(callee.clone());
            free.evidence(evidence);
            for arg in arguments {
                free.merge(core_free_variables(arg)?);
            }
        }
        CoreExpr::Lambda {
            parameter, body, ..
        } => {
            free = core_free_variables(body)?;
            free.names.remove(&parameter.id);
        }
        CoreExpr::Tuple { elements, .. } | CoreExpr::Array { elements, .. } => {
            for element in elements {
                free.merge(core_free_variables(element)?);
            }
        }
        CoreExpr::FieldAccess { receiver, .. } => free.merge(core_free_variables(receiver)?),
        CoreExpr::Record { items, .. } => {
            for item in items {
                match item {
                    crate::CoreRecordValueItem::Field { value, .. } => {
                        free.merge(core_free_variables(value)?)
                    }
                    _ => return None,
                }
            }
        }
        CoreExpr::Binary {
            left,
            right,
            evidence,
            ..
        } => {
            free.merge(core_free_variables(left)?);
            free.merge(core_free_variables(right)?);
            free.evidence(evidence);
        }
        CoreExpr::Unary { operand, .. } => free.merge(core_free_variables(operand)?),
        CoreExpr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            for value in [condition, then_branch, else_branch] {
                free.merge(core_free_variables(value)?);
            }
        }
        CoreExpr::Decision {
            scrutinee,
            branches,
            ..
        } => {
            free.merge(core_free_variables(scrutinee)?);
            for branch in branches {
                let mut inner = core_free_variables(&branch.value)?;
                if let Some(guard) = &branch.guard {
                    inner.merge(core_free_variables(guard)?);
                }
                for binding in &branch.bindings {
                    inner.names.remove(&binding.name);
                }
                free.merge(inner);
            }
        }
        CoreExpr::Sequence {
            statements, result, ..
        } => {
            // Walk backwards: each initializer sees the preceding scope, while
            // the binding scopes over subsequent statements and the result.
            free = core_free_variables(result)?;
            for statement in statements.iter().rev() {
                match statement {
                    crate::CoreStatement::PureLet { name, value, .. } => {
                        free.names.remove(name);
                        free.merge(core_free_variables(value)?);
                    }
                    _ => return None,
                }
            }
        }
        _ => return None,
    }
    Some(free)
}
