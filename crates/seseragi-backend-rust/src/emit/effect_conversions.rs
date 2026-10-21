use super::*;

impl Backend<'_> {
    pub(super) fn from_either_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if name != "std/effect::fromEither" || scope.contains(name) {
            return Ok(None);
        }
        if self.declared_method_type_kind(type_ref, scope) != Some(0) {
            return Err("Rust backend: unproven fromEither value kinds".into());
        }
        let ty = self.local_canonical_type(type_ref, scope)?;
        let CoreType::Function { parameter, result } = ty else {
            return Err("Rust backend: fromEither callable contract".into());
        };
        self.validate_from_either(&parameter, &result)?;
        if !evidence.is_empty() {
            return Err("Rust backend: fromEither evidence is unsupported".into());
        }
        self.capabilities.insert("effect-sync".into());
        Ok(Some(
            "V::fun(vec![], |_, value| native_operation(\"std/effect::fromEither\", vec![value]))"
                .into(),
        ))
    }
    fn validate_from_either(&self, input: &CoreType, output: &CoreType) -> Result<()> {
        let (
            CoreType::Named {
                name: either,
                arguments: payloads,
            },
            CoreType::Named {
                name: effect,
                arguments: outputs,
            },
        ) = (input, output)
        else {
            return Err("Rust backend: fromEither canonical type contract".into());
        };
        if either != "std/prelude::Either"
            || payloads.len() != 2
            || effect != "std/prelude::Effect"
            || outputs.len() != 3
            || outputs[1..] != payloads[..]
            || !matches!(&outputs[0], CoreType::Record { closed: true, fields } if fields.is_empty())
        {
            return Err("Rust backend: fromEither canonical failure/success contract".into());
        }
        Ok(())
    }
    pub(super) fn from_either_call(
        &mut self,
        expr: &CoreExpr,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let (callee, arguments, output, valid) = match expr {
            CoreExpr::Call {
                callee,
                arguments,
                type_ref,
                evidence,
                trait_dispatch,
                deferred_evidence_parameters,
                deferred_evidence_type_constructor_parameters,
                evidence_argument_index,
                ..
            } => (
                callee,
                arguments,
                type_ref.clone(),
                evidence.is_empty()
                    && trait_dispatch.is_none()
                    && deferred_evidence_parameters.is_empty()
                    && deferred_evidence_type_constructor_parameters.is_empty()
                    && evidence_argument_index.is_none(),
            ),
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
                CoreType::ExternalNamed {
                    name: "Effect".into(),
                    canonical: "std/prelude::Effect".into(),
                    arguments: vec![requirements.clone(), failure.clone(), success.clone()],
                },
                evidence.is_empty(),
            ),
            _ => return Ok(None),
        };
        if callee != "std/effect::fromEither" || scope.contains(callee) {
            return Ok(None);
        }
        if !valid || arguments.len() != 1 {
            return Err("Rust backend: unproven fromEither source application".into());
        }
        let input = local::method_argument_type(&arguments[0])
            .ok_or("Rust backend: missing fromEither argument type")?;
        if self.declared_method_type_kind(&input, scope) != Some(0)
            || self.declared_method_type_kind(&output, scope) != Some(0)
        {
            return Err("Rust backend: unproven fromEither argument/result kinds".into());
        }
        self.validate_from_either(
            &self.local_canonical_type(&input, scope)?,
            &self.local_canonical_type(&output, scope)?,
        )?;
        self.capabilities.insert("effect-sync".into());
        Ok(Some(format!(
            "native_operation(\"std/effect::fromEither\", vec![{}])",
            self.expr(&arguments[0], scope)?
        )))
    }
}
