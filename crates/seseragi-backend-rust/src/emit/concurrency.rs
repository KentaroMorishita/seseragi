use super::*;

fn named(name: &str, arguments: Vec<CoreType>) -> CoreType {
    CoreType::Named {
        name: name.into(),
        arguments,
    }
}
fn primitive(name: &str) -> CoreType {
    named(&format!("std/prelude::{name}"), vec![])
}
fn variable(name: &str) -> CoreType {
    named(&format!("\0concurrency_{name}"), vec![])
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
fn signature(name: &str) -> Option<(Vec<CoreType>, CoreType)> {
    let e = variable("E");
    let a = variable("A");
    let row = CoreType::Record {
        closed: true,
        fields: vec![],
    };
    let effect = |r, e, a| named("std/prelude::Effect", vec![r, e, a]);
    let task = |a| effect(row.clone(), primitive("Never"), a);
    let fiber = || named("std/effect::Fiber", vec![e.clone(), a.clone()]);
    let exit = || named("std/effect::FiberExit", vec![e.clone(), a.clone()]);
    let deferred = || named("std/deferred::Deferred", vec![e.clone(), a.clone()]);
    let either = || named("std/prelude::Either", vec![e.clone(), a.clone()]);
    let maybe = |a| named("std/prelude::Maybe", vec![a]);
    Some(match name {
        "std/prelude::Right" => (vec![a.clone()], either()),
        "std/prelude::Left" => (vec![e.clone()], either()),
        "std/effect::FiberSucceeded" => (vec![a.clone()], exit()),
        "std/effect::FiberFailed" => (vec![e.clone()], exit()),
        "std/effect::FiberCancelled" => (vec![], exit()),
        "std/effect::fork" => (
            vec![effect(variable("R"), e.clone(), a.clone())],
            effect(variable("R"), primitive("Never"), fiber()),
        ),
        "std/effect::await" => (vec![fiber()], task(exit())),
        "std/effect::poll" => (vec![fiber()], task(maybe(exit()))),
        "std/effect::join" => (vec![fiber()], effect(row, e.clone(), a.clone())),
        "std/effect::interrupt" => (vec![fiber()], task(primitive("Unit"))),
        "std/effect::yieldNow" => (vec![primitive("Unit")], task(primitive("Unit"))),
        "std/deferred::make" => (vec![primitive("Unit")], task(deferred())),
        "std/deferred::await" => (vec![deferred()], effect(row, e.clone(), a.clone())),
        "std/deferred::poll" => (vec![deferred()], task(maybe(either()))),
        "std/deferred::complete" => (vec![either(), deferred()], task(primitive("Bool"))),
        "std/deferred::succeed" => (vec![a.clone(), deferred()], task(primitive("Bool"))),
        "std/deferred::fail" => (vec![e.clone(), deferred()], task(primitive("Bool"))),
        _ => return None,
    })
}
fn match_contract(
    template: &CoreType,
    actual: &CoreType,
    slots: &mut BTreeMap<String, CoreType>,
) -> bool {
    match (template, actual) {
        (CoreType::Named { name, arguments }, _)
            if arguments.is_empty()
                && matches!(
                    name.as_str(),
                    "\0concurrency_E" | "\0concurrency_A" | "\0concurrency_R"
                ) =>
        {
            if name == "\0concurrency_R" && !matches!(actual, CoreType::Record { closed: true, .. })
            {
                return false;
            }
            if let Some(previous) = slots.get(name) {
                previous == actual
            } else {
                slots.insert(name.clone(), actual.clone());
                true
            }
        }
        (
            CoreType::Named { name, arguments },
            CoreType::Named {
                name: other,
                arguments: values,
            },
        ) => {
            name == other
                && arguments.len() == values.len()
                && arguments
                    .iter()
                    .zip(values)
                    .all(|(a, b)| match_contract(a, b, slots))
        }
        (
            CoreType::Function { parameter, result },
            CoreType::Function {
                parameter: p,
                result: r,
            },
        ) => match_contract(parameter, p, slots) && match_contract(result, r, slots),
        _ => template == actual,
    }
}
impl Backend<'_> {
    pub(super) fn concurrency_reference(
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
        if !evidence.is_empty() || self.declared_method_type_kind(type_ref, scope) != Some(0) {
            return Err("Rust backend: unproven concurrent callable evidence or kinds".into());
        }
        let actual = self.local_canonical_type(type_ref, scope)?;
        if !match_contract(&arrow(&parameters, result), &actual, &mut BTreeMap::new()) {
            return Err("Rust backend: concurrent canonical callable contract mismatch".into());
        }
        let constructor = matches!(
            name,
            "std/prelude::Right"
                | "std/prelude::Left"
                | "std/effect::FiberSucceeded"
                | "std/effect::FiberFailed"
                | "std/effect::FiberCancelled"
        );
        if !constructor {
            self.capabilities.insert("effect-sync".into());
        }
        self.capabilities.insert(
            if name.starts_with("std/prelude::") {
                "either"
            } else if name.starts_with("std/deferred::") {
                "deferred"
            } else {
                "fiber"
            }
            .into(),
        );
        let names: Vec<_> = (0..parameters.len())
            .map(|i| format!("\0concurrent_argument_{i}"))
            .collect();
        let args = names
            .iter()
            .map(|name| format!("{}.clone()", id(name)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = if constructor {
            format!(
                "V::adt({}, {})",
                quoted(name),
                if names.is_empty() {
                    "V::Unit".to_owned()
                } else {
                    format!("{}.clone()", id(&names[0]))
                }
            )
        } else {
            format!("native_operation({}, vec![{args}])", quoted(name))
        };
        for i in (0..names.len()).rev() {
            body = Self::closure(&names[i], &body, &names[..i].iter().cloned().collect());
        }
        Ok(Some(body))
    }
    pub(super) fn concurrency_call(
        &mut self,
        expr: &CoreExpr,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let (name, arguments, result, valid) = match expr {
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
                named(
                    "std/prelude::Effect",
                    vec![requirements.clone(), failure.clone(), success.clone()],
                ),
                evidence.is_empty(),
            ),
            _ => return Ok(None),
        };
        let Some((parameters, expected)) = signature(name) else {
            return Ok(None);
        };
        if scope.contains(name) {
            return Ok(None);
        }
        if !valid || arguments.len() > parameters.len() {
            return Err("Rust backend: unproven concurrent source application".into());
        }
        let mut input = Vec::new();
        for argument in arguments {
            input.push(
                local::method_argument_type(argument)
                    .ok_or("Rust backend: missing concurrent argument type")?,
            );
        }
        let actual = arrow(&input, result);
        // Check kinds before canonical erasure, including every applied slot.
        if self.declared_method_type_kind(&actual, scope) != Some(0) {
            return Err("Rust backend: unproven concurrent argument/result kinds".into());
        }
        let canonical = self.local_canonical_type(&actual, scope)?;
        if !match_contract(
            &arrow(&parameters, expected),
            &canonical,
            &mut BTreeMap::new(),
        ) {
            return Err(
                "Rust backend: concurrent canonical source/result contract mismatch".into(),
            );
        }
        let function = self
            .concurrency_reference(name, &[], &actual, scope)?
            .unwrap();
        let temporary = self.fresh();
        let mut value = format!("{{ let mut {temporary} = {function};");
        for argument in arguments {
            value.push_str(&format!(
                "{temporary} = {temporary}.apply({});",
                self.expr(argument, scope)?
            ));
        }
        value.push_str(&format!("{temporary} }}"));
        Ok(Some(value))
    }
}
