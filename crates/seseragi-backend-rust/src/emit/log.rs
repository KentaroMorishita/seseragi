use super::*;

fn named(name: &str, arguments: Vec<CoreType>) -> CoreType {
    CoreType::Named {
        name: name.into(),
        arguments,
    }
}
fn value(name: &str) -> CoreType {
    named(name, vec![])
}
fn arrow(parameters: &[CoreType], result: &CoreType) -> CoreType {
    parameters
        .iter()
        .rev()
        .fold(result.clone(), |result, parameter| CoreType::Function {
            parameter: Box::new(parameter.clone()),
            result: Box::new(result),
        })
}
fn signature(name: &str) -> Option<(Vec<CoreType>, CoreType)> {
    Some(match name {
        "std/log::LogTrace"
        | "std/log::LogDebug"
        | "std/log::LogInfo"
        | "std/log::LogWarn"
        | "std/log::LogFailure" => (vec![], value("std/log::LogLevel")),
        "std/log::LogString" | "std/log::LogInt" | "std/log::LogFloat" | "std/log::LogBool" => (
            vec![value(match name {
                "std/log::LogString" => "std/prelude::String",
                "std/log::LogInt" => "std/prelude::Int",
                "std/log::LogFloat" => "std/prelude::Float",
                _ => "std/prelude::Bool",
            })],
            value("std/log::LogValue"),
        ),
        "std/log::log" => (
            vec![value("std/log::LogEvent")],
            named(
                "std/prelude::Effect",
                vec![
                    CoreType::Record {
                        closed: true,
                        fields: vec![seseragi_lowering::CoreRecordField {
                            name: "logger".into(),
                            optional: false,
                            type_ref: value("std/log::Logger"),
                        }],
                    },
                    value("std/log::LogError"),
                    value("std/prelude::Unit"),
                ],
            ),
        ),
        _ => return None,
    })
}

impl Backend<'_> {
    fn log_type_matches(
        &self,
        actual: &CoreType,
        expected: &CoreType,
        scope: &Scope,
    ) -> Result<()> {
        if self.declared_method_type_kind(actual, scope) != Some(0)
            || self.local_canonical_type(actual, scope)? != *expected
        {
            return Err("Rust backend: unproven canonical Logger argument or result".into());
        }
        Ok(())
    }

    pub(super) fn log_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let Some((parameters, result)) = signature(name) else {
            return Ok(None);
        };
        if scope.contains(name) {
            return Ok(None);
        }
        if self.program_wasm_library {
            return Err("Rust backend: native Logger is not admitted in program WASM".into());
        }
        if !evidence.is_empty() {
            return Err("Rust backend: unsupported Logger evidence".into());
        }
        self.log_type_matches(type_ref, &arrow(&parameters, &result), scope)?;
        self.capabilities.insert("logger".into());
        if name == "std/log::log" {
            self.capabilities.insert("effect-sync".into());
        }
        Ok(Some(if parameters.is_empty() {
            format!("native_log_operation({}, vec![])", quoted(name))
        } else {
            format!(
                "V::fun(vec![], |_, argument| native_log_operation({}, vec![argument]))",
                quoted(name)
            )
        }))
    }

    pub(super) fn log_call(&mut self, expr: &CoreExpr, scope: &Scope) -> Result<Option<String>> {
        let (name, arguments, result, valid, saturated, checked_signature) = match expr {
            CoreExpr::Call {
                callee,
                arguments,
                type_ref,
                evidence,
                trait_dispatch,
                deferred_evidence_parameters,
                deferred_evidence_type_constructor_parameters,
                evidence_argument_index,
                callee_signature,
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
                callee_signature.as_ref(),
            ),
            CoreExpr::EffectInvoke {
                callee,
                arguments,
                requirements,
                failure,
                success,
                evidence,
                callee_signature,
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
                callee_signature.as_ref(),
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
                true,
                true,
                None,
            ),
            _ => return Ok(None),
        };
        let Some((parameters, expected)) = signature(name) else {
            return Ok(None);
        };
        if scope.contains(name) {
            return Ok(None);
        }
        if self.program_wasm_library {
            return Err("Rust backend: native Logger is not admitted in program WASM".into());
        }
        if !valid
            || arguments.len() > parameters.len()
            || (saturated && (name != "std/log::log" || arguments.len() != parameters.len()))
        {
            return Err("Rust backend: unproven Logger source application".into());
        }
        if let Some(signature) = checked_signature {
            if !signature.type_parameters.is_empty()
                || !signature.instantiated_type_arguments.is_empty()
                || signature.parameters.len() != parameters.len()
            {
                return Err("Rust backend: unproven Logger declaration arity".into());
            }
            for (actual, expected) in signature.parameters.iter().zip(&parameters) {
                self.log_type_matches(actual, expected, scope)?;
            }
            self.log_type_matches(&signature.result, &expected, scope)?;
            if !signature.argument_types.is_empty() {
                if signature.argument_types.len() != arguments.len() {
                    return Err("Rust backend: unproven Logger argument arity".into());
                }
                for (actual, expected) in signature.argument_types.iter().zip(&parameters) {
                    self.log_type_matches(actual, expected, scope)?;
                }
            }
        }
        for (argument, expected) in arguments.iter().zip(&parameters) {
            let actual = local::method_argument_type(argument)
                .ok_or("Rust backend: missing Logger argument type")?;
            self.log_type_matches(&actual, expected, scope)?;
        }
        self.log_type_matches(
            &result,
            &arrow(&parameters[arguments.len()..], &expected),
            scope,
        )?;
        let function = self
            .log_reference(name, &[], &arrow(&parameters, &expected), scope)?
            .unwrap();
        Ok(Some(if let Some(argument) = arguments.first() {
            format!("({function}).apply({})", self.expr(argument, scope)?)
        } else {
            function
        }))
    }
}
