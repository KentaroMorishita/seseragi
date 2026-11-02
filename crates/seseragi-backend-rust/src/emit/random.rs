use super::*;

fn named(name: &str, arguments: Vec<CoreType>) -> CoreType {
    CoreType::Named {
        name: name.into(),
        arguments,
    }
}

fn ty(name: &str) -> CoreType {
    named(name, vec![])
}

fn arity(name: &str) -> Option<usize> {
    Some(match name {
        "std/random::intBetween" => 2,
        "std/random::randomSize"
        | "std/random::algorithmId"
        | "std/random::nextBool"
        | "std/random::nextInt"
        | "std/random::unitFloat"
        | "std/random::chance"
        | "std/random::randomBytes"
        | "std/random::choose"
        | "std/random::shuffle" => 1,
        _ => return None,
    })
}

fn arrow(parameters: &[CoreType], result: CoreType) -> CoreType {
    parameters
        .iter()
        .rev()
        .fold(result, |result, parameter| CoreType::Function {
            parameter: Box::new(parameter.clone()),
            result: Box::new(result),
        })
}

impl Backend<'_> {
    fn random_signature(&self, name: &str, full: &CoreType, scope: &Scope) -> Result<()> {
        if self.program_wasm_library {
            return Err("program WASM: native Random requires a target-specific ABI proof".into());
        }
        if scope.has_unproven_type_scope() || self.declared_method_type_kind(full, scope) != Some(0)
        {
            return Err("Rust backend: unproven Random argument/result kinds".into());
        }
        let full = self.local_canonical_type(full, scope)?;
        let int = || ty("std/prelude::Int");
        let boolean = || ty("std/prelude::Bool");
        let float = || ty("std/prelude::Float");
        let unit = || ty("std/prelude::Unit");
        let never = || ty("std/prelude::Never");
        let effect = |failure, success| {
            named(
                "std/prelude::Effect",
                vec![
                    CoreType::Record {
                        closed: true,
                        fields: vec![seseragi_lowering::CoreRecordField {
                            name: "random".into(),
                            optional: false,
                            type_ref: ty("std/random::Random"),
                        }],
                    },
                    failure,
                    success,
                ],
            )
        };
        let (parameters, result) = match name {
            "std/random::randomSize" => (
                vec![int()],
                named(
                    "std/prelude::Either",
                    vec![
                        ty("std/random::RandomConfigError"),
                        ty("std/random::RandomSize"),
                    ],
                ),
            ),
            "std/random::algorithmId" => (vec![unit()], effect(never(), ty("std/prelude::String"))),
            "std/random::nextBool" => (vec![unit()], effect(never(), boolean())),
            "std/random::nextInt" => (vec![unit()], effect(never(), int())),
            "std/random::unitFloat" => (vec![unit()], effect(never(), float())),
            "std/random::intBetween" => (
                vec![int(), int()],
                effect(ty("std/random::RandomRangeError"), int()),
            ),
            "std/random::chance" => (
                vec![float()],
                effect(ty("std/random::RandomRangeError"), boolean()),
            ),
            "std/random::randomBytes" => (
                vec![ty("std/random::RandomSize")],
                effect(never(), ty("std/bytes::Bytes")),
            ),
            "std/random::choose" | "std/random::shuffle" => {
                let CoreType::Function { parameter, .. } = &full else {
                    return Err("Rust backend: missing Random collection argument".into());
                };
                let CoreType::Named {
                    name: owner,
                    arguments,
                } = parameter.as_ref()
                else {
                    return Err("Rust backend: missing canonical Random collection".into());
                };
                let expected_owner = if name == "std/random::choose" {
                    "std/non-empty-list::NonEmptyList"
                } else {
                    "std/prelude::Array"
                };
                if owner != expected_owner || arguments.len() != 1 {
                    return Err("Rust backend: Random collection owner or arity mismatch".into());
                }
                let success = if name == "std/random::choose" {
                    arguments[0].clone()
                } else {
                    *parameter.clone()
                };
                (vec![*parameter.clone()], effect(never(), success))
            }
            _ => unreachable!(),
        };
        if full != arrow(&parameters, result) {
            return Err("Rust backend: unproven canonical Random signature".into());
        }
        Ok(())
    }

    pub(super) fn random_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let Some(arity) = arity(name) else {
            return Ok(None);
        };
        if scope.contains(name) {
            return Ok(None);
        }
        if !evidence.is_empty() {
            return Err("Rust backend: unsupported Random reference evidence".into());
        }
        self.random_signature(name, type_ref, scope)?;
        if name != "std/random::randomSize" {
            self.capabilities.insert("random".into());
            self.capabilities.insert("effect-sync".into());
        }
        let names: Vec<_> = (0..arity)
            .map(|i| format!("\0random_argument_{i}"))
            .collect();
        let arguments = names
            .iter()
            .map(|name| format!("{}.clone()", id(name)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = format!(
            "native_random_operation({}, vec![{arguments}])",
            quoted(name)
        );
        for i in (0..arity).rev() {
            body = Self::closure(&names[i], &body, &names[..i].iter().cloned().collect());
        }
        Ok(Some(body))
    }

    pub(super) fn random_call(&mut self, expr: &CoreExpr, scope: &Scope) -> Result<Option<String>> {
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
        let Some(arity) = arity(name) else {
            return Ok(None);
        };
        if scope.contains(name) {
            return Ok(None);
        }
        if !valid || arguments.len() > arity || (saturated && arguments.len() != arity) {
            return Err("Rust backend: unproven Random source application".into());
        }
        let supplied = arguments
            .iter()
            .map(|argument| {
                local::method_argument_type(argument)
                    .ok_or_else(|| "Rust backend: missing Random argument type".into())
            })
            .collect::<Result<Vec<_>>>()?;
        let function = self
            .random_reference(name, &[], &arrow(&supplied, result), scope)?
            .unwrap();
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
