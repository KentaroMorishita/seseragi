use super::*;

fn arity(name: &str) -> Option<usize> {
    Some(match name {
        "std/map::empty" | "std/map::keys" | "std/map::values" | "std/map::entries"
        | "std/map::size" | "std/map::isEmpty" | "std/set::empty" | "std/set::singleton"
        | "std/set::toArray" | "std/set::toList" | "std/set::size" | "std/set::isEmpty" => 1,
        "std/map::singleton"
        | "std/map::get"
        | "std/map::containsKey"
        | "std/map::remove"
        | "std/set::contains"
        | "std/set::insert"
        | "std/set::remove"
        | "std/set::filter"
        | "std/set::union"
        | "std/set::intersection"
        | "std/set::difference"
        | "std/set::isSubsetOf" => 2,
        "std/map::insert" => 3,
        _ => return None,
    })
}

fn capability(name: &str) -> &'static str {
    if name.starts_with("std/set::") {
        "set"
    } else {
        "map"
    }
}
fn operation(name: &str) -> &'static str {
    if capability(name) == "set" {
        "native_set_operation"
    } else {
        "native_map_operation"
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
    fn collection_operation_type(
        &self,
        name: &str,
        arguments: &[CoreExpr],
        type_ref: &CoreType,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<usize> {
        if self.program_wasm_library {
            return Err(
                "program WASM: native collection requires a target-specific ABI proof".into(),
            );
        }
        let arity = arity(name).expect("known collection operation");
        if scope.has_unproven_type_scope() || arguments.len() > arity {
            return Err("Rust backend: unproven collection source application".into());
        }
        let supplied = arguments
            .iter()
            .map(|argument| {
                local::method_argument_type(argument)
                    .ok_or_else(|| "Rust backend: missing collection source argument type".into())
            })
            .collect::<Result<Vec<_>>>()?;
        let full = function(&supplied, type_ref.clone());
        if self.declared_method_type_kind(&full, scope) != Some(0) {
            return Err("Rust backend: unproven collection argument/result kind".into());
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
                return Err("Rust backend: unproven collection source arity".into());
            };
            parameters.push(parameter.as_ref());
            result = next;
        }
        let map = if matches!(
            name,
            "std/map::empty" | "std/map::singleton" | "std/set::empty" | "std/set::singleton"
        ) {
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
        let (expected_owner, type_arity) = if capability(name) == "set" {
            ("std/set::Set", 1)
        } else {
            ("std/map::Map", 2)
        };
        if owner != expected_owner || slots.len() != type_arity {
            return Err("Rust backend: collection owner or type arity mismatch".into());
        }
        let key = &slots[0];
        let unit = CoreType::Named {
            name: "std/prelude::Unit".into(),
            arguments: vec![],
        };
        let value = slots.get(1).unwrap_or(&unit);
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
            "std/map::containsKey" => (vec![key.clone(), map.clone()], named("Bool", vec![]), true),
            "std/map::entries" => (
                vec![map.clone()],
                named(
                    "Array",
                    vec![CoreType::Tuple {
                        elements: vec![key.clone(), value.clone()],
                    }],
                ),
                false,
            ),
            "std/set::empty" => (vec![unit.clone()], map.clone(), false),
            "std/set::singleton" => (vec![key.clone()], map.clone(), true),
            "std/set::contains" => (vec![key.clone(), map.clone()], named("Bool", vec![]), true),
            "std/set::insert" | "std/set::remove" => {
                (vec![key.clone(), map.clone()], map.clone(), true)
            }
            "std/set::toArray" => (vec![map.clone()], named("Array", vec![key.clone()]), false),
            "std/set::toList" => (vec![map.clone()], named("List", vec![key.clone()]), false),
            "std/set::filter" => (
                vec![function(&[key.clone()], named("Bool", vec![])), map.clone()],
                map.clone(),
                false,
            ),
            "std/set::union" | "std/set::intersection" | "std/set::difference" => {
                (vec![map.clone(), map.clone()], map.clone(), true)
            }
            "std/set::isSubsetOf" => (vec![map.clone(), map.clone()], named("Bool", vec![]), true),
            "std/set::size" => (vec![map.clone()], named("Int", vec![]), false),
            "std/set::isEmpty" => (vec![map.clone()], named("Bool", vec![]), false),
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
            return Err("Rust backend: collection element/source contract mismatch".into());
        }
        if needs_key {
            let [eq, hash] = evidence else {
                return Err(
                    "Rust backend: collection requires selected Eq and Hash dictionaries".into(),
                );
            };
            self.validate_eq_evidence(eq, scope)?;
            self.validate_hash_evidence(hash, scope)?;
            for selected in [eq, hash] {
                if self.local_canonical_type(&selected.constraint.arguments[0], scope)? != *key {
                    return Err("Rust backend: collection key dictionary argument mismatch".into());
                }
            }
        } else if !evidence.is_empty() {
            return Err("Rust backend: unexpected collection dictionary".into());
        }
        Ok(arity)
    }

    fn collection_callable(
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
            "{}({}, native_map_dictionaries(&{}), vec![{args}])",
            operation(name),
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

    pub(super) fn collection_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if arity(name).is_none() || scope.contains(name) {
            return Ok(None);
        }
        let arity = self.collection_operation_type(name, &[], type_ref, evidence, scope)?;
        self.capabilities.insert(capability(name).into());
        Ok(Some(
            self.collection_callable(name, arity, evidence, scope)?,
        ))
    }

    pub(super) fn collection_call(
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
        let arity = self.collection_operation_type(callee, arguments, type_ref, evidence, scope)?;
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
        self.capabilities.insert(capability(callee).into());
        if arguments.len() == arity {
            self.direct_calls += 1;
            let dictionaries = self.evidence(evidence, scope)?;
            return Ok(Some(format!(
                "{}({}, {dictionaries}, vec![{}])",
                operation(callee),
                quoted(callee),
                self.expressions(arguments, scope)?
            )));
        }
        let value = self.fresh();
        let callable = self.collection_callable(callee, arity, evidence, scope)?;
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
