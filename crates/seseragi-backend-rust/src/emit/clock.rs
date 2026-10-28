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
    let duration = || value("std/time::Duration");
    let error = || value("std/time::DurationError");
    Some(match name {
        "std/time::milliseconds" => (
            vec![value("std/prelude::Int")],
            named("std/prelude::Either", vec![error(), duration()]),
        ),
        "std/time::NegativeDuration" => (vec![value("std/prelude::Int")], error()),
        "std/time::DurationOutsideRange" => (vec![], error()),
        "std/clock::sleep" => (
            vec![duration()],
            named(
                "std/prelude::Effect",
                vec![
                    CoreType::Record {
                        closed: true,
                        fields: vec![seseragi_lowering::CoreRecordField {
                            name: "clock".into(),
                            optional: false,
                            type_ref: value("std/clock::Clock"),
                        }],
                    },
                    value("std/prelude::Never"),
                    value("std/prelude::Unit"),
                ],
            ),
        ),
        _ => return None,
    })
}

impl Backend<'_> {
    fn clock_type_matches(
        &self,
        actual: &CoreType,
        expected: &CoreType,
        scope: &Scope,
    ) -> Result<()> {
        if self.declared_method_type_kind(actual, scope) != Some(0)
            || self.local_canonical_type(actual, scope)? != *expected
        {
            return Err(
                "Rust backend: unproven canonical Clock/Duration argument or result".into(),
            );
        }
        Ok(())
    }
    fn require_clock(&mut self, name: &str, origin: &seseragi_lowering::SourceSpan) -> Result<()> {
        if self.program_wasm_library {
            return Err(
                "Rust backend: native Clock/Duration is not admitted in program WASM".into(),
            );
        }
        self.capabilities.insert("duration".into());
        if name == "std/clock::sleep" {
            // Contract types and this finite selection are proved before any
            // generated output or host wait is allowed to exist.
            crate::clock_provider::receipt(self.options, vec![origin.clone()])?;
            self.capabilities.insert("clock-sleep".into());
            self.capabilities.insert("effect-sync".into());
            if !self.clock_origins.contains(origin) {
                self.clock_origins.push(origin.clone());
            }
        }
        Ok(())
    }
    fn clock_callable(name: &str, arity: usize) -> String {
        let names = (0..arity)
            .map(|i| format!("\0clock_argument_{i}"))
            .collect::<Vec<_>>();
        let args = names
            .iter()
            .map(|name| format!("{}.clone()", id(name)))
            .collect::<Vec<_>>()
            .join(",");
        let mut result = format!("native_clock_operation({}, vec![{args}])", quoted(name));
        for i in (0..arity).rev() {
            result = Self::closure(&names[i], &result, &names[..i].iter().cloned().collect());
        }
        result
    }
    pub(super) fn clock_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        ty: &CoreType,
        origin: &seseragi_lowering::SourceSpan,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if scope.contains(name) {
            return Ok(None);
        }
        if name == "std/clock::now" {
            return Err("Rust backend: selected native Clock slice supports sleep only; now is not admitted".into());
        }
        let Some((parameters, result)) = signature(name) else {
            return Ok(None);
        };
        if !evidence.is_empty() {
            return Err("Rust backend: Clock/Duration has no dictionary evidence".into());
        }
        self.clock_type_matches(ty, &arrow(&parameters, &result), scope)?;
        self.require_clock(name, origin)?;
        Ok(Some(Self::clock_callable(name, parameters.len())))
    }
    pub(super) fn clock_call(&mut self, expr: &CoreExpr, scope: &Scope) -> Result<Option<String>> {
        let (name, arguments, result, valid, checked_signature) = match expr {
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
                None,
            ),
            _ => return Ok(None),
        };
        if scope.contains(name) {
            return Ok(None);
        }
        if name == "std/clock::now" {
            return Err("Rust backend: selected native Clock slice supports sleep only; now is not admitted".into());
        }
        let Some((parameters, expected)) = signature(name) else {
            return Ok(None);
        };
        if !valid || arguments.len() > parameters.len() {
            return Err("Rust backend: unproven Clock source arity or evidence".into());
        }
        if let Some(signature) = checked_signature {
            if !signature.type_parameters.is_empty()
                || !signature.instantiated_type_arguments.is_empty()
                || signature.parameters.len() != parameters.len()
            {
                return Err("Rust backend: unproven Clock declaration source arity".into());
            }
            for (actual, expected) in signature.parameters.iter().zip(&parameters) {
                self.clock_type_matches(actual, expected, scope)?;
            }
            self.clock_type_matches(&signature.result, &expected, scope)?;
            if !signature.argument_types.is_empty() {
                if signature.argument_types.len() != arguments.len() {
                    return Err("Rust backend: unproven Clock actual argument arity".into());
                }
                for (actual, expected) in signature.argument_types.iter().zip(&parameters) {
                    self.clock_type_matches(actual, expected, scope)?;
                }
            }
        }
        for (argument, expected) in arguments.iter().zip(&parameters) {
            let actual = local::method_argument_type(argument)
                .ok_or("Rust backend: missing Clock argument type")?;
            self.clock_type_matches(&actual, expected, scope)?;
        }
        self.clock_type_matches(
            &result,
            &arrow(&parameters[arguments.len()..], &expected),
            scope,
        )?;
        self.require_clock(name, core_expr_origin(expr))?;
        let temporary = self.fresh();
        let mut output = format!(
            "{{ let mut {temporary} = {};",
            Self::clock_callable(name, parameters.len())
        );
        for argument in arguments {
            output.push_str(&format!(
                "{temporary} = {temporary}.apply({});",
                self.expr(argument, scope)?
            ));
        }
        output.push_str(&format!("{temporary} }}"));
        Ok(Some(output))
    }
}
