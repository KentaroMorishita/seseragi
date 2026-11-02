use super::*;

fn operation(name: &str) -> bool {
    matches!(
        name,
        "std/transformer/writer::run"
            | "std/transformer/writer::tell"
            | "std/transformer/writer::listen"
            | "std/transformer/writer::lift"
    )
}

fn named(name: &str, arguments: Vec<CoreType>) -> CoreType {
    CoreType::Named {
        name: name.into(),
        arguments,
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
                .ok_or_else(|| "Rust backend: unproven WriterT base application".into())
        }
        _ => Err("Rust backend: unproven WriterT base constructor".into()),
    }
}

impl Backend<'_> {
    fn writer_transformer_type(
        &self,
        name: &str,
        arguments: &[CoreExpr],
        type_ref: &CoreType,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<()> {
        if self.program_wasm_library {
            return Err(
                "program WASM: native WriterT operation requires a target-specific proof".into(),
            );
        }
        if scope.has_unproven_type_scope() {
            return Err("Rust backend: unproven WriterT source scope".into());
        }
        let full = arguments
            .iter()
            .rev()
            .try_fold(type_ref.clone(), |result, arg| {
                Ok::<_, RustBackendError>(CoreType::Function {
                    parameter: Box::new(
                        local::method_argument_type(arg)
                            .ok_or("Rust backend: missing WriterT argument type")?,
                    ),
                    result: Box::new(result),
                })
            })?;
        if self.declared_method_type_kind(&full, scope) != Some(0) {
            return Err("Rust backend: WriterT source/result must have value kind".into());
        }
        let full = self.local_canonical_type(&full, scope)?;
        let CoreType::Function { parameter, result } = &full else {
            return Err("Rust backend: missing WriterT source stage".into());
        };
        let run = name == "std/transformer/writer::run";
        let listen = name == "std/transformer/writer::listen";
        let CoreType::Named {
            name: owner,
            arguments: types,
        } = (if run || listen {
            parameter.as_ref()
        } else {
            result.as_ref()
        })
        else {
            return Err("Rust backend: WriterT requires its canonical owner".into());
        };
        let [output, base, item] = types.as_slice() else {
            return Err("Rust backend: WriterT type arity mismatch".into());
        };
        if owner != "std/transformer/writer::WriterT"
            || self.declared_method_type_kind(output, scope) != Some(0)
            || self.declared_method_type_kind(base, scope) != Some(1)
            || self.declared_method_type_kind(item, scope) != Some(0)
        {
            return Err("Rust backend: unproven WriterT owner or kinds".into());
        }
        let wrapped = named(owner, types.clone());
        let pair = CoreType::Tuple {
            elements: vec![item.clone(), output.clone()],
        };
        let (parameter, result) = if run {
            if !evidence.is_empty() {
                return Err("Rust backend: unexpected WriterT run dictionary".into());
            }
            (wrapped, apply(base, pair)?)
        } else {
            if evidence.len() != 2 {
                return Err(
                    "Rust backend: WriterT requires base Monad and output Monoid dictionaries"
                        .into(),
                );
            }
            for (selected, (label, expected)) in
                evidence.iter().zip([("Monad", base), ("Monoid", output)])
            {
                if selected.constraint.name != label
                    || selected.constraint.trait_identity.as_deref()
                        != Some(format!("std/prelude::{label}").as_str())
                    || selected.constraint.arguments.len() != 1
                    || self.local_canonical_type(&selected.constraint.arguments[0], scope)?
                        != *expected
                {
                    return Err("Rust backend: WriterT selected dictionary mismatch".into());
                }
                self.validate_parameter_evidence(selected, scope)?;
            }
            let constraints = evidence
                .iter()
                .map(|selected| selected.constraint.clone())
                .collect::<Vec<_>>();
            self.validate_local_evidence(
                &LocalCallable::new(std::iter::empty(), &constraints, &[]),
                evidence,
                scope,
            )?;
            match name {
                "std/transformer/writer::tell" => {
                    if *item != named("std/prelude::Unit", vec![]) {
                        return Err("Rust backend: WriterT tell requires Unit result".into());
                    }
                    (output.clone(), wrapped)
                }
                "std/transformer/writer::listen" => (
                    wrapped,
                    named(owner, vec![output.clone(), base.clone(), pair]),
                ),
                "std/transformer/writer::lift" => (apply(base, item.clone())?, wrapped),
                _ => unreachable!(),
            }
        };
        self.match_local_type(
            &CoreType::Function {
                parameter: Box::new(parameter),
                result: Box::new(result),
            },
            &full,
            &BTreeMap::new(),
            &mut BTreeMap::new(),
            scope,
        )
    }

    pub(super) fn writer_transformer_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if !operation(name) || scope.contains(name) {
            return Ok(None);
        }
        self.writer_transformer_type(name, &[], type_ref, evidence, scope)?;
        self.capabilities.insert("writer-transformer".into());
        Ok(Some(format!(
            "native_writer_transformer_callable({}, {})",
            quoted(name),
            self.evidence(evidence, scope)?
        )))
    }

    pub(super) fn writer_transformer_call(
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
            } if operation(callee) && !scope.contains(callee) => {
                if trait_dispatch.is_some()
                    || !deferred_evidence_parameters.is_empty()
                    || !deferred_evidence_type_constructor_parameters.is_empty()
                    || *evidence_argument_index != (arguments.len() > 1).then_some(1)
                {
                    return Err("Rust backend: unproven WriterT source stages".into());
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
            } if operation(callee) && !scope.contains(callee) => {
                effect_type = named(
                    "std/prelude::Effect",
                    vec![requirements.clone(), failure.clone(), success.clone()],
                );
                (callee, arguments, evidence, &effect_type)
            }
            _ => return Ok(None),
        };
        self.writer_transformer_type(callee, arguments, type_ref, evidence, scope)?;
        self.capabilities.insert("writer-transformer".into());
        let mut value = format!(
            "native_writer_transformer_callable({}, {})",
            quoted(callee),
            self.evidence(evidence, scope)?
        );
        for argument in arguments {
            value = format!("({value}).apply({})", self.expr(argument, scope)?);
        }
        Ok(Some(value))
    }
}
