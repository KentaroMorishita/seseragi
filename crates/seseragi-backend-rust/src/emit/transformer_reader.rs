use super::*;

fn arity(name: &str) -> Option<usize> {
    match name {
        "std/transformer/reader::run" | "std/transformer/reader::local" => Some(2),
        "std/transformer/reader::ask"
        | "std/transformer/reader::asks"
        | "std/transformer/reader::lift" => Some(1),
        _ => None,
    }
}

fn named(name: &str, arguments: Vec<CoreType>) -> CoreType {
    CoreType::Named {
        name: name.into(),
        arguments,
    }
}

fn arrow(parameter: CoreType, result: CoreType) -> CoreType {
    CoreType::Function {
        parameter: Box::new(parameter),
        result: Box::new(result),
    }
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
                .ok_or_else(|| "Rust backend: unproven ReaderT base application".into())
        }
        _ => Err("Rust backend: unproven ReaderT base constructor".into()),
    }
}

impl Backend<'_> {
    fn reader_transformer_type(
        &self,
        name: &str,
        arguments: &[CoreExpr],
        type_ref: &CoreType,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<()> {
        if self.program_wasm_library {
            return Err(
                "program WASM: native ReaderT operation requires a target-specific proof".into(),
            );
        }
        if scope.has_unproven_type_scope() {
            return Err("Rust backend: unproven ReaderT source scope".into());
        }
        let full = arguments
            .iter()
            .rev()
            .try_fold(type_ref.clone(), |result, argument| {
                let parameter = local::method_argument_type(argument)
                    .ok_or("Rust backend: missing ReaderT argument type")?;
                Ok::<_, RustBackendError>(arrow(parameter, result))
            })?;
        if self.declared_method_type_kind(&full, scope) != Some(0) {
            return Err("Rust backend: ReaderT source/result must have value kind".into());
        }
        let full = self.local_canonical_type(&full, scope)?;
        let mut result = &full;
        let mut parameters = Vec::new();
        for _ in 0..arity(name).unwrap() {
            let CoreType::Function {
                parameter,
                result: next,
            } = result
            else {
                return Err("Rust backend: missing ReaderT source stage".into());
            };
            parameters.push(parameter.as_ref());
            result = next;
        }
        let run = name == "std/transformer/reader::run";
        let CoreType::Named {
            name: owner,
            arguments: types,
        } = (if run { parameters[1] } else { result })
        else {
            return Err("Rust backend: ReaderT requires its canonical owner".into());
        };
        let [environment, base, item] = types.as_slice() else {
            return Err("Rust backend: ReaderT type arity mismatch".into());
        };
        if owner != "std/transformer/reader::ReaderT"
            || self.declared_method_type_kind(environment, scope) != Some(0)
            || self.declared_method_type_kind(base, scope) != Some(1)
            || self.declared_method_type_kind(item, scope) != Some(0)
        {
            return Err("Rust backend: unproven ReaderT owner or kinds".into());
        }
        if run {
            if !evidence.is_empty() {
                return Err("Rust backend: unexpected ReaderT run dictionary".into());
            }
        } else {
            let [selected] = evidence else {
                return Err("Rust backend: ReaderT requires one base Monad dictionary".into());
            };
            if selected.constraint.name != "Monad"
                || selected.constraint.trait_identity.as_deref() != Some("std/prelude::Monad")
                || selected.constraint.arguments.len() != 1
                || self.local_canonical_type(&selected.constraint.arguments[0], scope)? != *base
            {
                return Err("Rust backend: ReaderT base Monad mismatch".into());
            }
            self.validate_parameter_evidence(selected, scope)?;
            self.validate_local_evidence(
                &LocalCallable::new(std::iter::empty(), &[selected.constraint.clone()], &[]),
                evidence,
                scope,
            )?;
        }
        let wrapped = named(owner, types.clone());
        let (parameters, result) = match name {
            "std/transformer/reader::run" => (
                vec![environment.clone(), wrapped],
                apply(base, item.clone())?,
            ),
            "std/transformer/reader::ask" => {
                if item != environment {
                    return Err("Rust backend: ReaderT ask must return its environment type".into());
                }
                (vec![named("std/prelude::Unit", vec![])], wrapped)
            }
            "std/transformer/reader::asks" => {
                (vec![arrow(environment.clone(), item.clone())], wrapped)
            }
            "std/transformer/reader::local" => (
                vec![
                    arrow(environment.clone(), environment.clone()),
                    wrapped.clone(),
                ],
                wrapped,
            ),
            "std/transformer/reader::lift" => (vec![apply(base, item.clone())?], wrapped),
            _ => unreachable!(),
        };
        let expected = parameters
            .into_iter()
            .rev()
            .fold(result, |result, parameter| arrow(parameter, result));
        self.match_local_type(
            &expected,
            &full,
            &BTreeMap::new(),
            &mut BTreeMap::new(),
            scope,
        )
    }

    pub(super) fn reader_transformer_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if arity(name).is_none() || scope.contains(name) {
            return Ok(None);
        }
        self.reader_transformer_type(name, &[], type_ref, evidence, scope)?;
        self.capabilities.insert("reader-transformer".into());
        Ok(Some(format!(
            "native_reader_transformer_callable({}, {})",
            quoted(name),
            self.evidence(evidence, scope)?
        )))
    }

    pub(super) fn reader_transformer_call(
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
                let arity = arity(callee).unwrap();
                if trait_dispatch.is_some()
                    || (!deferred_evidence_type_constructor_parameters.is_empty()
                        && (deferred_evidence_type_constructor_parameters != &["M".to_owned()]
                            || deferred_evidence_parameters.is_empty()
                            || arguments.len() >= arity))
                    || *evidence_argument_index != (arguments.len() > arity).then_some(arity)
                {
                    return Err("Rust backend: unproven ReaderT source stages".into());
                }
                if !deferred_evidence_parameters.is_empty() {
                    if arguments.len() + deferred_evidence_parameters.len() != arity {
                        return Err("Rust backend: ReaderT deferred source arity mismatch".into());
                    }
                    let mut remaining = type_ref;
                    for deferred in deferred_evidence_parameters {
                        let CoreType::Function { parameter, result } = remaining else {
                            return Err(
                                "Rust backend: missing ReaderT deferred source stage".into()
                            );
                        };
                        if self.local_canonical_type(parameter, scope)?
                            != self.local_canonical_type(deferred, scope)?
                        {
                            return Err(
                                "Rust backend: ReaderT deferred source type mismatch".into()
                            );
                        }
                        remaining = result;
                    }
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
                effect_type = named(
                    "std/prelude::Effect",
                    vec![requirements.clone(), failure.clone(), success.clone()],
                );
                (callee, arguments, evidence, &effect_type)
            }
            _ => return Ok(None),
        };
        self.reader_transformer_type(callee, arguments, type_ref, evidence, scope)?;
        self.capabilities.insert("reader-transformer".into());
        let mut value = format!(
            "native_reader_transformer_callable({}, {})",
            quoted(callee),
            self.evidence(evidence, scope)?
        );
        for argument in arguments {
            value = format!("({value}).apply({})", self.expr(argument, scope)?);
        }
        Ok(Some(value))
    }
}
