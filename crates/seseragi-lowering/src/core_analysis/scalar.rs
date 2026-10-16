use crate::{CoreExpr, CoreFunction, CoreInstanceEvidence, CoreModule, CoreType};
use std::collections::{BTreeMap, BTreeSet};

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
    /// Original CoreIr spelling -> proven canonical binding. This authorizes
    /// scalar reads at their original position, never constant propagation or
    /// moving reads ahead of a branch/module initializer.
    pub global_reads: BTreeMap<String, CoreScalarGlobalRead>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreScalarGlobalRead {
    pub symbol: String,
    pub kind: CoreScalarKind,
}

/// Facts derived from checked declarations, before target type/evidence
/// erasure. Deliberately limited to closed monomorphic Int/Bool literals.
/// No values are copied into the certificate, so a target must still read the
/// original global. Unknown/duplicate declarations cannot provide a proof.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CoreScalarGlobals {
    bindings: BTreeMap<String, CoreScalarKind>,
    seen: BTreeSet<String>,
}

impl CoreScalarGlobals {
    pub fn from_modules<'a>(modules: impl IntoIterator<Item = &'a CoreModule>) -> Self {
        modules.into_iter().fold(Self::default(), Self::with_module)
    }

    /// Extend a checked project prefix without re-walking earlier modules.
    pub fn with_module(mut self, module: &CoreModule) -> Self {
        for binding in &module.bindings {
            if !self.seen.insert(binding.symbol.clone()) {
                self.bindings.remove(&binding.symbol);
                continue;
            }
            if binding.symbol.rsplit_once("::").map(|(owner, _)| owner)
                != Some(module.module.as_str())
                || !binding.type_parameters.is_empty()
            {
                continue;
            }
            let kind = match &binding.value {
                CoreExpr::Integer { value, .. } if safe_integer(value) => CoreScalarKind::Int,
                CoreExpr::Boolean { .. } => CoreScalarKind::Bool,
                _ => continue,
            };
            self.bindings.insert(binding.symbol.clone(), kind);
        }
        self
    }

    fn resolve(&self, owner: &str, name: &str) -> Option<CoreScalarGlobalRead> {
        let symbol = if name.contains("::") {
            name.to_owned()
        } else {
            format!("{owner}::{name}")
        };
        self.bindings.get(&symbol).map(|kind| CoreScalarGlobalRead {
            symbol,
            kind: *kind,
        })
    }
}

fn safe_integer(value: &str) -> bool {
    value
        .parse::<i64>()
        .is_ok_and(|value| (-9_007_199_254_740_991..=9_007_199_254_740_991).contains(&value))
}

/// Recognizes a deliberately small pure slice before erasing type/evidence
/// identity. Unknown shapes fall back to the backend's ordinary implementation.
pub fn scalar_self_tail_loop(function: &CoreFunction) -> Option<CoreScalarTailLoop> {
    scalar_self_tail_loop_with_globals(
        function,
        function
            .symbol
            .rsplit_once("::")
            .map_or("", |(owner, _)| owner),
        &CoreScalarGlobals::default(),
    )
}

pub fn scalar_self_tail_loop_with_globals(
    function: &CoreFunction,
    owner: &str,
    globals: &CoreScalarGlobals,
) -> Option<CoreScalarTailLoop> {
    if function.is_effect
        || !function.type_parameters.is_empty()
        || !function.constraints.is_empty()
        || function.parameters.is_empty()
        || function.symbol.rsplit_once("::").map(|(owner, _)| owner) != Some(owner)
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
        owner,
        globals,
        global_reads: BTreeMap::new(),
    };
    if analysis.expression(&function.body, true)? != result || analysis.self_tail_calls == 0 {
        return None;
    }
    let self_tail_calls = analysis.self_tail_calls;
    let global_reads = analysis.global_reads;
    Some(CoreScalarTailLoop {
        parameters,
        result,
        self_tail_calls,
        global_reads,
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
    owner: &'a str,
    globals: &'a CoreScalarGlobals,
    global_reads: BTreeMap<String, CoreScalarGlobalRead>,
}

impl Analysis<'_> {
    fn expression(&mut self, expr: &CoreExpr, tail: bool) -> Option<CoreScalarKind> {
        use CoreScalarKind::{Bool, Int};
        let kind = match expr {
            CoreExpr::Integer { value, .. } => {
                if !safe_integer(value) {
                    return None;
                }
                Int
            }
            CoreExpr::Boolean { .. } => Bool,
            CoreExpr::Variable { name, evidence, .. } if evidence.is_empty() => {
                if let Some(kind) = self.names.get(name.as_str()) {
                    *kind
                } else {
                    let read = self.globals.resolve(self.owner, name)?;
                    let kind = read.kind;
                    self.global_reads.insert(name.clone(), read);
                    kind
                }
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
