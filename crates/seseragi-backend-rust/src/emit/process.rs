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
fn signature(name: &str) -> Option<(CoreType, CoreType)> {
    let string = || value("std/prelude::String");
    let (input, success) = match name {
        "std/process::arguments" => (
            value("std/prelude::Unit"),
            named("std/prelude::Array", vec![string()]),
        ),
        "std/process::environment" => (string(), named("std/prelude::Maybe", vec![string()])),
        "std/process::currentDirectory" => (value("std/prelude::Unit"), value("std/path::Path")),
        _ => return None,
    };
    Some((
        input,
        named(
            "std/prelude::Effect",
            vec![
                CoreType::Record {
                    closed: true,
                    fields: vec![seseragi_lowering::CoreRecordField {
                        name: "process".into(),
                        optional: false,
                        type_ref: value("std/process::Process"),
                    }],
                },
                value("std/process::ProcessError"),
                success,
            ],
        ),
    ))
}

impl Backend<'_> {
    fn process_type_matches(
        &self,
        actual: &CoreType,
        expected: &CoreType,
        scope: &Scope,
    ) -> Result<()> {
        if self.declared_method_type_kind(actual, scope) != Some(0)
            || self.local_canonical_type(actual, scope)? != *expected
        {
            return Err("Rust backend: unproven canonical Process argument or result".into());
        }
        Ok(())
    }

    pub(super) fn process_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let Some((parameter, result)) = signature(name) else {
            return Ok(None);
        };
        if scope.contains(name) {
            return Ok(None);
        }
        if self.program_wasm_library {
            return Err("Rust backend: native Process is not admitted in program WASM".into());
        }
        if !evidence.is_empty() {
            return Err("Rust backend: unsupported Process reference evidence".into());
        }
        self.process_type_matches(
            type_ref,
            &CoreType::Function {
                parameter: Box::new(parameter),
                result: Box::new(result),
            },
            scope,
        )?;
        self.capabilities.insert("process".into());
        self.capabilities.insert("effect-sync".into());
        Ok(Some(format!(
            "V::fun(vec![], |_, argument| native_process_operation({}, argument))",
            quoted(name)
        )))
    }

    pub(super) fn process_call(
        &mut self,
        expr: &CoreExpr,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let (name, arguments, result, valid, saturated) = match expr {
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
        let Some((parameter, expected)) = signature(name) else {
            return Ok(None);
        };
        if scope.contains(name) {
            return Ok(None);
        }
        if self.program_wasm_library {
            return Err("Rust backend: native Process is not admitted in program WASM".into());
        }
        if !valid || arguments.len() > 1 || (saturated && arguments.len() != 1) {
            return Err("Rust backend: unproven Process source application".into());
        }
        let Some(argument) = arguments.first() else {
            return self.process_reference(name, &[], &result, scope);
        };
        let input = local::method_argument_type(argument)
            .ok_or("Rust backend: missing Process argument type")?;
        self.process_type_matches(&input, &parameter, scope)?;
        self.process_type_matches(&result, &expected, scope)?;
        self.process_reference(
            name,
            &[],
            &CoreType::Function {
                parameter: Box::new(parameter),
                result: Box::new(expected),
            },
            scope,
        )?;
        Ok(Some(format!(
            "native_process_operation({}, {})",
            quoted(name),
            self.expr(argument, scope)?
        )))
    }
}
