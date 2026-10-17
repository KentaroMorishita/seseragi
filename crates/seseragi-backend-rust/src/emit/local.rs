use super::*;
use seseragi_lowering::CoreInstanceConstraint;

mod evidence;
mod types;
use types::argument_type;

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
        if callable.parameters.is_empty() {
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
        let CoreExpr::Call {
            callee,
            arguments,
            evidence,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            origin,
            ..
        } = expr
        else {
            unreachable!()
        };
        let callable = &scope.callables[callee];
        if !deferred_evidence_type_constructor_parameters.is_empty()
            || evidence_argument_index
                .is_some_and(|index| index != callable.parameters.len() || index > arguments.len())
        {
            return Err("Rust backend: unproven local callable evidence staging".into());
        }
        let mut bindings = self.validate_local_arguments(callable, arguments, evidence, scope)?;
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
            let mut completed = expr.clone();
            let CoreExpr::Call {
                arguments,
                deferred_evidence_parameters: deferred,
                ..
            } = &mut completed
            else {
                unreachable!()
            };
            let parameters = deferred
                .drain(..)
                .map(|type_ref| {
                    let id = format!("\0local_partial_{}", self.fresh());
                    arguments.push(CoreExpr::Variable {
                        name: id.clone(),
                        evidence: vec![],
                        type_ref: type_ref.clone(),
                        origin: origin.clone(),
                    });
                    CoreParameter {
                        id,
                        kind: "named".into(),
                        type_ref,
                    }
                })
                .collect::<Vec<_>>();
            // TS keeps the supplied expressions inside the deferred lambda.
            // Preserve that timing, including defects before later arguments.
            return self.function(&parameters, &completed, scope);
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
