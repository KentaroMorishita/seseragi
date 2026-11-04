use super::*;

fn ty(name: &str) -> CoreType {
    named(name, vec![])
}

fn named(name: &str, arguments: Vec<CoreType>) -> CoreType {
    CoreType::Named {
        name: name.into(),
        arguments,
    }
}

fn signature(name: &str) -> Option<CoreType> {
    let (parameter, result) = match name {
        "std/entropy::entropySize" => (
            ty("std/prelude::Int"),
            named(
                "std/prelude::Either",
                vec![
                    ty("std/entropy::EntropyConfigError"),
                    ty("std/entropy::EntropySize"),
                ],
            ),
        ),
        "std/entropy::secureBytes" => (
            ty("std/entropy::EntropySize"),
            named(
                "std/prelude::Effect",
                vec![
                    CoreType::Record {
                        closed: true,
                        fields: vec![seseragi_lowering::CoreRecordField {
                            name: "entropy".into(),
                            optional: false,
                            type_ref: ty("std/entropy::Entropy"),
                        }],
                    },
                    ty("std/entropy::EntropyError"),
                    ty("std/bytes::Bytes"),
                ],
            ),
        ),
        _ => return None,
    };
    Some(CoreType::Function {
        parameter: Box::new(parameter),
        result: Box::new(result),
    })
}

impl Backend<'_> {
    pub(super) fn entropy_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let Some(expected) = signature(name) else {
            return Ok(None);
        };
        if scope.contains(name) {
            return Ok(None);
        }
        if self.program_wasm_library {
            return Err("program WASM: native Entropy requires a target-specific ABI proof".into());
        }
        if !evidence.is_empty()
            || scope.has_unproven_type_scope()
            || self.declared_method_type_kind(type_ref, scope) != Some(0)
            || self.local_canonical_type(type_ref, scope)? != expected
        {
            return Err("Rust backend: unproven canonical Entropy signature or evidence".into());
        }
        if name == "std/entropy::secureBytes" {
            self.capabilities.insert("entropy".into());
            self.capabilities.insert("effect-sync".into());
        }
        let argument = "\0entropy_argument";
        Ok(Some(Self::closure(
            argument,
            &format!(
                "native_entropy_operation({}, vec![{}.clone()])",
                quoted(name),
                id(argument)
            ),
            &Scope::new(),
        )))
    }

    pub(super) fn entropy_call(
        &mut self,
        expr: &CoreExpr,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let (name, arguments, mut result, valid, saturated) = match expr {
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
                false,
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
                named(
                    "std/prelude::Effect",
                    vec![requirements.clone(), failure.clone(), success.clone()],
                ),
                evidence.is_empty(),
                true,
            ),
            _ => return Ok(None),
        };
        if signature(name).is_none() || scope.contains(name) {
            return Ok(None);
        }
        if !valid || arguments.len() > 1 || (saturated && arguments.len() != 1) {
            return Err("Rust backend: unproven Entropy source application".into());
        }
        if let Some(argument) = arguments.first() {
            let parameter = local::method_argument_type(argument)
                .ok_or("Rust backend: missing Entropy argument type")?;
            result = CoreType::Function {
                parameter: Box::new(parameter),
                result: Box::new(result),
            };
        }
        let function = self.entropy_reference(name, &[], &result, scope)?.unwrap();
        let temporary = self.fresh();
        let mut body = format!("{{ let mut {temporary} = {function};");
        for argument in arguments {
            body.push_str(&format!(
                "{temporary} = {temporary}.apply({});",
                self.expr(argument, scope)?
            ));
        }
        body.push_str(&format!("{temporary} }}"));
        Ok(Some(body))
    }
}
