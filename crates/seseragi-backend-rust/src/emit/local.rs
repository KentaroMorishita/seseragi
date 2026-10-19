use super::*;
use seseragi_lowering::CoreInstanceConstraint;

mod evidence;
mod factory;
mod types;
use types::argument_type;
pub(super) use types::argument_type as method_argument_type;

#[derive(Clone, Default)]
pub(super) struct Scope {
    names: BTreeSet<String>,
    blocked: BTreeSet<String>,
    pub(super) callables: BTreeMap<String, LocalCallable>,
    pub(super) dictionaries: Vec<CoreInstanceConstraint>,
    type_parameters: BTreeMap<String, (u32, CoreType)>,
    unproven_type_scope: bool,
}

impl Scope {
    pub(super) fn new() -> Self {
        Self::default()
    }
    pub(super) fn has_unproven_type_scope(&self) -> bool {
        self.unproven_type_scope
    }
    pub(super) fn type_parameters(
        &mut self,
        parameters: impl Iterator<Item = (String, u32)>,
        declaration: &str,
    ) {
        for (index, (name, arity)) in parameters.enumerate() {
            if self.type_parameters.contains_key(&name) {
                self.unproven_type_scope = true;
            }
            self.type_parameters.insert(
                name,
                (
                    arity,
                    CoreType::Named {
                        name: format!("\0type_parameter::{declaration}::{index}"),
                        arguments: vec![],
                    },
                ),
            );
        }
    }
    pub(super) fn contains(&self, name: &str) -> bool {
        self.names.contains(name)
    }
    pub(super) fn insert(&mut self, name: String) {
        self.callables.remove(&name);
        self.blocked.remove(&name);
        self.names.insert(name);
    }
    pub(super) fn block(&mut self, name: String) {
        self.names.remove(&name);
        self.callables.remove(&name);
        self.blocked.insert(name);
    }
    pub(super) fn is_blocked(&self, name: &str) -> bool {
        self.blocked.contains(name)
    }
    pub(super) fn insert_callable(&mut self, name: String, callable: LocalCallable) {
        self.insert(name.clone());
        if !callable.constraints.is_empty() {
            self.callables.insert(name, callable);
        }
    }
    pub(super) fn iter(&self) -> impl Iterator<Item = &String> {
        self.names.iter()
    }
    pub(super) fn retain(&mut self, keep: impl FnMut(&String) -> bool) {
        self.names.retain(keep);
        self.callables.retain(|name, _| self.names.contains(name));
    }
}
impl FromIterator<String> for Scope {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        Self {
            names: iter.into_iter().collect(),
            ..Self::default()
        }
    }
}

#[derive(Clone)]
pub(super) struct LocalCallable {
    pub(super) self_name: Option<String>,
    variables: BTreeMap<String, u32>,
    constraints: Vec<CoreInstanceConstraint>,
    parameters: Vec<CoreParameter>,
    /// A let factory introduces only dictionaries and returns this checked
    /// method type. It has no implicit Unit or eta-expanded source stages.
    pub(super) method_value: Option<CoreType>,
}
impl LocalCallable {
    pub(super) fn new(
        variables: impl Iterator<Item = (String, u32)>,
        constraints: &[CoreInstanceConstraint],
        parameters: &[CoreParameter],
    ) -> Self {
        Self {
            self_name: None,
            variables: variables.collect(),
            constraints: constraints.to_vec(),
            parameters: parameters.to_vec(),
            method_value: None,
        }
    }
}

impl Backend<'_> {
    pub(super) fn local_function(
        &mut self,
        callable: &LocalCallable,
        body: &CoreExpr,
        scope: &Scope,
    ) -> Result<String> {
        if callable.parameters.is_empty() && callable.method_value.is_none() {
            return Err("Rust backend: local callable needs source parameters".into());
        }
        let mut typed_scope = scope.clone();
        if let Some(name) = &callable.self_name {
            typed_scope.block(name.clone());
        }
        if !callable.variables.is_empty() {
            let declaration = self.fresh();
            typed_scope.type_parameters(
                callable
                    .variables
                    .iter()
                    .map(|(name, arity)| (name.clone(), *arity)),
                &declaration,
            );
        }
        let scope = &typed_scope;
        if callable.constraints.is_empty() {
            return self.function(&callable.parameters, body, scope);
        }
        if scope.unproven_type_scope {
            return Err("Rust backend: unproven overlapping local type parameter scope".into());
        }
        if callable
            .constraints
            .iter()
            .any(|constraint| !self.valid_constraint_label(constraint))
        {
            return Err("Rust backend: unproven local callable trait identity".into());
        }
        // A generalized lambda has its own evidence frame, including any
        // monomorphic obligations inherited from the enclosing function.
        // Callers explicitly pass those obligations; old indexes must not leak
        // into the new frame. Term captures retain their original graph edges.
        let mut factory = scope.clone();
        factory.retain(|name| !name.starts_with("\0dictionary_"));
        if let Some(mut free) = core_free_variables(body) {
            for parameter in &callable.parameters {
                free.names.remove(&parameter.id);
            }
            factory.retain(|name| free.names.contains(name));
        }
        let mut inner = factory.clone();
        inner.dictionaries = callable.constraints.clone();
        for index in 0..callable.constraints.len() {
            inner.insert(format!("\0dictionary_{index}"));
        }
        if callable.method_value.is_some() {
            self.validate_method_factory(callable, body, &inner)?;
        }
        let mut value = self.function(&callable.parameters, body, &inner)?;
        // These are implementation-only stages, never source parameters. Both
        // dictionaries and source captures remain ordinary traceable V edges.
        for index in (0..callable.constraints.len()).rev() {
            let mut captures = factory.clone();
            for previous in 0..index {
                captures.insert(format!("\0dictionary_{previous}"));
            }
            value = Self::closure(&format!("\0dictionary_{index}"), &value, &captures);
        }
        Ok(value)
    }

    pub(super) fn local_call(&mut self, expr: &CoreExpr, scope: &Scope) -> Result<String> {
        let CoreExpr::Call { callee, .. } = expr else {
            unreachable!()
        };
        let callable = &scope.callables[callee];
        self.call_local_value(expr, callable, scope)
    }

    pub(in crate::emit) fn call_local_value(
        &mut self,
        expr: &CoreExpr,
        callable: &LocalCallable,
        scope: &Scope,
    ) -> Result<String> {
        let CoreExpr::Call {
            callee,
            arguments,
            evidence,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            type_ref,
            ..
        } = expr
        else {
            unreachable!()
        };

        if callable.method_value.is_some()
            && (!deferred_evidence_parameters.is_empty()
                || *evidence_argument_index != (!arguments.is_empty()).then_some(0))
        {
            return Err("Rust backend: method factory dictionary staging mismatch".into());
        }
        if !deferred_evidence_type_constructor_parameters.is_empty()
            || evidence_argument_index
                .is_some_and(|index| index != callable.parameters.len() || index > arguments.len())
        {
            return Err("Rust backend: unproven local callable evidence staging".into());
        }
        if arguments.len() < callable.parameters.len()
            && deferred_evidence_parameters.len() != callable.parameters.len() - arguments.len()
        {
            return Err("Rust backend: missing local partial source stages".into());
        }
        let mut bindings = self.validate_local_arguments(callable, arguments, evidence, scope)?;
        if let Some(method_type) = &callable.method_value {
            let mut remaining = method_type.clone();
            for argument in arguments {
                // A value-kinded result slot can be bound to a function by
                // the selected dictionary. Follow only those proved formal
                // substitutions before applying further returned stages.
                let mut visited = BTreeSet::new();
                while let CoreType::Named { name, arguments } = &remaining {
                    if !arguments.is_empty() || !callable.variables.contains_key(name) {
                        break;
                    }
                    let Some(bound) = bindings.get(name) else {
                        break;
                    };
                    if !visited.insert(name.clone()) {
                        return Err("Rust backend: cyclic method result substitution".into());
                    }
                    remaining = bound.clone();
                }
                let CoreType::Function { parameter, result } = remaining else {
                    return Err("Rust backend: method factory source arity mismatch".into());
                };
                let actual = argument_type(argument)
                    .ok_or("Rust backend: unproven method factory argument")?;
                self.match_local_type(
                    &parameter,
                    &actual,
                    &callable.variables,
                    &mut bindings,
                    scope,
                )?;
                remaining = *result;
            }
            self.match_local_type(
                &remaining,
                type_ref,
                &callable.variables,
                &mut bindings,
                scope,
            )?;
        }
        for (parameter, actual) in callable
            .parameters
            .iter()
            .skip(arguments.len())
            .zip(deferred_evidence_parameters)
        {
            self.match_local_type(
                &parameter.type_ref,
                actual,
                &callable.variables,
                &mut bindings,
                scope,
            )?;
        }
        if !deferred_evidence_parameters.is_empty() {
            if arguments.len() + deferred_evidence_parameters.len() != callable.parameters.len()
                || evidence_argument_index.is_some()
            {
                return Err("Rust backend: local partial source arity mismatch".into());
            }
            // Local factories take their checked dictionaries before the
            // source stages. Applying the supplied prefix directly retains
            // already evaluated values as ordinary traceable closure edges.
        }
        let function = self.reference(callee, evidence, scope)?;
        let value = self.fresh();
        let mut output = format!("{{ let mut {value} = {function};");
        for argument in arguments {
            output.push_str(&format!(
                "{value} = {value}.apply({});",
                self.expr(argument, scope)?
            ));
        }
        output.push_str(&format!("{value} }}"));
        Ok(output)
    }

    pub(super) fn validate_local_arguments(
        &self,
        callable: &LocalCallable,
        arguments: &[CoreExpr],
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<BTreeMap<String, CoreType>> {
        let mut bindings = self.validate_local_evidence(callable, evidence, scope)?;
        for (parameter, argument) in callable.parameters.iter().zip(arguments) {
            let actual = argument_type(argument)
                .ok_or("Rust backend: unproven local callable argument type")?;
            self.match_local_type(
                &parameter.type_ref,
                &actual,
                &callable.variables,
                &mut bindings,
                scope,
            )?;
        }
        Ok(bindings)
    }
}
