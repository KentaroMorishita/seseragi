use super::*;
fn arity(name: &str) -> Option<usize> {
    Some(match name {
        "std/stream::singleton"
        | "std/stream::fromArray"
        | "std/stream::fromEffect"
        | "std/stream::runCollect" => 1,
        "std/stream::unfold"
        | "std/stream::map"
        | "std/stream::filter"
        | "std/stream::filterMap"
        | "std/stream::mapError"
        | "std/stream::flatMap"
        | "std/stream::take"
        | "std/stream::drop"
        | "std/stream::concat"
        | "std/stream::runForEach" => 2,
        "std/stream::runFold" => 3,
        _ => return None,
    })
}
fn named(name: &str, arguments: Vec<CoreType>) -> CoreType {
    CoreType::Named {
        name: name.into(),
        arguments,
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
impl Backend<'_> {
    fn stream_operation_type(
        &self,
        name: &str,
        arguments: &[CoreExpr],
        type_ref: &CoreType,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<usize> {
        if self.program_wasm_library
            || scope.has_unproven_type_scope()
            || !evidence.is_empty()
            || arguments.len() > arity(name).unwrap()
        {
            return Err("Rust backend: unproven native Stream application".into());
        }
        let supplied = arguments
            .iter()
            .map(|a| {
                local::method_argument_type(a)
                    .ok_or_else(|| "Rust backend: missing Stream argument type".into())
            })
            .collect::<Result<Vec<_>>>()?;
        let full = function(&supplied, type_ref.clone());
        if self.declared_method_type_kind(&full, scope) != Some(0) {
            return Err("Rust backend: Stream source/result must have value kind".into());
        }
        let full = self.local_canonical_type(&full, scope)?;
        let var = |name| named(name, vec![]);
        let r = var("\0stream_env");
        let e = var("\0stream_error");
        let f = var("\0stream_next_error");
        let a = var("\0stream_value");
        let b = var("\0stream_result");
        let s = var("\0stream_state");
        let stream = |a| named("std/stream::Stream", vec![r.clone(), e.clone(), a]);
        let effect = |a| named("std/prelude::Effect", vec![r.clone(), e.clone(), a]);
        let array = |a| named("std/prelude::Array", vec![a]);
        let maybe = |a| named("std/prelude::Maybe", vec![a]);
        let (parameters, result) = match name {
            "std/stream::singleton" => (vec![a.clone()], stream(a.clone())),
            "std/stream::fromArray" => (vec![array(a.clone())], stream(a.clone())),
            "std/stream::fromEffect" => (vec![effect(a.clone())], stream(a.clone())),
            "std/stream::unfold" => (
                vec![
                    function(
                        &[s.clone()],
                        maybe(CoreType::Tuple {
                            elements: vec![a.clone(), s.clone()],
                        }),
                    ),
                    s.clone(),
                ],
                stream(a.clone()),
            ),
            "std/stream::map" => (
                vec![function(&[a.clone()], b.clone()), stream(a.clone())],
                stream(b.clone()),
            ),
            "std/stream::filter" => (
                vec![
                    function(&[a.clone()], named("std/prelude::Bool", vec![])),
                    stream(a.clone()),
                ],
                stream(a.clone()),
            ),
            "std/stream::filterMap" => (
                vec![function(&[a.clone()], maybe(b.clone())), stream(a.clone())],
                stream(b.clone()),
            ),
            "std/stream::mapError" => (
                vec![function(&[e.clone()], f.clone()), stream(a.clone())],
                named("std/stream::Stream", vec![r.clone(), f.clone(), a.clone()]),
            ),
            "std/stream::flatMap" => (
                vec![function(&[a.clone()], stream(b.clone())), stream(a.clone())],
                stream(b.clone()),
            ),
            "std/stream::take" | "std/stream::drop" => (
                vec![named("std/prelude::Int", vec![]), stream(a.clone())],
                stream(a.clone()),
            ),
            "std/stream::concat" => (
                vec![stream(a.clone()), stream(a.clone())],
                stream(a.clone()),
            ),
            "std/stream::runCollect" => (vec![stream(a.clone())], effect(array(a.clone()))),
            "std/stream::runFold" => (
                vec![
                    b.clone(),
                    function(&[b.clone(), a.clone()], b.clone()),
                    stream(a.clone()),
                ],
                effect(b.clone()),
            ),
            "std/stream::runForEach" => (
                vec![
                    function(&[a.clone()], effect(named("std/prelude::Unit", vec![]))),
                    stream(a.clone()),
                ],
                effect(named("std/prelude::Unit", vec![])),
            ),
            _ => unreachable!(),
        };
        self.match_local_type(
            &function(&parameters, result),
            &full,
            &BTreeMap::from([
                ("\0stream_env".into(), 0),
                ("\0stream_error".into(), 0),
                ("\0stream_next_error".into(), 0),
                ("\0stream_value".into(), 0),
                ("\0stream_result".into(), 0),
                ("\0stream_state".into(), 0),
            ]),
            &mut BTreeMap::new(),
            scope,
        )?;
        Ok(arity(name).unwrap())
    }

    fn stream_operation_callable(
        &mut self,
        name: &str,
        arity: usize,
        _evidence: &[CoreCallEvidence],
        _scope: &Scope,
    ) -> Result<String> {
        let names: Vec<_> = (0..arity)
            .map(|i| format!("\0stream_argument_{i}"))
            .collect();
        let args = names
            .iter()
            .map(|n| format!("{}.clone()", id(n)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = format!("native_stream_operation({}, vec![{args}])", quoted(name));
        for i in (0..arity).rev() {
            body = Self::closure(&names[i], &body, &names[..i].iter().cloned().collect());
        }
        Ok(body)
    }

    pub(super) fn stream_operation_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if arity(name).is_none() || scope.contains(name) {
            return Ok(None);
        }
        let arity = self.stream_operation_type(name, &[], type_ref, evidence, scope)?;
        self.capabilities
            .extend(["stream".into(), "effect-sync".into()]);
        Ok(Some(
            self.stream_operation_callable(name, arity, evidence, scope)?,
        ))
    }

    pub(super) fn stream_operation_call(
        &mut self,
        expr: &CoreExpr,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if let CoreExpr::EffectInvoke {
            callee,
            arguments,
            requirements,
            failure,
            success,
            evidence,
            ..
        } = expr
        {
            if arity(callee).is_none() || scope.contains(callee) {
                return Ok(None);
            }
            let result = named(
                "std/prelude::Effect",
                vec![requirements.clone(), failure.clone(), success.clone()],
            );
            let arity = self.stream_operation_type(callee, arguments, &result, evidence, scope)?;
            self.capabilities
                .extend(["stream".into(), "effect-sync".into()]);
            let callable = self.stream_operation_callable(callee, arity, evidence, scope)?;
            let value = self.fresh();
            let mut body = format!("{{ let mut {value} = {callable};");
            for argument in arguments {
                body.push_str(&format!(
                    "{value} = {value}.apply({});",
                    self.expr(argument, scope)?
                ));
            }
            body.push_str(&format!("{value} }}"));
            return Ok(Some(body));
        }
        let CoreExpr::Call {
            callee,
            arguments,
            type_ref,
            evidence,
            trait_dispatch,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            ..
        } = expr
        else {
            return Ok(None);
        };
        if arity(callee).is_none() || scope.contains(callee) {
            return Ok(None);
        }
        if trait_dispatch.is_some()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || evidence_argument_index.is_some()
        {
            return Err("Rust backend: unproven native source stages".into());
        }
        let arity = self.stream_operation_type(callee, arguments, type_ref, evidence, scope)?;
        // Retained argument types must describe the remaining curried parameters.
        if !deferred_evidence_parameters.is_empty() {
            if arguments.len() + deferred_evidence_parameters.len() != arity {
                return Err("Rust backend: collection deferred source arity mismatch".into());
            }
            let mut remaining = type_ref;
            for deferred in deferred_evidence_parameters {
                let CoreType::Function { parameter, result } = remaining else {
                    return Err("Rust backend: missing collection deferred source stage".into());
                };
                if self.local_canonical_type(parameter, scope)?
                    != self.local_canonical_type(deferred, scope)?
                {
                    return Err("Rust backend: collection deferred source type mismatch".into());
                }
                remaining = result;
            }
        }
        self.capabilities
            .extend(["stream".into(), "effect-sync".into()]);
        let value = self.fresh();
        let callable = self.stream_operation_callable(callee, arity, evidence, scope)?;
        let mut body = format!("{{ let mut {value}={callable};");
        for argument in arguments {
            body.push_str(&format!(
                "{value}={value}.apply({});",
                self.expr(argument, scope)?
            ));
        }
        body.push_str(&format!("{value} }}"));
        Ok(Some(body))
    }
}
