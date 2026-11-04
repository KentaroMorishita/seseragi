use super::*;

fn arity(name: &str) -> Option<usize> {
    match name {
        "std/transformer/state::run" => Some(2),
        "std/transformer/state::get"
        | "std/transformer/state::put"
        | "std/transformer/state::modify"
        | "std/transformer/state::lift" => Some(1),
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

fn apply(constructor: &CoreType, argument: CoreType) -> Result<CoreType> {
    let mut applied = constructor.clone();
    match &mut applied {
        CoreType::Named { arguments, .. } | CoreType::ExternalNamed { arguments, .. } => {
            arguments.push(argument);
            Ok(applied)
        }
        CoreType::TypeFunction { .. } => {
            seseragi_syntax::type_functions::apply_type_function(constructor, &[argument])
                .ok_or_else(|| "Rust backend: unproven StateT base application".into())
        }
        _ => Err("Rust backend: unproven StateT base constructor".into()),
    }
}

impl Backend<'_> {
    fn state_transformer_type(
        &self,
        name: &str,
        arguments: &[CoreExpr],
        type_ref: &CoreType,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<()> {
        if self.program_wasm_library {
            return Err(
                "program WASM: native StateT operation requires a target-specific proof".into(),
            );
        }
        let arity = arity(name).expect("known StateT operation");
        if scope.has_unproven_type_scope() {
            return Err("Rust backend: unproven StateT source application".into());
        }
        let parameters = arguments
            .iter()
            .map(|argument| {
                local::method_argument_type(argument)
                    .ok_or_else(|| "Rust backend: missing StateT argument type".into())
            })
            .collect::<Result<Vec<_>>>()?;
        let full = function(&parameters, type_ref.clone());
        if self.declared_method_type_kind(&full, scope) != Some(0) {
            return Err("Rust backend: StateT source/result must have value kind".into());
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
                return Err("Rust backend: missing StateT source stage".into());
            };
            parameters.push(parameter.as_ref());
            result = next;
        }
        let run = name == "std/transformer/state::run";
        let CoreType::Named {
            name: owner,
            arguments: types,
        } = (if run { parameters[1] } else { result })
        else {
            return Err("Rust backend: StateT requires its canonical owner".into());
        };
        let [state, base, item] = types.as_slice() else {
            return Err("Rust backend: StateT type arity mismatch".into());
        };
        if owner != "std/transformer/state::StateT"
            || self.declared_method_type_kind(state, scope) != Some(0)
            || self.declared_method_type_kind(base, scope) != Some(1)
            || self.declared_method_type_kind(item, scope) != Some(0)
        {
            return Err("Rust backend: unproven StateT owner or kinds".into());
        }
        let wrapped = CoreType::Named {
            name: owner.clone(),
            arguments: types.clone(),
        };
        let unit = CoreType::Named {
            name: "std/prelude::Unit".into(),
            arguments: vec![],
        };
        let expected = if run {
            if !evidence.is_empty() {
                return Err("Rust backend: unexpected StateT run dictionary".into());
            }
            function(
                &[state.clone(), wrapped],
                apply(
                    base,
                    CoreType::Tuple {
                        elements: vec![item.clone(), state.clone()],
                    },
                )?,
            )
        } else {
            let [selected] = evidence else {
                return Err("Rust backend: StateT requires one base Monad dictionary".into());
            };
            if selected.constraint.name != "Monad"
                || selected.constraint.trait_identity.as_deref() != Some("std/prelude::Monad")
                || selected.constraint.arguments.len() != 1
                || self.local_canonical_type(&selected.constraint.arguments[0], scope)? != *base
            {
                return Err("Rust backend: StateT base Monad mismatch".into());
            }
            self.validate_parameter_evidence(selected, scope)?;
            self.validate_local_evidence(
                &LocalCallable::new(std::iter::empty(), &[selected.constraint.clone()], &[]),
                evidence,
                scope,
            )?;
            let parameter = match name {
                "std/transformer/state::get" => {
                    if item != state {
                        return Err("Rust backend: StateT get result/state mismatch".into());
                    }
                    unit
                }
                "std/transformer/state::put" | "std/transformer/state::modify" => {
                    if *item != unit {
                        return Err("Rust backend: StateT update requires Unit result".into());
                    }
                    if name == "std/transformer/state::put" {
                        state.clone()
                    } else {
                        function(std::slice::from_ref(state), state.clone())
                    }
                }
                "std/transformer/state::lift" => apply(base, item.clone())?,
                _ => unreachable!(),
            };
            function(&[parameter], wrapped)
        };
        self.match_local_type(
            &expected,
            &full,
            &BTreeMap::new(),
            &mut BTreeMap::new(),
            scope,
        )
    }

    pub(super) fn state_transformer_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if arity(name).is_none() || scope.contains(name) {
            return Ok(None);
        }
        self.state_transformer_type(name, &[], type_ref, evidence, scope)?;
        self.capabilities.insert("state-transformer".into());
        Ok(Some(format!(
            "native_state_transformer_callable({}, {})",
            quoted(name),
            self.evidence(evidence, scope)?
        )))
    }

    pub(super) fn state_transformer_call(
        &mut self,
        expr: &CoreExpr,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let effect_type;
        let (callee, arguments, evidence, type_ref) = match expr {
            CoreExpr::Call {
                callee,
                arguments,
                evidence,
                type_ref,
                trait_dispatch,
                deferred_evidence_parameters,
                deferred_evidence_type_constructor_parameters,
                evidence_argument_index,
                ..
            } if arity(callee).is_some() && !scope.contains(callee) => {
                if trait_dispatch.is_some()
                    || !deferred_evidence_parameters.is_empty()
                    || !deferred_evidence_type_constructor_parameters.is_empty()
                    || *evidence_argument_index
                        != (arguments.len() > arity(callee).unwrap())
                            .then_some(arity(callee).unwrap())
                {
                    return Err("Rust backend: unproven StateT source stages".into());
                }
                (callee, arguments, evidence, type_ref)
            }
            CoreExpr::EffectInvoke {
                callee,
                arguments,
                evidence,
                requirements,
                failure,
                success,
                ..
            } if arity(callee).is_some() && !scope.contains(callee) => {
                effect_type = CoreType::Named {
                    name: "std/prelude::Effect".into(),
                    arguments: vec![requirements.clone(), failure.clone(), success.clone()],
                };
                (callee, arguments, evidence, &effect_type)
            }
            _ => return Ok(None),
        };
        self.state_transformer_type(callee, arguments, type_ref, evidence, scope)?;
        self.capabilities.insert("state-transformer".into());
        let mut value = format!(
            "native_state_transformer_callable({}, {})",
            quoted(callee),
            self.evidence(evidence, scope)?
        );
        for argument in arguments {
            value = format!("({value}).apply({})", self.expr(argument, scope)?);
        }
        Ok(Some(value))
    }
}
