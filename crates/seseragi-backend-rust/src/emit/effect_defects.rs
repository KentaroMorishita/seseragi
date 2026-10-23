use super::*;

impl Backend<'_> {
    fn validate_die(&self, input: &CoreType, output: &CoreType, scope: &Scope) -> Result<()> {
        if self.declared_method_type_kind(input, scope) != Some(0)
            || self.declared_method_type_kind(output, scope) != Some(0)
        {
            return Err("Rust backend: unproven die argument/result kinds".into());
        }
        let input = self.local_canonical_type(input, scope)?;
        let output = self.local_canonical_type(output, scope)?;
        let never = CoreType::Named {
            name: "std/prelude::Never".into(),
            arguments: vec![],
        };
        if !matches!(input, CoreType::Named { name, arguments } if name == "std/prelude::String" && arguments.is_empty())
            || !matches!(output, CoreType::Named { name, arguments }
                if name == "std/prelude::Effect"
                    && arguments.len() == 3
                    && matches!(&arguments[0], CoreType::Record { closed: true, fields } if fields.is_empty())
                    && arguments[1] == never
                    && arguments[2] == never)
        {
            return Err("Rust backend: die canonical String/Effect contract mismatch".into());
        }
        Ok(())
    }

    fn require_die(&mut self) {
        self.capabilities.insert("effect-sync".into());
        self.capabilities.insert("effect-defect".into());
    }

    pub(super) fn die_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if name != "std/effect::die" || scope.contains(name) {
            return Ok(None);
        }
        if !evidence.is_empty() {
            return Err("Rust backend: die evidence is unsupported".into());
        }
        let CoreType::Function { parameter, result } = type_ref else {
            return Err("Rust backend: unproven die callable contract".into());
        };
        self.validate_die(parameter, result, scope)?;
        self.require_die();
        Ok(Some(
            "V::fun(vec![], |_, message| native_effect_die(message))".into(),
        ))
    }

    pub(super) fn die_call(&mut self, expr: &CoreExpr, scope: &Scope) -> Result<Option<String>> {
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
        if callee != "std/effect::die" || scope.contains(callee) {
            return Ok(None);
        }
        if !valid || arguments.len() != 1 {
            return Err("Rust backend: unproven die source application".into());
        }
        self.emit_die(arguments, &output, scope).map(Some)
    }

    pub(super) fn die_operation(
        &mut self,
        expr: &CoreExpr,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let CoreExpr::EffectOperation {
            operation,
            requirements,
            failure,
            success,
            arguments,
            ..
        } = expr
        else {
            return Ok(None);
        };
        if operation != "effect.die" {
            return Ok(None);
        }
        if arguments.len() != 1 {
            return Err("Rust backend: unproven die operation arity".into());
        }
        let output = CoreType::ExternalNamed {
            name: "Effect".into(),
            canonical: "std/prelude::Effect".into(),
            arguments: vec![requirements.clone(), failure.clone(), success.clone()],
        };
        self.emit_die(arguments, &output, scope).map(Some)
    }

    fn emit_die(
        &mut self,
        arguments: &[CoreExpr],
        output: &CoreType,
        scope: &Scope,
    ) -> Result<String> {
        let input = local::method_argument_type(&arguments[0])
            .ok_or("Rust backend: unproven die message type")?;
        self.validate_die(&input, output, scope)?;
        self.require_die();
        Ok(format!(
            "native_effect_die({})",
            self.expr(&arguments[0], scope)?
        ))
    }
}
