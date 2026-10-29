use super::*;

fn arity(name: &str) -> Option<usize> {
    Some(match name {
        "std/map::empty" | "std/map::keys" | "std/map::values" | "std/map::size"
        | "std/map::isEmpty" => 1,
        "std/map::singleton" | "std/map::get" | "std/map::remove" => 2,
        "std/map::insert" => 3,
        _ => return None,
    })
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
    fn map_operation_type(
        &self,
        name: &str,
        arguments: &[CoreExpr],
        type_ref: &CoreType,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<usize> {
        if self.program_wasm_library {
            return Err("program WASM: native Map requires a target-specific ABI proof".into());
        }
        let arity = arity(name).expect("known Map operation");
        if scope.has_unproven_type_scope() || arguments.len() > arity {
            return Err("Rust backend: unproven Map source application".into());
        }
        let supplied = arguments
            .iter()
            .map(|argument| {
                local::method_argument_type(argument)
                    .ok_or_else(|| "Rust backend: missing Map source argument type".into())
            })
            .collect::<Result<Vec<_>>>()?;
        let full = function(&supplied, type_ref.clone());
        if self.declared_method_type_kind(&full, scope) != Some(0) {
            return Err("Rust backend: unproven Map argument/result kind".into());
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
                return Err("Rust backend: unproven Map source arity".into());
            };
            parameters.push(parameter.as_ref());
            result = next;
        }
        let map = if matches!(name, "std/map::empty" | "std/map::singleton") {
            result
        } else {
            parameters[arity - 1]
        };
        let CoreType::Named {
            name: owner,
            arguments: slots,
        } = map
        else {
            return Err("Rust backend: missing canonical Map type".into());
        };
        if owner != "std/map::Map" || slots.len() != 2 {
            return Err("Rust backend: Map owner or type arity mismatch".into());
        }
        let (key, value) = (&slots[0], &slots[1]);
        let named = |name: &str, arguments| CoreType::Named {
            name: format!("std/prelude::{name}"),
            arguments,
        };
        let (expected, output, needs_key) = match name {
            "std/map::empty" => (vec![named("Unit", vec![])], map.clone(), false),
            "std/map::singleton" => (vec![key.clone(), value.clone()], map.clone(), true),
            "std/map::get" => (
                vec![key.clone(), map.clone()],
                named("Maybe", vec![value.clone()]),
                true,
            ),
            "std/map::insert" => (
                vec![key.clone(), value.clone(), map.clone()],
                map.clone(),
                true,
            ),
            "std/map::remove" => (vec![key.clone(), map.clone()], map.clone(), true),
            "std/map::keys" => (vec![map.clone()], named("Array", vec![key.clone()]), false),
            "std/map::values" => (
                vec![map.clone()],
                named("Array", vec![value.clone()]),
                false,
            ),
            "std/map::size" => (vec![map.clone()], named("Int", vec![]), false),
            "std/map::isEmpty" => (vec![map.clone()], named("Bool", vec![]), false),
            _ => unreachable!(),
        };
        if full != function(&expected, output) {
            return Err("Rust backend: Map key/value/source contract mismatch".into());
        }
        if needs_key {
            let [eq, hash] = evidence else {
                return Err("Rust backend: Map requires selected Eq and Hash dictionaries".into());
            };
            self.validate_eq_evidence(eq, scope)?;
            self.validate_hash_evidence(hash, scope)?;
            for selected in [eq, hash] {
                if self.local_canonical_type(&selected.constraint.arguments[0], scope)? != *key {
                    return Err("Rust backend: Map key dictionary argument mismatch".into());
                }
            }
        } else if !evidence.is_empty() {
            return Err("Rust backend: unexpected Map dictionary".into());
        }
        Ok(arity)
    }

    fn map_callable(
        &mut self,
        name: &str,
        arity: usize,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<String> {
        let dictionary = "\0map_dictionaries".to_owned();
        let names: Vec<_> = (0..arity).map(|i| format!("\0map_argument_{i}")).collect();
        let args = names
            .iter()
            .map(|n| format!("{}.clone()", id(n)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = format!(
            "native_map_operation({}, native_map_dictionaries(&{}), vec![{args}])",
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

    pub(super) fn map_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if arity(name).is_none() || scope.contains(name) {
            return Ok(None);
        }
        let arity = self.map_operation_type(name, &[], type_ref, evidence, scope)?;
        self.capabilities.insert("map".into());
        Ok(Some(self.map_callable(name, arity, evidence, scope)?))
    }

    pub(super) fn map_call(&mut self, expr: &CoreExpr, scope: &Scope) -> Result<Option<String>> {
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
            return Err("Rust backend: unproven Map source stages".into());
        }
        let arity = self.map_operation_type(callee, arguments, type_ref, evidence, scope)?;
        // A generic partial application can retain its remaining source types.
        // Its selected dictionaries are already checked in the caller's scope;
        // the metadata must describe exactly the ordinary curried suffix.
        if !deferred_evidence_parameters.is_empty() {
            if arguments.len() + deferred_evidence_parameters.len() != arity {
                return Err("Rust backend: Map deferred source arity mismatch".into());
            }
            let mut remaining = type_ref;
            for deferred in deferred_evidence_parameters {
                let CoreType::Function { parameter, result } = remaining else {
                    return Err("Rust backend: missing Map deferred source stage".into());
                };
                if self.local_canonical_type(parameter, scope)?
                    != self.local_canonical_type(deferred, scope)?
                {
                    return Err("Rust backend: Map deferred source type mismatch".into());
                }
                remaining = result;
            }
        }
        self.capabilities.insert("map".into());
        if arguments.len() == arity {
            self.direct_calls += 1;
            let dictionaries = self.evidence(evidence, scope)?;
            return Ok(Some(format!(
                "native_map_operation({}, {dictionaries}, vec![{}])",
                quoted(callee),
                self.expressions(arguments, scope)?
            )));
        }
        let value = self.fresh();
        let callable = self.map_callable(callee, arity, evidence, scope)?;
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
