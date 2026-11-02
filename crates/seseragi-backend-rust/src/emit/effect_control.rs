use super::*;

fn arity(name: &str) -> Option<usize> {
    match name {
        "std/effect::Continue" | "std/effect::Break" => Some(0),
        "std/effect::attempt" => Some(1),
        "std/effect::fromMaybe" | "std/effect::forEachUntil" => Some(2),
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
fn effect_arguments(ty: &CoreType) -> Result<&[CoreType]> {
    match ty {
        CoreType::Named { name, arguments }
            if name == "std/prelude::Effect" && arguments.len() == 3 =>
        {
            Ok(arguments)
        }
        _ => Err("Rust backend: Effect control requires canonical Effect".into()),
    }
}

impl Backend<'_> {
    fn effect_control_type(
        &self,
        name: &str,
        arguments: &[CoreExpr],
        result: &CoreType,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<usize> {
        if self.program_wasm_library {
            return Err(
                "program WASM: native Effect control requires a target-specific ABI proof".into(),
            );
        }
        let arity = arity(name).expect("known Effect control");
        if scope.has_unproven_type_scope() || arguments.len() > arity {
            return Err("Rust backend: unproven Effect control source application".into());
        }
        let supplied = arguments
            .iter()
            .map(|argument| {
                local::method_argument_type(argument)
                    .ok_or_else(|| "Rust backend: missing Effect control argument type".into())
            })
            .collect::<Result<Vec<_>>>()?;
        let full = function(&supplied, result.clone());
        if self.declared_method_type_kind(&full, scope) != Some(0) {
            return Err("Rust backend: Effect control requires value-kind source/result".into());
        }
        let full = self.local_canonical_type(&full, scope)?;
        let mut output = &full;
        let mut parameters = Vec::new();
        for _ in 0..arity {
            let CoreType::Function { parameter, result } = output else {
                return Err("Rust backend: missing Effect control source arity".into());
            };
            parameters.push(parameter.as_ref().clone());
            output = result;
        }
        let control = named("std/effect::LoopControl", vec![]);
        let expected = match name {
            "std/effect::Continue" | "std/effect::Break" => control,
            "std/effect::attempt" => {
                let slots = effect_arguments(&parameters[0])?;
                named(
                    "std/prelude::Effect",
                    vec![
                        slots[0].clone(),
                        named("std/prelude::Never", vec![]),
                        named("std/prelude::Either", slots[1..].to_vec()),
                    ],
                )
            }
            "std/effect::fromMaybe" => {
                let CoreType::Named { name, arguments } = &parameters[1] else {
                    return Err("Rust backend: fromMaybe requires canonical Maybe".into());
                };
                if name != "std/prelude::Maybe" || arguments.len() != 1 {
                    return Err(
                        "Rust backend: fromMaybe canonical Maybe owner/arity mismatch".into(),
                    );
                }
                named(
                    "std/prelude::Effect",
                    vec![
                        CoreType::Record {
                            closed: true,
                            fields: vec![],
                        },
                        parameters[0].clone(),
                        arguments[0].clone(),
                    ],
                )
            }
            "std/effect::forEachUntil" => {
                let [selected] = evidence else {
                    return Err("Rust backend: forEachUntil requires selected Iterable".into());
                };
                self.validate_iterable_evidence(selected, scope)?;
                let CoreType::Function { parameter, result } = &parameters[0] else {
                    return Err("Rust backend: forEachUntil requires an action".into());
                };
                let slots = effect_arguments(result)?;
                if self.local_canonical_type(&selected.constraint.arguments[0], scope)?
                    != parameters[1]
                    || self.local_canonical_type(&selected.constraint.arguments[1], scope)?
                        != **parameter
                    || slots[2] != control
                {
                    return Err(
                        "Rust backend: forEachUntil source, element or LoopControl mismatch".into(),
                    );
                }
                named(
                    "std/prelude::Effect",
                    vec![
                        slots[0].clone(),
                        slots[1].clone(),
                        named("std/prelude::Unit", vec![]),
                    ],
                )
            }
            _ => unreachable!(),
        };
        if name != "std/effect::forEachUntil" && !evidence.is_empty() {
            return Err("Rust backend: unexpected Effect control dictionary".into());
        }
        if *output != expected {
            return Err("Rust backend: Effect control canonical result mismatch".into());
        }
        Ok(arity)
    }

    fn effect_control_callable(
        &mut self,
        name: &str,
        arity: usize,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<String> {
        let dictionary = "\0effect_control_dictionaries".to_owned();
        let names: Vec<_> = (0..arity)
            .map(|i| format!("\0effect_control_argument_{i}"))
            .collect();
        let args = names
            .iter()
            .map(|n| format!("{}.clone()", id(n)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = format!(
            "native_effect_control({}, native_map_dictionaries(&{}), vec![{args}])",
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

    pub(super) fn effect_control_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if arity(name).is_none() || scope.contains(name) {
            return Ok(None);
        }
        let arity = self.effect_control_type(name, &[], type_ref, evidence, scope)?;
        self.capabilities.insert("effect-sync".into());
        Ok(Some(
            self.effect_control_callable(name, arity, evidence, scope)?,
        ))
    }

    pub(super) fn effect_control_call(
        &mut self,
        expr: &CoreExpr,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let (name, arguments, result, evidence) = match expr {
            CoreExpr::Call {
                callee,
                arguments,
                type_ref,
                evidence,
                ..
            } => (callee, arguments, type_ref.clone(), evidence.as_slice()),
            CoreExpr::EffectInvoke {
                callee,
                arguments,
                requirements,
                failure,
                success,
                evidence,
                ..
            } => (
                callee,
                arguments,
                named(
                    "std/prelude::Effect",
                    vec![requirements.clone(), failure.clone(), success.clone()],
                ),
                evidence.as_slice(),
            ),
            CoreExpr::EffectOperation {
                operation,
                arguments,
                requirements,
                failure,
                success,
                ..
            } => (
                operation,
                arguments,
                named(
                    "std/prelude::Effect",
                    vec![requirements.clone(), failure.clone(), success.clone()],
                ),
                &[][..],
            ),
            _ => return Ok(None),
        };
        if arity(name).is_none() || scope.contains(name) {
            return Ok(None);
        }
        if let CoreExpr::Call {
            trait_dispatch,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            ..
        } = expr
        {
            if trait_dispatch.is_some()
                || !deferred_evidence_type_constructor_parameters.is_empty()
                || evidence_argument_index.is_some()
            {
                return Err("Rust backend: unproven Effect control source stages".into());
            }
            if !deferred_evidence_parameters.is_empty() {
                if arguments.len() + deferred_evidence_parameters.len() != arity(name).unwrap() {
                    return Err(
                        "Rust backend: Effect control deferred source arity mismatch".into(),
                    );
                }
                let mut remaining = &result;
                for deferred in deferred_evidence_parameters {
                    let CoreType::Function { parameter, result } = remaining else {
                        return Err(
                            "Rust backend: missing Effect control deferred source stage".into()
                        );
                    };
                    if self.local_canonical_type(parameter, scope)?
                        != self.local_canonical_type(deferred, scope)?
                    {
                        return Err(
                            "Rust backend: Effect control deferred source type mismatch".into()
                        );
                    }
                    remaining = result;
                }
            }
        }
        let arity = self.effect_control_type(name, arguments, &result, evidence, scope)?;
        self.capabilities.insert("effect-sync".into());
        if arguments.len() == arity {
            self.direct_calls += 1;
            let dictionaries = self.evidence(evidence, scope)?;
            return Ok(Some(format!(
                "native_effect_control({}, {dictionaries}, vec![{}])",
                quoted(name),
                self.expressions(arguments, scope)?
            )));
        }
        let value = self.fresh();
        let callable = self.effect_control_callable(name, arity, evidence, scope)?;
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
