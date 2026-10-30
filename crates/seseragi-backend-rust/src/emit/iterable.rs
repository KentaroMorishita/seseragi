use super::*;

fn arity(name: &str) -> Option<usize> {
    matches!(
        name,
        "std/array::fromIterable"
            | "std/list::fromIterable"
            | "std/set::fromIterable"
            | "std/map::fromEntries"
    )
    .then_some(1)
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
fn named(name: &str, arguments: Vec<CoreType>) -> CoreType {
    CoreType::Named {
        name: name.into(),
        arguments,
    }
}
impl Backend<'_> {
    pub(super) fn validate_iterable_evidence(
        &self,
        selected: &CoreCallEvidence,
        scope: &Scope,
    ) -> Result<()> {
        if selected.constraint.trait_identity.as_deref() != Some("std/prelude::Iterable")
            || !self.valid_constraint_label(&selected.constraint)
            || selected.constraint.arguments.len() != 2
            || selected
                .constraint
                .arguments
                .iter()
                .any(|ty| self.declared_method_type_kind(ty, scope) != Some(0))
        {
            return Err("Rust backend: unproven canonical Iterable constraint".into());
        }
        self.validate_parameter_evidence(selected, scope)?;
        self.validate_local_evidence(
            &LocalCallable::new(std::iter::empty(), &[selected.constraint.clone()], &[]),
            std::slice::from_ref(selected),
            scope,
        )?;
        if let CoreInstanceEvidence::Standard {
            identity,
            type_arguments,
            evidence_arguments,
        } = &selected.evidence
        {
            let (owner, count) = match identity.as_str() {
                "std/array::Iterable" => ("std/prelude::Array", 1),
                "std/list::Iterable" => ("std/prelude::List", 1),
                "std/iterator::Iterable" => ("std/prelude::Iterator", 1),
                "std/map::Iterable" => ("std/map::Map", 2),
                "std/set::Iterable" => ("std/set::Set", 1),
                _ => return Err("Rust backend: native Iterable provider needs an ABI proof".into()),
            };
            if !type_arguments.is_empty() || !evidence_arguments.is_empty() {
                return Err("Rust backend: Iterable provider type/child arity mismatch".into());
            }
            // These rank-polymorphic standard dictionaries carry their
            // instantiation in the checked constraint, not factory arguments.
            let actual = selected
                .constraint
                .arguments
                .iter()
                .map(|ty| self.local_canonical_type(ty, scope))
                .collect::<Result<Vec<_>>>()?;
            let CoreType::Named {
                name: actual_owner,
                arguments: slots,
            } = &actual[0]
            else {
                return Err(
                    "Rust backend: Iterable provider requires a canonical collection".into(),
                );
            };
            if actual_owner != owner || slots.len() != count {
                return Err(
                    "Rust backend: Iterable provider collection owner/arity mismatch".into(),
                );
            }
            let element = if count == 2 {
                CoreType::Tuple {
                    elements: slots.clone(),
                }
            } else {
                slots[0].clone()
            };
            if actual[1] != element {
                return Err("Rust backend: Iterable provider element mismatch".into());
            }
        }
        Ok(())
    }
    pub(super) fn iterable_dictionary(
        &mut self,
        selected: &CoreCallEvidence,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let CoreInstanceEvidence::Standard { identity, .. } = &selected.evidence else {
            return Ok(None);
        };
        if selected.constraint.trait_identity.as_deref() != Some("std/prelude::Iterable") {
            return Ok(None);
        }
        if self.program_wasm_library {
            return Err(
                "program WASM: native Iterable dictionary needs a target-specific ABI proof".into(),
            );
        }
        self.validate_iterable_evidence(selected, scope)?;
        self.capabilities.insert("iterable".into());
        Ok(Some(format!(
            "native_iterable_dictionary({})",
            quoted(identity)
        )))
    }
    fn iterable_build_type(
        &self,
        name: &str,
        arguments: &[CoreExpr],
        type_ref: &CoreType,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<usize> {
        if self.program_wasm_library {
            return Err(
                "program WASM: native Iterable constructor needs a target-specific ABI proof"
                    .into(),
            );
        }
        if scope.has_unproven_type_scope() || arguments.len() > 1 {
            return Err("Rust backend: unproven Iterable constructor source application".into());
        }
        let selected = evidence
            .first()
            .ok_or("Rust backend: Iterable constructor requires selected dictionary")?;
        self.validate_iterable_evidence(selected, scope)?;
        let collection = self.local_canonical_type(&selected.constraint.arguments[0], scope)?;
        let element = self.local_canonical_type(&selected.constraint.arguments[1], scope)?;
        let output = match name {
            "std/array::fromIterable" | "std/list::fromIterable" => {
                if evidence.len() != 1 {
                    return Err("Rust backend: unexpected Iterable constructor dictionary".into());
                }
                named(
                    if name.starts_with("std/array") {
                        "std/prelude::Array"
                    } else {
                        "std/prelude::List"
                    },
                    vec![element],
                )
            }
            _ => {
                let [_, eq, hash] = evidence else {
                    return Err(
                        "Rust backend: keyed Iterable constructor requires Iterable/Eq/Hash".into(),
                    );
                };
                self.validate_eq_evidence(eq, scope)?;
                self.validate_hash_evidence(hash, scope)?;
                let (key, output) = if name == "std/map::fromEntries" {
                    let CoreType::Tuple { elements } = &element else {
                        return Err("Rust backend: Map fromEntries requires pairs".into());
                    };
                    if elements.len() != 2 {
                        return Err("Rust backend: Map fromEntries pair arity".into());
                    }
                    (elements[0].clone(), named("std/map::Map", elements.clone()))
                } else {
                    (element.clone(), named("std/set::Set", vec![element]))
                };
                for selected in [eq, hash] {
                    if self.local_canonical_type(&selected.constraint.arguments[0], scope)? != key {
                        return Err(
                            "Rust backend: Iterable constructor key dictionary mismatch".into()
                        );
                    }
                }
                output
            }
        };
        let supplied = arguments
            .iter()
            .map(|argument| {
                local::method_argument_type(argument)
                    .ok_or_else(|| "Rust backend: missing Iterable constructor source type".into())
            })
            .collect::<Result<Vec<_>>>()?;
        let full = function(&supplied, type_ref.clone());
        if self.declared_method_type_kind(&full, scope) != Some(0)
            || self.local_canonical_type(&full, scope)? != function(&[collection], output)
        {
            return Err("Rust backend: Iterable constructor canonical signature mismatch".into());
        }
        Ok(1)
    }

    fn iterable_build_callable(
        &mut self,
        name: &str,
        arity: usize,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<String> {
        let dictionary = "\0iterable_build_dictionaries".to_owned();
        let names: Vec<_> = (0..arity)
            .map(|i| format!("\0iterable_build_argument_{i}"))
            .collect();
        let args = names
            .iter()
            .map(|n| format!("{}.clone()", id(n)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = format!(
            "{}({}, native_map_dictionaries(&{}), vec![{args}])",
            "native_iterable_build",
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

    pub(super) fn iterable_build_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if arity(name).is_none() || scope.contains(name) {
            return Ok(None);
        }
        let arity = self.iterable_build_type(name, &[], type_ref, evidence, scope)?;
        self.capabilities.insert("iterable-build".into());
        Ok(Some(
            self.iterable_build_callable(name, arity, evidence, scope)?,
        ))
    }

    pub(super) fn iterable_build_call(
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
        let arity = self.iterable_build_type(callee, arguments, type_ref, evidence, scope)?;
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
        self.capabilities.insert("iterable-build".into());
        if arguments.len() == arity {
            self.direct_calls += 1;
            let dictionaries = self.evidence(evidence, scope)?;
            return Ok(Some(format!(
                "{}({}, {dictionaries}, vec![{}])",
                "native_iterable_build",
                quoted(callee),
                self.expressions(arguments, scope)?
            )));
        }
        let value = self.fresh();
        let callable = self.iterable_build_callable(callee, arity, evidence, scope)?;
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
