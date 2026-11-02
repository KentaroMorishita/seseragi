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
fn arrow(parameters: &[CoreType], result: CoreType) -> CoreType {
    parameters
        .iter()
        .rev()
        .fold(result, |result, parameter| CoreType::Function {
            parameter: Box::new(parameter.clone()),
            result: Box::new(result),
        })
}
fn record(fields: &[(&str, CoreType)]) -> CoreType {
    CoreType::Record {
        closed: true,
        fields: fields
            .iter()
            .map(|(name, ty)| seseragi_lowering::CoreRecordField {
                name: (*name).into(),
                optional: false,
                type_ref: ty.clone(),
            })
            .collect(),
    }
}
fn effect(environment: CoreType, error: CoreType, success: CoreType) -> CoreType {
    named("std/prelude::Effect", vec![environment, error, success])
}
fn signature(name: &str) -> Option<(Vec<CoreType>, CoreType, Vec<(&'static str, &'static str)>)> {
    let benchmark = value("std/benchmark::Benchmark");
    let failure = value("std/benchmark::BenchmarkFailure");
    let string = value("std/prelude::String");
    let unit = value("std/prelude::Unit");
    Some(match name {
        "std/benchmark::benchmark" => (
            vec![
                string,
                effect(
                    record(&[
                        ("console", value("std/prelude::Console")),
                        ("logger", value("std/log::Logger")),
                        ("random", value("std/random::Random")),
                    ]),
                    failure,
                    unit,
                ),
            ],
            benchmark,
            vec![],
        ),
        "std/benchmark::suite" => (
            vec![string, named("std/prelude::Array", vec![benchmark.clone()])],
            benchmark,
            vec![],
        ),
        "std/benchmark::inputSize" => (
            vec![value("std/prelude::Int"), benchmark.clone()],
            benchmark,
            vec![],
        ),
        "std/benchmark::blackBox" => {
            let a = value("\0benchmark_value");
            (vec![a.clone()], a, vec![])
        }
        "std/benchmark::fail" => (vec![string], effect(record(&[]), failure, unit), vec![]),
        _ => return None,
    })
}

impl Backend<'_> {
    fn benchmark_type(
        &self,
        name: &str,
        arguments: &[CoreExpr],
        result: &CoreType,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<usize> {
        if self.program_wasm_library || scope.has_unproven_type_scope() {
            return Err("Rust backend: std/benchmark requires a proven native value scope".into());
        }
        let (parameters, expected, _) = signature(name).unwrap();
        if arguments.len() > parameters.len() || !evidence.is_empty() {
            return Err("Rust backend: std/benchmark source or dictionary arity mismatch".into());
        }
        let supplied = arguments
            .iter()
            .map(|argument| {
                local::method_argument_type(argument)
                    .ok_or_else(|| "Rust backend: missing std/benchmark argument type".into())
            })
            .collect::<Result<Vec<_>>>()?;
        let full = arrow(&supplied, result.clone());
        if self.declared_method_type_kind(&full, scope) != Some(0) {
            return Err(
                "Rust backend: std/benchmark requires value-kind arguments and result".into(),
            );
        }
        let mut full = self.local_canonical_type(&full, scope)?;
        if name == "std/benchmark::benchmark" {
            if let CoreType::Function { result, .. } = &mut full {
                if let CoreType::Function { parameter, .. } = result.as_mut() {
                    self.benchmark_body_type(parameter, &parameters[1])?;
                    *parameter = Box::new(parameters[1].clone());
                }
            }
        }
        let mut bindings = BTreeMap::new();
        self.match_local_type(
            &arrow(&parameters, expected),
            &full,
            &BTreeMap::from([("\0benchmark_value".into(), 0)]),
            &mut bindings,
            scope,
        )?;
        Ok(parameters.len())
    }

    fn benchmark_body_type(&self, actual: &CoreType, expected: &CoreType) -> Result<()> {
        let CoreType::Named { name, arguments } = actual else {
            return Err("Rust backend: benchmark body requires canonical Effect".into());
        };
        let CoreType::Named {
            arguments: expected,
            ..
        } = expected
        else {
            unreachable!()
        };
        if name != "std/prelude::Effect" || arguments.len() != 3 {
            return Err("Rust backend: benchmark body Effect owner or arity mismatch".into());
        }
        let (
            CoreType::Record {
                closed: true,
                fields,
            },
            CoreType::Record {
                fields: provided, ..
            },
        ) = (&arguments[0], &expected[0])
        else {
            return Err(
                "Rust backend: benchmark body requires proven BenchmarkEnvironment services".into(),
            );
        };
        if fields.iter().any(|field| !provided.contains(field))
            || (arguments[1] != expected[1] && arguments[1] != value("std/prelude::Never"))
            || (arguments[2] != expected[2] && arguments[2] != value("std/prelude::Never"))
        {
            return Err(
                "Rust backend: benchmark body environment, failure or success mismatch".into(),
            );
        }
        Ok(())
    }

    fn benchmark_callable(
        &mut self,
        name: &str,
        arity: usize,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<String> {
        let dictionaries = "\0benchmark_dictionaries".to_owned();
        let names = (0..arity)
            .map(|i| format!("\0benchmark_argument_{i}"))
            .collect::<Vec<_>>();
        let args = names
            .iter()
            .map(|name| format!("{}.clone()", id(name)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = format!(
            "native_benchmark_operation({}, native_map_dictionaries(&{}), vec![{args}])",
            quoted(name),
            id(&dictionaries)
        );
        for i in (0..arity).rev() {
            let mut captures: Scope = names[..i].iter().cloned().collect();
            captures.insert(dictionaries.clone());
            body = Self::closure(&names[i], &body, &captures);
        }
        Ok(format!(
            "{{ let {} = V::Tuple(Shared::new({})); {body} }}",
            id(&dictionaries),
            self.evidence(evidence, scope)?
        ))
    }

    pub(super) fn benchmark_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        ty: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if signature(name).is_none() || scope.contains(name) {
            return Ok(None);
        }
        let arity = self.benchmark_type(name, &[], ty, evidence, scope)?;
        self.capabilities.insert("benchmark".into());
        Ok(Some(self.benchmark_callable(name, arity, evidence, scope)?))
    }

    pub(super) fn benchmark_call(
        &mut self,
        expr: &CoreExpr,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let (name, arguments, result, evidence, saturated, checked) = match expr {
            CoreExpr::Call {
                callee,
                arguments,
                type_ref,
                evidence,
                callee_signature,
                trait_dispatch,
                deferred_evidence_parameters,
                deferred_evidence_type_constructor_parameters,
                evidence_argument_index,
                ..
            } => {
                if signature(callee).is_none() || scope.contains(callee) {
                    return Ok(None);
                }
                if trait_dispatch.is_some()
                    || !deferred_evidence_type_constructor_parameters.is_empty()
                    || evidence_argument_index.is_some()
                {
                    return Err("Rust backend: unproven std/benchmark source staging".into());
                }
                if !deferred_evidence_parameters.is_empty() {
                    if arguments.len() + deferred_evidence_parameters.len()
                        != signature(callee).unwrap().0.len()
                    {
                        return Err(
                            "Rust backend: std/benchmark deferred source arity mismatch".into()
                        );
                    }
                    let mut rest = type_ref;
                    for deferred in deferred_evidence_parameters {
                        let CoreType::Function { parameter, result } = rest else {
                            return Err("Rust backend: missing std/benchmark deferred stage".into());
                        };
                        if self.local_canonical_type(parameter, scope)?
                            != self.local_canonical_type(deferred, scope)?
                        {
                            return Err(
                                "Rust backend: std/benchmark deferred source type mismatch".into(),
                            );
                        }
                        rest = result;
                    }
                }
                (
                    callee,
                    arguments,
                    type_ref.clone(),
                    evidence.as_slice(),
                    false,
                    callee_signature.as_ref(),
                )
            }
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
                effect(requirements.clone(), failure.clone(), success.clone()),
                evidence.as_slice(),
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
                effect(requirements.clone(), failure.clone(), success.clone()),
                &[][..],
                true,
                None,
            ),
            _ => return Ok(None),
        };
        if signature(name).is_none() || scope.contains(name) {
            return Ok(None);
        }
        let arity = self.benchmark_type(name, arguments, &result, evidence, scope)?;
        if saturated && arguments.len() != arity {
            return Err("Rust backend: std/benchmark Effect source arity mismatch".into());
        }
        if let Some(checked) = checked {
            if checked.parameters.len() != arity {
                return Err("Rust backend: std/benchmark declaration arity mismatch".into());
            }
            if name == "std/benchmark::blackBox" {
                let [parameter] = checked.type_parameters.as_slice() else {
                    return Err("Rust backend: blackBox requires one value type parameter".into());
                };
                let variable = value(&parameter.name);
                if parameter.arity != 0
                    || checked.parameters != [variable.clone()]
                    || checked.result != variable
                {
                    return Err(
                        "Rust backend: blackBox declaration must preserve its value type".into(),
                    );
                }
                let [instantiated] = checked.instantiated_type_arguments.as_slice() else {
                    return Err("Rust backend: blackBox requires one proven type argument".into());
                };
                if self.declared_method_type_kind(instantiated, scope) != Some(0) {
                    return Err("Rust backend: blackBox type argument must have value kind".into());
                }
                let supplied = arguments
                    .iter()
                    .map(|argument| local::method_argument_type(argument).unwrap())
                    .collect::<Vec<_>>();
                let actual = self.local_canonical_type(&arrow(&supplied, result.clone()), scope)?;
                let instantiated = self.local_canonical_type(instantiated, scope)?;
                if actual != arrow(&[instantiated.clone()], instantiated) {
                    return Err("Rust backend: blackBox instantiated source type mismatch".into());
                }
            } else {
                if !checked.type_parameters.is_empty()
                    || !checked.instantiated_type_arguments.is_empty()
                {
                    return Err("Rust backend: unexpected std/benchmark type parameters".into());
                }
                let full = arrow(&checked.parameters, checked.result.clone());
                self.benchmark_type(name, &[], &full, evidence, scope)?;
            }
            if !checked.argument_types.is_empty() {
                if checked.argument_types.len() != arguments.len() {
                    return Err(
                        "Rust backend: std/benchmark checked argument arity mismatch".into(),
                    );
                }
                for (checked, argument) in checked.argument_types.iter().zip(arguments) {
                    let actual = local::method_argument_type(argument).unwrap();
                    if self.local_canonical_type(checked, scope)?
                        != self.local_canonical_type(&actual, scope)?
                    {
                        return Err(
                            "Rust backend: std/benchmark checked argument type mismatch".into()
                        );
                    }
                }
            }
        }
        self.capabilities.insert("benchmark".into());
        if arguments.len() == arity {
            return Ok(Some(format!(
                "native_benchmark_operation({}, {}, vec![{}])",
                quoted(name),
                self.evidence(evidence, scope)?,
                self.expressions(arguments, scope)?
            )));
        }
        let callable = self.benchmark_callable(name, arity, evidence, scope)?;
        let value = self.fresh();
        let mut body = format!("{{ let mut {value} = {callable};");
        for argument in arguments {
            body.push_str(&format!(
                "{value} = {value}.apply({});",
                self.expr(argument, scope)?
            ));
        }
        body.push_str(&format!("{value} }}"));
        Ok(Some(body))
    }
}
