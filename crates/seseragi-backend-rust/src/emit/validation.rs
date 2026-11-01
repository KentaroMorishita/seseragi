use super::*;
fn arity(name: &str) -> Option<usize> {
    match name {
        "std/validation::Valid"
        | "std/validation::Invalid"
        | "std/validation::valid"
        | "std/validation::invalid"
        | "std/validation::invalidMany"
        | "std/validation::fromEither"
        | "std/validation::toEither" => Some(1),
        _ => None,
    }
}
fn named(name: &str, arguments: Vec<CoreType>) -> CoreType {
    CoreType::Named {
        name: name.into(),
        arguments,
    }
}
fn function(parameters: &[CoreType], result: CoreType) -> CoreType {
    parameters
        .iter()
        .rev()
        .fold(result, |result, parameter| CoreType::Function {
            parameter: Box::new(parameter.clone()),
            result: Box::new(result),
        })
}
impl Backend<'_> {
    fn validation_operation_type(
        &self,
        name: &str,
        arguments: &[CoreExpr],
        type_ref: &CoreType,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<usize> {
        if self.program_wasm_library
            || scope.has_unproven_type_scope()
            || !evidence.is_empty()
            || arguments.len() > 1
        {
            return Err("Rust backend: unproven native Validation application".into());
        }
        let supplied = arguments
            .iter()
            .map(|a| {
                local::method_argument_type(a)
                    .ok_or_else(|| "Rust backend: missing Validation argument type".into())
            })
            .collect::<Result<Vec<_>>>()?;
        let full = function(&supplied, type_ref.clone());
        if self.declared_method_type_kind(&full, scope) != Some(0) {
            return Err("Rust backend: Validation requires value-kind arguments/result".into());
        }
        let full = self.local_canonical_type(&full, scope)?;
        let e = named("\0validation_error", vec![]);
        let a = named("\0validation_value", vec![]);
        let validation = named("std/validation::Validation", vec![e.clone(), a.clone()]);
        let errors = named("std/non-empty-list::NonEmptyList", vec![e.clone()]);
        let either = |e| named("std/prelude::Either", vec![e, a.clone()]);
        let (argument, result) = match name {
            "std/validation::Valid" | "std/validation::valid" => (a.clone(), validation),
            "std/validation::Invalid" | "std/validation::invalidMany" => (errors, validation),
            "std/validation::invalid" => (e.clone(), validation),
            "std/validation::fromEither" => (either(e.clone()), validation),
            "std/validation::toEither" => (validation, either(errors)),
            _ => unreachable!(),
        };
        self.match_local_type(
            &function(&[argument], result),
            &full,
            &BTreeMap::from([
                ("\0validation_error".into(), 0),
                ("\0validation_value".into(), 0),
            ]),
            &mut BTreeMap::new(),
            scope,
        )?;
        Ok(1)
    }

    fn validation_operation_callable(
        &mut self,
        name: &str,
        arity: usize,
        _evidence: &[CoreCallEvidence],
        _scope: &Scope,
    ) -> Result<String> {
        let names: Vec<_> = (0..arity)
            .map(|i| format!("\0validation_argument_{i}"))
            .collect();
        let args = names
            .iter()
            .map(|n| format!("{}.clone()", id(n)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = format!(
            "native_validation_operation({}, vec![{args}])",
            quoted(name)
        );
        for i in (0..arity).rev() {
            body = Self::closure(&names[i], &body, &names[..i].iter().cloned().collect());
        }
        Ok(body)
    }

    pub(super) fn validation_operation_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if arity(name).is_none() || scope.contains(name) {
            return Ok(None);
        }
        let arity = self.validation_operation_type(name, &[], type_ref, evidence, scope)?;
        self.capabilities.insert("validation".into());
        Ok(Some(self.validation_operation_callable(
            name, arity, evidence, scope,
        )?))
    }

    pub(super) fn validation_operation_call(
        &mut self,
        expr: &CoreExpr,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let CoreExpr::Call {
            callee,
            arguments,
            type_ref,
            evidence,
            trait_dispatch,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            ..
        } = expr
        else {
            return Ok(None);
        };
        if arity(callee).is_none() || scope.contains(callee) {
            return Ok(None);
        }
        if trait_dispatch.is_some()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || evidence_argument_index.is_some()
        {
            return Err("Rust backend: unproven native source stages".into());
        }
        let arity = self.validation_operation_type(callee, arguments, type_ref, evidence, scope)?;
        // Retained argument types must describe the remaining curried parameters.
        if !deferred_evidence_parameters.is_empty() {
            if arguments.len() + deferred_evidence_parameters.len() != arity {
                return Err("Rust backend: collection deferred source arity mismatch".into());
            }
            let mut remaining = type_ref;
            for deferred in deferred_evidence_parameters {
                let CoreType::Function { parameter, result } = remaining else {
                    return Err("Rust backend: missing collection deferred source stage".into());
                };
                if self.local_canonical_type(parameter, scope)?
                    != self.local_canonical_type(deferred, scope)?
                {
                    return Err("Rust backend: collection deferred source type mismatch".into());
                }
                remaining = result;
            }
        }
        self.capabilities.insert("validation".into());
        let value = self.fresh();
        let callable = self.validation_operation_callable(callee, arity, evidence, scope)?;
        let mut body = format!("{{ let mut {value}={callable};");
        for argument in arguments {
            body.push_str(&format!(
                "{value}={value}.apply({});",
                self.expr(argument, scope)?
            ));
        }
        body.push_str(&format!("{value} }}"));
        Ok(Some(body))
    }
}
