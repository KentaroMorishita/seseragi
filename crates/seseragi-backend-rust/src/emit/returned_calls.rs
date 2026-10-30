//! Prove constrained global source stages against their declaration. Native
//! dictionaries precede source arguments in the value ABI, so partial and
//! returned source functions retain their ordinary traceable captures.
use super::*;

impl Backend<'_> {
    pub(super) fn validate_returned_evidence_call(
        &self,
        expr: &CoreExpr,
        scope: &Scope,
    ) -> Result<bool> {
        let CoreExpr::Call {
            callee,
            arguments,
            evidence,
            trait_dispatch,
            evidence_argument_index,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            type_ref,
            ..
        } = expr
        else {
            return Ok(false);
        };
        let partial = !deferred_evidence_parameters.is_empty();
        if self.program_wasm_library {
            return Ok(false);
        }
        let function = self
            .module
            .functions
            .iter()
            .find(|function| function.symbol == *callee);
        if evidence_argument_index.is_none() && !partial {
            if trait_dispatch.is_none()
                && function.is_some_and(|function| {
                    !function.constraints.is_empty() && arguments.len() < function.parameters.len()
                })
            {
                return Err("Rust backend: missing global partial source stages".into());
            }
            return Ok(false);
        }
        if trait_dispatch.is_none()
            && function.is_none_or(|function| function.constraints.is_empty())
        {
            return Ok(false);
        }
        if !deferred_evidence_type_constructor_parameters.is_empty() {
            return Err("Rust backend: unproven returned callable source stages".into());
        }
        let index = if partial {
            let Some(function) = function.filter(|_| trait_dispatch.is_none()) else {
                return Err("Rust backend: unproven partial callable declaration".into());
            };
            if evidence_argument_index.is_some()
                || arguments.len() + deferred_evidence_parameters.len() != function.parameters.len()
                || function
                    .type_parameters
                    .iter()
                    .any(|parameter| parameter.arity != 0)
            {
                return Err("Rust backend: partial function source boundary mismatch".into());
            }
            function.parameters.len()
        } else {
            let index = evidence_argument_index.expect("checked returned source boundary");
            if index == 0 || index >= arguments.len() {
                return Err("Rust backend: unproven returned callable source stages".into());
            }
            index
        };
        let call_type = arguments
            .iter()
            .rev()
            .try_fold(type_ref.clone(), |result, argument| {
                let parameter = local::method_argument_type(argument)
                    .ok_or("Rust backend: unproven returned callable argument")?;
                Ok::<_, RustBackendError>(CoreType::Function {
                    parameter: Box::new(parameter),
                    result: Box::new(result),
                })
            })?;
        if let Some(dispatch) = trait_dispatch {
            let [selected] = evidence.as_slice() else {
                return Err("Rust backend: returned method needs one primary dictionary".into());
            };
            let declaration = self
                .module
                .trait_methods
                .iter()
                .find(|declaration| {
                    declaration.trait_identity == dispatch.trait_identity
                        && declaration.method == dispatch.method
                })
                .ok_or("Rust backend: missing returned method declaration")?;
            if declaration.signature.parameters.len() != index {
                return Err("Rust backend: returned method source boundary mismatch".into());
            }
            self.validate_declared_parameter_method(dispatch, selected, &call_type, scope)?;
            self.validate_bare_method(dispatch, evidence, &call_type, scope)?;
            return Ok(true);
        }
        let function = function.expect("checked constrained function");
        if scope.contains(callee) || function.is_effect || function.parameters.len() != index {
            return Err("Rust backend: returned function source boundary mismatch".into());
        }
        let owner = self
            .owners
            .get(callee)
            .ok_or("Rust backend: missing returned function owner")?;
        let mut substitutions = BTreeMap::new();
        let mut variables = BTreeMap::new();
        for (index, parameter) in function.type_parameters.iter().enumerate() {
            let name = format!("\0returned_function_type_{index}");
            if substitutions
                .insert(
                    parameter.name.clone(),
                    CoreType::Named {
                        name: name.clone(),
                        arguments: vec![],
                    },
                )
                .is_some()
            {
                return Err("Rust backend: overlapping returned function binders".into());
            }
            variables.insert(name, parameter.arity);
        }
        let canonical = |ty| {
            display::canonical_type(ty, owner, &substitutions, self.type_names, true).ok_or_else(
                || RustBackendError::from("Rust backend: unproven returned function declaration"),
            )
        };
        let value_type = canonical(
            function
                .value_type
                .as_ref()
                .ok_or("Rust backend: missing returned function value type")?,
        )?;
        let parameters = function
            .parameters
            .iter()
            .map(|parameter| {
                Ok(CoreParameter {
                    type_ref: canonical(&parameter.type_ref)?,
                    ..parameter.clone()
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let constraints = function
            .constraints
            .iter()
            .map(|constraint| {
                Ok(seseragi_lowering::CoreInstanceConstraint {
                    arguments: constraint
                        .arguments
                        .iter()
                        .map(canonical)
                        .collect::<Result<_>>()?,
                    ..constraint.clone()
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let callable = LocalCallable::new(variables.clone().into_iter(), &constraints, &parameters);
        let mut bindings = self.validate_local_arguments(&callable, arguments, evidence, scope)?;
        for (parameter, actual) in parameters
            .iter()
            .skip(arguments.len())
            .zip(deferred_evidence_parameters)
        {
            self.match_local_type(
                &parameter.type_ref,
                actual,
                &variables,
                &mut bindings,
                scope,
            )?;
        }
        self.match_local_type(&value_type, &call_type, &variables, &mut bindings, scope)?;
        Ok(true)
    }
}
