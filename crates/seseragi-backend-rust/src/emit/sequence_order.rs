use super::*;

fn arity(name: &str) -> Option<usize> {
    let (owner, operation) = name.split_once("::")?;
    if !matches!(owner, "std/array" | "std/list") {
        return None;
    }
    match operation {
        "sort" => Some(1),
        "sortBy" | "groupBy" => Some(2),
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
impl Backend<'_> {
    fn sequence_order_type(
        &self,
        name: &str,
        arguments: &[CoreExpr],
        type_ref: &CoreType,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<usize> {
        if self.program_wasm_library {
            return Err(
                "program WASM: native sequence ordering requires a target-specific ABI proof"
                    .into(),
            );
        }
        let arity = arity(name).expect("known sequence ordering operation");
        if scope.has_unproven_type_scope() || arguments.len() > arity {
            return Err("Rust backend: unproven sequence ordering source application".into());
        }
        let supplied = arguments
            .iter()
            .map(|argument| {
                local::method_argument_type(argument)
                    .ok_or_else(|| "Rust backend: missing sequence ordering argument type".into())
            })
            .collect::<Result<Vec<_>>>()?;
        let full = function(&supplied, type_ref.clone());
        if self.declared_method_type_kind(&full, scope) != Some(0) {
            return Err("Rust backend: unproven sequence ordering argument/result kind".into());
        }
        let full = self.local_canonical_type(&full, scope)?;
        let mut result = &full;
        let mut parameters = Vec::new();
        for _ in 0..arity {
            let CoreType::Function {
                parameter,
                result: next,
            } = result
            else {
                return Err("Rust backend: missing sequence ordering source stage".into());
            };
            parameters.push(parameter.as_ref().clone());
            result = next;
        }
        let owner = if name.starts_with("std/list::") {
            "std/prelude::List"
        } else {
            "std/prelude::Array"
        };
        let collection = parameters[arity - 1].clone();
        let CoreType::Named {
            name: actual,
            arguments: slots,
        } = &collection
        else {
            return Err("Rust backend: unproven sequence ordering collection".into());
        };
        if actual != owner || slots.len() != 1 {
            return Err("Rust backend: sequence ordering canonical owner/arity mismatch".into());
        }
        let element = slots[0].clone();
        let key = if arity == 1 {
            element.clone()
        } else {
            let CoreType::Function { parameter, result } = &parameters[0] else {
                return Err("Rust backend: sequence ordering requires a key function".into());
            };
            if parameter.as_ref() != &element {
                return Err("Rust backend: sequence ordering key parameter mismatch".into());
            }
            result.as_ref().clone()
        };
        let output = if name.ends_with("::groupBy") {
            CoreType::Named {
                name: "std/map::Map".into(),
                arguments: vec![key.clone(), collection.clone()],
            }
        } else {
            collection
        };
        if result != &output {
            return Err("Rust backend: sequence ordering result mismatch".into());
        }
        if name.ends_with("::groupBy") {
            let [eq, hash] = evidence else {
                return Err("Rust backend: sequence groupBy requires selected Eq/Hash".into());
            };
            self.validate_eq_evidence(eq, scope)?;
            self.validate_hash_evidence(hash, scope)?;
        } else {
            let [ord] = evidence else {
                return Err("Rust backend: sequence sort requires selected Ord".into());
            };
            if ord.constraint.trait_identity.as_deref() != Some("std/prelude::Ord")
                || !self.valid_constraint_label(&ord.constraint)
            {
                return Err("Rust backend: sequence sort requires canonical Ord".into());
            }
            self.validate_parameter_evidence(ord, scope)?;
            let mut normalized = ord.clone();
            normalized.constraint.name = "Ord".into();
            self.validate_ord_evidence(&normalized, scope)?;
        }
        for selected in evidence {
            if self.local_canonical_type(&selected.constraint.arguments[0], scope)? != key {
                return Err("Rust backend: sequence ordering dictionary key mismatch".into());
            }
        }
        Ok(arity)
    }

    fn sequence_order_callable(
        &mut self,
        name: &str,
        arity: usize,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<String> {
        let dictionary = "\0sequence_order_dictionaries".to_owned();
        let names: Vec<_> = (0..arity)
            .map(|i| format!("\0sequence_order_argument_{i}"))
            .collect();
        let args = names
            .iter()
            .map(|n| format!("{}.clone()", id(n)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = format!(
            "{}({}, native_map_dictionaries(&{}), vec![{args}])",
            "native_sequence_order_operation",
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

    pub(super) fn sequence_order_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if arity(name).is_none() || scope.contains(name) {
            return Ok(None);
        }
        let arity = self.sequence_order_type(name, &[], type_ref, evidence, scope)?;
        self.capabilities.insert("sequence-order".into());
        Ok(Some(
            self.sequence_order_callable(name, arity, evidence, scope)?,
        ))
    }

    pub(super) fn sequence_order_call(
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
        let arity = self.sequence_order_type(callee, arguments, type_ref, evidence, scope)?;
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
        self.capabilities.insert("sequence-order".into());
        if arguments.len() == arity {
            self.direct_calls += 1;
            let dictionaries = self.evidence(evidence, scope)?;
            return Ok(Some(format!(
                "{}({}, {dictionaries}, vec![{}])",
                "native_sequence_order_operation",
                quoted(callee),
                self.expressions(arguments, scope)?
            )));
        }
        let value = self.fresh();
        let callable = self.sequence_order_callable(callee, arity, evidence, scope)?;
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
