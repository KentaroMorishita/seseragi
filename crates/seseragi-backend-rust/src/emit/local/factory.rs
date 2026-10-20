use super::*;
use seseragi_lowering::CoreFunction;

impl Backend<'_> {
    pub(in crate::emit) fn validate_method_factory(
        &self,
        callable: &LocalCallable,
        body: &CoreExpr,
        scope: &Scope,
    ) -> Result<()> {
        if scope.has_unproven_type_scope()
            || !matches!(callable.method_value, Some(CoreType::Function { .. }))
        {
            return Err("Rust backend: unproven method factory binder scope or value".into());
        }
        let CoreExpr::Call {
            arguments,
            evidence,
            trait_dispatch,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            callee,
            ..
        } = body
        else {
            return Err("Rust backend: unproven dictionary-only method factory".into());
        };
        if !callable.parameters.is_empty()
            || callable.self_name.is_some()
            || callable.constraints.len() != 1
            || (callable.variables.values().any(|arity| *arity != 0)
                && !callable
                    .constraints
                    .first()
                    .is_some_and(|constraint| self.declared_constructor_factory(constraint)))
            || !arguments.is_empty()
            || !deferred_evidence_parameters.is_empty()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || evidence_argument_index.is_some()
            || evidence.len() != 1
            || !matches!(
                evidence[0].evidence,
                CoreInstanceEvidence::Parameter { index: 0 }
            )
            || evidence[0].constraint.name != callable.constraints[0].name
            || evidence[0].constraint.arguments != callable.constraints[0].arguments
            || evidence[0]
                .constraint
                .trait_identity
                .as_ref()
                .is_some_and(|identity| {
                    Some(identity) != callable.constraints[0].trait_identity.as_ref()
                })
            || (trait_dispatch.is_none()
                && !scope
                    .callables
                    .get(callee)
                    .is_some_and(|c| c.method_value.is_some())
                && self.global_method_factory(callee)?.is_none())
        {
            return Err("Rust backend: unproven dictionary-only method factory".into());
        }
        if let Some(dispatch) = trait_dispatch {
            if dispatch.method_contract.is_some()
                || !matches!(
                    (dispatch.trait_identity.as_str(), dispatch.method.as_str()),
                    ("std/prelude::Eq", "eq")
                        | ("std/prelude::Ord", "compare")
                        | ("std/prelude::Add", "add")
                        | ("std/prelude::Sub", "sub")
                        | ("std/prelude::Mul", "mul")
                        | ("std/prelude::Div", "div")
                        | ("std/prelude::Rem", "rem")
                        | ("std/prelude::Pow", "pow")
                ) && !self.module.trait_methods.iter().any(|declaration| {
                    declaration.trait_identity == dispatch.trait_identity
                        && declaration.method == dispatch.method
                        && declaration.constraints.is_empty()
                        && !declaration.signature.parameters.is_empty()
                })
            {
                return Err("Rust backend: unproven method factory signature".into());
            }
        }
        Ok(())
    }

    /// A module-level constrained alias is a dictionary factory, not a Unit
    /// function. Preserve its declaration owner before comparing caller types.
    pub(in crate::emit) fn global_method_factory(
        &self,
        name: &str,
    ) -> Result<Option<LocalCallable>> {
        if !self.functions.contains(name) {
            return Ok(None);
        }
        let Some(function) = self.module.functions.iter().find(|f| f.symbol == name) else {
            return Ok(None);
        };
        if !function.parameters.is_empty() {
            return Ok(None);
        }
        let mut current = name;
        let mut visited = BTreeSet::new();
        loop {
            if !visited.insert(current) {
                return Err("Rust backend: cyclic method factory alias".into());
            }
            let declaration = self
                .module
                .functions
                .iter()
                .find(|f| f.symbol == current)
                .ok_or("Rust backend: unknown method factory declaration")?;
            if declaration.is_effect
                || !declaration.parameters.is_empty()
                || declaration.constraints.len() != 1
                || (declaration.type_parameters.iter().any(|p| p.arity != 0)
                    && !declaration
                        .constraints
                        .first()
                        .is_some_and(|constraint| self.declared_constructor_factory(constraint)))
            {
                return Err("Rust backend: unproven global method factory".into());
            }
            let CoreExpr::Call {
                callee,
                trait_dispatch,
                arguments,
                ..
            } = &declaration.body
            else {
                return Err("Rust backend: unproven global method factory value".into());
            };
            if !arguments.is_empty() {
                return Err("Rust backend: method factory introduces source arguments".into());
            }
            if trait_dispatch.is_some() {
                break;
            }
            current = callee;
        }
        let owner = &self.owners[name];
        let variables = function
            .type_parameters
            .iter()
            .enumerate()
            .map(|(i, p)| {
                (
                    p.name.clone(),
                    format!("\0global_method_factory::{name}::{i}"),
                    p.arity,
                )
            })
            .collect::<Vec<_>>();
        let substitutions = variables
            .iter()
            .map(|(old, name, _)| {
                (
                    old.clone(),
                    CoreType::Named {
                        name: name.clone(),
                        arguments: vec![],
                    },
                )
            })
            .collect();
        let canonical = |ty: &CoreType| {
            display::canonical_type(ty, owner, &substitutions, self.type_names, true).ok_or_else(
                || RustBackendError::from("Rust backend: unproven global method factory type"),
            )
        };
        let constraints = function
            .constraints
            .iter()
            .map(|c| {
                Ok(CoreInstanceConstraint {
                    arguments: c
                        .arguments
                        .iter()
                        .map(canonical)
                        .collect::<Result<Vec<_>>>()?,
                    ..c.clone()
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let mut callable = LocalCallable::new(
            variables.into_iter().map(|(_, name, arity)| (name, arity)),
            &constraints,
            &[],
        );
        let result = method_argument_type(&function.body)
            .ok_or("Rust backend: unproven global method result")?;
        callable.method_value = Some(canonical(&result)?);
        Ok(Some(callable))
    }

    /// An open constructor scheme needs the checked source declaration. The
    /// selected method is still independently checked against its exact kind,
    /// full value type and evidence frame when the factory body is emitted.
    fn declared_constructor_factory(&self, constraint: &CoreInstanceConstraint) -> bool {
        self.module.trait_methods.iter().any(|declaration| {
            Some(&declaration.trait_identity) == constraint.trait_identity.as_ref()
                && declaration.constraints.is_empty()
                && !declaration.signature.parameters.is_empty()
                && declaration
                    .trait_parameters
                    .iter()
                    .chain(&declaration.signature.type_parameters)
                    .any(|p| p.arity > 0)
        })
    }

    pub(in crate::emit) fn validate_global_method_factory(
        &self,
        function: &CoreFunction,
        scope: &Scope,
    ) -> Result<()> {
        self.global_method_factory(&function.symbol)?
            .ok_or("Rust backend: missing global method factory")?;
        let mut callable = LocalCallable::new(
            function
                .type_parameters
                .iter()
                .map(|p| (p.name.clone(), p.arity)),
            &function.constraints,
            &[],
        );
        callable.method_value = method_argument_type(&function.body);
        self.validate_method_factory(&callable, &function.body, scope)
    }
}
