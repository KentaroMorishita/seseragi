use crate::{CoreExpr, CoreFunction, CoreInstanceEvidence, CoreType};
use std::collections::BTreeMap;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreScalarKind {
    Int,
    Bool,
}

impl CoreScalarKind {
    fn from_type(ty: &CoreType) -> Option<Self> {
        // ExternalNamed preserves nominal identity, including a user's Int.
        let CoreType::Named { name, arguments } = ty else {
            return None;
        };
        if !arguments.is_empty() {
            return None;
        }
        match name.as_str() {
            "Int" => Some(Self::Int),
            "Bool" => Some(Self::Bool),
            _ => None,
        }
    }
}

/// A checked scalar slice. Backends keep the public/source calling convention
/// and use this fact only inside the fully applied function body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreScalarTailLoop {
    pub parameters: Vec<CoreScalarKind>,
    pub result: CoreScalarKind,
    pub self_tail_calls: usize,
}

/// Recognizes a deliberately small pure slice before erasing type/evidence
/// identity. Unknown shapes fall back to the backend's ordinary implementation.
pub fn scalar_self_tail_loop(function: &CoreFunction) -> Option<CoreScalarTailLoop> {
    if function.is_effect
        || !function.type_parameters.is_empty()
        || !function.constraints.is_empty()
        || function.parameters.is_empty()
    {
        return None;
    }
    let parameters = function
        .parameters
        .iter()
        .map(|p| {
            if p.kind != "named" {
                return None;
            }
            CoreScalarKind::from_type(&p.type_ref)
        })
        .collect::<Option<Vec<_>>>()?;
    let names = function
        .parameters
        .iter()
        .zip(&parameters)
        .map(|(p, kind)| (p.id.as_str(), *kind))
        .collect::<BTreeMap<_, _>>();
    if names.len() != parameters.len() || names.contains_key(function.symbol.as_str()) {
        return None;
    }
    let result = expression_kind(&function.body)?;
    let mut analysis = Analysis {
        function,
        names,
        parameters: &parameters,
        result,
        self_tail_calls: 0,
    };
    if analysis.expression(&function.body, true)? != result || analysis.self_tail_calls == 0 {
        return None;
    }
    let self_tail_calls = analysis.self_tail_calls;
    Some(CoreScalarTailLoop {
        parameters,
        result,
        self_tail_calls,
    })
}

fn expression_kind(expr: &CoreExpr) -> Option<CoreScalarKind> {
    match expr {
        CoreExpr::Integer { .. } => Some(CoreScalarKind::Int),
        CoreExpr::Boolean { .. } => Some(CoreScalarKind::Bool),
        CoreExpr::Variable { type_ref, .. }
        | CoreExpr::Binary { type_ref, .. }
        | CoreExpr::Unary { type_ref, .. }
        | CoreExpr::If { type_ref, .. }
        | CoreExpr::Call { type_ref, .. } => CoreScalarKind::from_type(type_ref),
        _ => None,
    }
}

struct Analysis<'a> {
    function: &'a CoreFunction,
    names: BTreeMap<&'a str, CoreScalarKind>,
    parameters: &'a [CoreScalarKind],
    result: CoreScalarKind,
    self_tail_calls: usize,
}

impl Analysis<'_> {
    fn expression(&mut self, expr: &CoreExpr, tail: bool) -> Option<CoreScalarKind> {
        use CoreScalarKind::{Bool, Int};
        let kind = match expr {
            CoreExpr::Integer { value, .. } => {
                let value = value.parse::<i64>().ok()?;
                if !(-9_007_199_254_740_991..=9_007_199_254_740_991).contains(&value) {
                    return None;
                }
                Int
            }
            CoreExpr::Boolean { .. } => Bool,
            CoreExpr::Variable { name, evidence, .. } if evidence.is_empty() => {
                *self.names.get(name.as_str())?
            }
            CoreExpr::Unary {
                operator, operand, ..
            } if operator == "-" => {
                if self.expression(operand, false)? != Int {
                    return None;
                }
                Int
            }
            CoreExpr::Binary {
                operator,
                left,
                right,
                evidence,
                ..
            } => {
                let (trait_name, identity, arity, result) = match operator.as_str() {
                    "+" => ("Add", "std/int::Add", 3, Int),
                    "-" => ("Sub", "std/int::Sub", 3, Int),
                    "*" => ("Mul", "std/int::Mul", 3, Int),
                    "==" => ("Eq", "std/int::Eq", 1, Bool),
                    "<" | "<=" => ("Ord", "std/int::Ord", 1, Bool),
                    _ => return None,
                };
                let [selected] = evidence.as_slice() else {
                    return None;
                };
                let CoreInstanceEvidence::Standard {
                    identity: selected_identity,
                    type_arguments,
                    evidence_arguments,
                } = &selected.evidence
                else {
                    return None;
                };
                if selected_identity != identity
                    || !type_arguments.is_empty()
                    || !evidence_arguments.is_empty()
                    || selected.constraint.name != trait_name
                    || selected
                        .constraint
                        .trait_identity
                        .as_deref()
                        .is_some_and(|name| name != format!("std/prelude::{trait_name}"))
                    || selected.constraint.arguments.len() != arity
                    || !selected
                        .constraint
                        .arguments
                        .iter()
                        .all(|ty| CoreScalarKind::from_type(ty) == Some(Int))
                    || self.expression(left, false)? != Int
                    || self.expression(right, false)? != Int
                {
                    return None;
                }
                result
            }
            CoreExpr::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                if self.expression(condition, false)? != Bool {
                    return None;
                }
                let left = self.expression(then_branch, tail)?;
                if self.expression(else_branch, tail)? != left {
                    return None;
                }
                left
            }
            CoreExpr::Call {
                callee,
                arguments,
                evidence,
                trait_dispatch,
                deferred_evidence_parameters,
                deferred_evidence_type_constructor_parameters,
                evidence_argument_index,
                ..
            } if tail
                && callee == &self.function.symbol
                && evidence.is_empty()
                && trait_dispatch.is_none()
                && deferred_evidence_parameters.is_empty()
                && deferred_evidence_type_constructor_parameters.is_empty()
                && evidence_argument_index.is_none()
                && arguments.len() == self.parameters.len() =>
            {
                for (arg, expected) in arguments.iter().zip(self.parameters) {
                    if self.expression(arg, false)? != *expected {
                        return None;
                    }
                }
                self.self_tail_calls += 1;
                self.result
            }
            _ => return None,
        };
        (expression_kind(expr)? == kind).then_some(kind)
    }
}
