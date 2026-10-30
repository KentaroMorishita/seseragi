use super::*;
fn arity(name: &str) -> Option<usize> {
    match name {
        "std/collection::reduceUntil" => Some(3),
        "std/collection::Next" | "std/collection::Done" => Some(1),
        _ => None,
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
fn control(value: CoreType) -> CoreType {
    CoreType::Named {
        name: "std/collection::ReduceStep".into(),
        arguments: vec![value],
    }
}
impl Backend<'_> {
    fn collection_control_type(
        &self,
        name: &str,
        arguments: &[CoreExpr],
        type_ref: &CoreType,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<usize> {
        if self.program_wasm_library {
            return Err(
                "program WASM: native collection control requires a target-specific ABI proof"
                    .into(),
            );
        }
        let arity = arity(name).expect("known collection control");
        if scope.has_unproven_type_scope() || arguments.len() > arity {
            return Err("Rust backend: unproven collection control source stage".into());
        }
        let supplied = arguments
            .iter()
            .map(|argument| {
                local::method_argument_type(argument)
                    .ok_or_else(|| "Rust backend: missing collection control argument type".into())
            })
            .collect::<Result<Vec<_>>>()?;
        let full = function(&supplied, type_ref.clone());
        if self.declared_method_type_kind(&full, scope) != Some(0) {
            return Err(
                "Rust backend: collection control requires value-kind source/result".into(),
            );
        }
        let full = self.local_canonical_type(&full, scope)?;
        let mut remaining = &full;
        let mut parameters = Vec::new();
        for _ in 0..arity {
            let CoreType::Function { parameter, result } = remaining else {
                return Err("Rust backend: missing collection control source arity".into());
            };
            parameters.push(parameter.as_ref().clone());
            remaining = result;
        }
        let accumulator = parameters[0].clone();
        let (expected, output) = if arity == 1 {
            if !evidence.is_empty() {
                return Err("Rust backend: unexpected ReduceStep constructor dictionary".into());
            }
            (vec![accumulator.clone()], control(accumulator))
        } else {
            let [selected] = evidence else {
                return Err("Rust backend: reduceUntil requires selected Iterable".into());
            };
            self.validate_iterable_evidence(selected, scope)?;
            let collection = self.local_canonical_type(&selected.constraint.arguments[0], scope)?;
            let element = self.local_canonical_type(&selected.constraint.arguments[1], scope)?;
            (
                vec![
                    accumulator.clone(),
                    function(
                        &[accumulator.clone(), element],
                        control(accumulator.clone()),
                    ),
                    collection,
                ],
                accumulator,
            )
        };
        if full != function(&expected, output) {
            return Err(
                "Rust backend: collection control canonical type, accumulator or source mismatch"
                    .into(),
            );
        }
        Ok(arity)
    }

    fn collection_control_callable(
        &mut self,
        name: &str,
        arity: usize,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<String> {
        let dictionary = "\0collection_control_dictionaries".to_owned();
        let names: Vec<_> = (0..arity)
            .map(|i| format!("\0collection_control_argument_{i}"))
            .collect();
        let args = names
            .iter()
            .map(|n| format!("{}.clone()", id(n)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = format!(
            "{}({}, native_map_dictionaries(&{}), vec![{args}])",
            "native_collection_control",
            quoted(name),
            id(&dictionary)
        );
        for i in (0..arity).rev() {
            let mut captures: Scope = names[..i].iter().cloned().collect();
            captures.insert(dictionary.clone());
            body = Self::closure(&names[i], &body, &captures);
        }
        let dictionaries = self.evidence(evidence, scope)?;
        Ok(format!(
            "{{ let {}=V::Tuple(Shared::new({dictionaries})); {body} }}",
            id(&dictionary)
        ))
    }

    pub(super) fn collection_control_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if arity(name).is_none() || scope.contains(name) {
            return Ok(None);
        }
        let arity = self.collection_control_type(name, &[], type_ref, evidence, scope)?;
        self.capabilities.insert("collection-control".into());
        Ok(Some(self.collection_control_callable(
            name, arity, evidence, scope,
        )?))
    }

    pub(super) fn collection_control_call(
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
            return Err("Rust backend: unproven collection source stages".into());
        }
        let arity = self.collection_control_type(callee, arguments, type_ref, evidence, scope)?;
        // A generic partial application can retain its remaining source types.
        // Its selected dictionaries are already checked in the caller's scope;
        // the metadata must describe exactly the ordinary curried suffix.
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
        self.capabilities.insert("collection-control".into());
        if arguments.len() == arity {
            self.direct_calls += 1;
            let dictionaries = self.evidence(evidence, scope)?;
            return Ok(Some(format!(
                "{}({}, {dictionaries}, vec![{}])",
                "native_collection_control",
                quoted(callee),
                self.expressions(arguments, scope)?
            )));
        }
        let value = self.fresh();
        let callable = self.collection_control_callable(callee, arity, evidence, scope)?;
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
