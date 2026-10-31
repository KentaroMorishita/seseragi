use super::*;

fn arity(name: &str) -> Option<usize> {
    Some(match name {
        "std/signal::make"
        | "std/signal::read"
        | "std/signal::transaction"
        | "std/signal::constant"
        | "std/signal::distinct"
        | "std/signal::unsubscribe" => 1,
        "std/signal::set"
        | "std/signal::update"
        | "std/signal::planSet"
        | "std/signal::planUpdate"
        | "std/signal::map"
        | "std/signal::switchMap"
        | "std/signal::subscribe" => 2,
        "std/signal::combine" => 3,
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
fn element(ty: &CoreType, owner: &str) -> Result<CoreType> {
    match ty {
        CoreType::Named { name, arguments } if name == owner && arguments.len() == 1 => {
            Ok(arguments[0].clone())
        }
        _ => Err("Rust backend: unproven canonical Signal owner or arity".into()),
    }
}
fn effect(ty: &CoreType) -> Result<(&CoreType, &CoreType)> {
    match ty {
        CoreType::Named { name, arguments }
            if name == "std/prelude::Effect"
                && arguments.len() == 3
                && arguments[1] == named("std/prelude::Never", vec![]) =>
        {
            Ok((&arguments[0], &arguments[2]))
        }
        _ => Err("Rust backend: Signal operation requires an infallible Effect".into()),
    }
}
impl Backend<'_> {
    fn signal_type(
        &self,
        name: &str,
        arguments: &[CoreExpr],
        result: &CoreType,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<usize> {
        if self.program_wasm_library || scope.has_unproven_type_scope() {
            return Err("Rust backend: native Signal requires a proven native type scope".into());
        }
        let arity = arity(name).unwrap();
        if arguments.len() > arity {
            return Err("Rust backend: Signal source arity mismatch".into());
        }
        let supplied = arguments
            .iter()
            .map(|arg| {
                local::method_argument_type(arg)
                    .ok_or_else(|| "Rust backend: missing Signal source type".into())
            })
            .collect::<Result<Vec<_>>>()?;
        let full = function(&supplied, result.clone());
        if self.declared_method_type_kind(&full, scope) != Some(0) {
            return Err("Rust backend: Signal requires value-kind arguments/result".into());
        }
        let full = self.local_canonical_type(&full, scope)?;
        let mut remaining = &full;
        let mut parameters = Vec::new();
        for _ in 0..arity {
            let CoreType::Function { parameter, result } = remaining else {
                return Err("Rust backend: Signal source arity mismatch".into());
            };
            parameters.push(parameter.as_ref().clone());
            remaining = result;
        }
        let signal = |a| named("std/signal::Signal", vec![a]);
        let mutable = |a| named("std/signal::MutableSignal", vec![a]);
        let unit = || named("std/prelude::Unit", vec![]);
        let change = || named("std/signal::SignalChange", vec![]);
        let subscription = || named("std/signal::Subscription", vec![]);
        let task = |a| {
            named(
                "std/prelude::Effect",
                vec![
                    CoreType::Record {
                        closed: true,
                        fields: vec![],
                    },
                    named("std/prelude::Never", vec![]),
                    a,
                ],
            )
        };
        let readable = |ty: &CoreType| {
            element(ty, "std/signal::Signal").or_else(|_| element(ty, "std/signal::MutableSignal"))
        };
        let expected = match name {
            "std/signal::make" => function(&parameters, task(mutable(parameters[0].clone()))),
            "std/signal::read" => function(&parameters, task(readable(&parameters[0])?)),
            "std/signal::set"
            | "std/signal::update"
            | "std/signal::planSet"
            | "std/signal::planUpdate" => {
                let a = element(&parameters[1], "std/signal::MutableSignal")?;
                let value = if name.ends_with("Update") || name.ends_with("::update") {
                    function(std::slice::from_ref(&a), a.clone())
                } else {
                    a.clone()
                };
                function(
                    &[value, mutable(a)],
                    if name.contains("::plan") {
                        change()
                    } else {
                        task(unit())
                    },
                )
            }
            "std/signal::transaction" => {
                function(&[named("std/prelude::Array", vec![change()])], task(unit()))
            }
            "std/signal::constant" => function(&parameters, signal(parameters[0].clone())),
            "std/signal::map" | "std/signal::switchMap" | "std/signal::combine" => {
                let a = readable(&parameters[1])?;
                let b = element(remaining, "std/signal::Signal")?;
                let mapper = if name.ends_with("::combine") {
                    function(&[a, readable(&parameters[2])?], b)
                } else {
                    function(
                        &[a],
                        if name.ends_with("::switchMap") {
                            let CoreType::Function { result, .. } = &parameters[0] else {
                                return Err("Rust backend: switchMap requires a function".into());
                            };
                            if readable(result)? != b {
                                return Err("Rust backend: switchMap inner element mismatch".into());
                            }
                            result.as_ref().clone()
                        } else {
                            b
                        },
                    )
                };
                let mut expected = parameters.clone();
                expected[0] = mapper;
                function(&expected, remaining.clone())
            }
            "std/signal::distinct" => {
                let a = readable(&parameters[0])?;
                let [eq] = evidence else {
                    return Err("Rust backend: distinct requires one Eq dictionary".into());
                };
                self.validate_eq_evidence(eq, scope)?;
                if self.local_canonical_type(&eq.constraint.arguments[0], scope)? != a {
                    return Err("Rust backend: distinct Eq element mismatch".into());
                }
                function(&parameters, signal(a))
            }
            "std/signal::subscribe" => {
                let a = readable(&parameters[1])?;
                let (environment, value) = effect(remaining)?;
                if *value != subscription() {
                    return Err("Rust backend: Signal Subscription result mismatch".into());
                }
                function(
                    &[
                        function(
                            &[a],
                            named(
                                "std/prelude::Effect",
                                vec![
                                    environment.clone(),
                                    named("std/prelude::Never", vec![]),
                                    unit(),
                                ],
                            ),
                        ),
                        parameters[1].clone(),
                    ],
                    remaining.clone(),
                )
            }
            "std/signal::unsubscribe" => function(&[subscription()], task(unit())),
            _ => unreachable!(),
        };
        if name != "std/signal::distinct" && !evidence.is_empty() {
            return Err("Rust backend: Signal unexpected evidence".into());
        }
        if expected != full {
            return Err("Rust backend: Signal argument/result type mismatch".into());
        }
        Ok(arity)
    }
    fn signal_callable(
        &mut self,
        name: &str,
        arity: usize,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<String> {
        let names = (0..arity)
            .map(|i| format!("\0signal_argument_{i}"))
            .collect::<Vec<_>>();
        let mut args = names
            .iter()
            .map(|n| format!("{}.clone()", id(n)))
            .collect::<Vec<_>>();
        let dictionaries = self.evidence(evidence, scope)?;
        if name == "std/signal::distinct" {
            args.insert(0, format!("{}.clone().field(\"eq\")", id("\0signal_eq")));
        }
        let mut body = format!(
            "native_signal_operation({}, vec![{}])",
            quoted(name),
            args.join(",")
        );
        let captures = if name == "std/signal::distinct" {
            vec!["\0signal_eq".to_owned()]
        } else {
            vec![]
        };
        for i in (0..arity).rev() {
            let scope = names[..i].iter().chain(&captures).cloned().collect();
            body = Self::closure(&names[i], &body, &scope);
        }
        if !captures.is_empty() {
            body = format!(
                "{{ let {} = ({dictionaries}).remove(0); {body} }}",
                id(&captures[0])
            );
        }
        Ok(body)
    }
    pub(super) fn signal_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if arity(name).is_none() || scope.contains(name) {
            return Ok(None);
        }
        let arity = self.signal_type(name, &[], type_ref, evidence, scope)?;
        self.capabilities
            .extend(["signal".into(), "ref".into(), "effect-sync".into()]);
        Ok(Some(self.signal_callable(name, arity, evidence, scope)?))
    }
    pub(super) fn signal_call(&mut self, expr: &CoreExpr, scope: &Scope) -> Result<Option<String>> {
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
            || !deferred_evidence_parameters.is_empty()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || evidence_argument_index.is_some()
        {
            return Err("Rust backend: unproven Signal source staging".into());
        }
        let arity = self.signal_type(callee, arguments, type_ref, evidence, scope)?;
        self.capabilities
            .extend(["signal".into(), "ref".into(), "effect-sync".into()]);
        let value = self.fresh();
        let callable = self.signal_callable(callee, arity, evidence, scope)?;
        let mut output = format!("{{ let mut {value} = {callable};");
        for argument in arguments {
            output.push_str(&format!(
                "{value} = {value}.apply({});",
                self.expr(argument, scope)?
            ));
        }
        output.push_str(&format!("{value} }}"));
        Ok(Some(output))
    }
    pub(super) fn validate_signal_method(
        &self,
        dispatch: &seseragi_lowering::CoreTraitDispatch,
        selected: &CoreCallEvidence,
        full_type: &CoreType,
        scope: &Scope,
    ) -> Result<()> {
        let CoreInstanceEvidence::Standard { identity, .. } = &selected.evidence else {
            return Ok(());
        };
        let expected = match identity.as_str() {
            "std/signal::Functor" => ("std/prelude::Functor", &["map"][..]),
            "std/signal::Applicative" => ("std/prelude::Applicative", &["pure", "apply"][..]),
            _ => return Ok(()),
        };
        if dispatch.trait_identity != expected.0 || !expected.1.contains(&dispatch.method.as_str())
        {
            return Err("Rust backend: Signal dictionary dispatch mismatch".into());
        }
        self.validate_hkt_parameter_method_value(dispatch, selected, full_type, scope)
    }
    pub(super) fn validate_signal_method_call(&self, expr: &CoreExpr, scope: &Scope) -> Result<()> {
        let CoreExpr::Call {
            trait_dispatch: Some(dispatch),
            arguments,
            type_ref,
            evidence,
            ..
        } = expr
        else {
            return Ok(());
        };
        let Some(selected) = evidence.first() else {
            return Ok(());
        };
        if !matches!(&selected.evidence, CoreInstanceEvidence::Standard { identity, .. } if matches!(identity.as_str(), "std/signal::Functor" | "std/signal::Applicative"))
        {
            return Ok(());
        }
        let parameters = arguments
            .iter()
            .map(|argument| {
                local::method_argument_type(argument)
                    .ok_or_else(|| "Rust backend: missing Signal method argument type".into())
            })
            .collect::<Result<Vec<_>>>()?;
        self.validate_signal_method(
            dispatch,
            selected,
            &function(&parameters, type_ref.clone()),
            scope,
        )
    }
    pub(super) fn signal_dictionary(
        &mut self,
        selected: &CoreCallEvidence,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let CoreInstanceEvidence::Standard {
            identity,
            type_arguments,
            evidence_arguments,
        } = &selected.evidence
        else {
            return Ok(None);
        };
        let method = match identity.as_str() {
            "std/signal::Functor" => "Functor",
            "std/signal::Applicative" => "Applicative",
            _ => return Ok(None),
        };
        if self.program_wasm_library
            || !type_arguments.is_empty()
            || !evidence_arguments.is_empty()
            || selected.constraint.name != method
            || selected.constraint.trait_identity.as_deref()
                != Some(&format!("std/prelude::{method}"))
            || selected.constraint.arguments.len() != 1
            || self.local_canonical_type(&selected.constraint.arguments[0], scope)?
                != named("std/signal::Signal", vec![])
        {
            return Err("Rust backend: unproven Signal dictionary".into());
        }
        self.capabilities.extend(["signal".into(), "ref".into()]);
        Ok(Some("native_signal_dictionary()".into()))
    }
}
