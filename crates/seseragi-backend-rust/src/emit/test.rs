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
    let test = value("std/test::Test");
    let failure = value("std/test::TestFailure");
    let string = value("std/prelude::String");
    let bool_ = value("std/prelude::Bool");
    let unit = value("std/prelude::Unit");
    let assertion = effect(record(&[]), failure.clone(), unit.clone());
    let a = value("\0test_value");
    Some(match name {
        "std/test::test" => (
            vec![
                string,
                effect(
                    record(&[
                        ("clock", value("std/clock::Clock")),
                        ("console", value("std/prelude::Console")),
                        ("logger", value("std/log::Logger")),
                        ("random", value("std/random::Random")),
                    ]),
                    failure,
                    unit,
                ),
            ],
            test,
            vec![],
        ),
        "std/test::suite" => (
            vec![string, named("std/prelude::Array", vec![test.clone()])],
            test,
            vec![],
        ),
        "std/test::skip" => (vec![string, test.clone()], test, vec![]),
        "std/test::timeout" => (
            vec![value("std/time::Duration"), test.clone()],
            test,
            vec![],
        ),
        "std/test::equal" | "std/test::notEqual" => (
            vec![a.clone(), a],
            assertion,
            vec![("Eq", "\0test_value"), ("Debug", "\0test_value")],
        ),
        "std/test::isTrue" | "std/test::isFalse" => (vec![bool_], assertion, vec![]),
        "std/test::fail" => (vec![string], assertion, vec![]),
        "std/test::expectFailure" => {
            let r = value("\0test_environment");
            let e = value("\0test_error");
            (
                vec![
                    arrow(&[e.clone()], bool_),
                    effect(value("\0test_source_environment"), e, a),
                ],
                effect(r, failure, unit),
                vec![("Debug", "\0test_error")],
            )
        }
        "std/test::ExpectedTypedFailure" => (vec![], failure, vec![]),
        "std/test::ExplicitTestFailure" | "std/test::TypedFailureDidNotMatch" => {
            (vec![string], failure, vec![])
        }
        "std/test::AssertionFailed" => (
            vec![record(&[
                ("actual", named("std/prelude::Maybe", vec![string.clone()])),
                (
                    "expected",
                    named("std/prelude::Maybe", vec![string.clone()]),
                ),
                ("message", string),
            ])],
            failure,
            vec![],
        ),
        _ => return None,
    })
}

impl Backend<'_> {
    fn test_type(
        &self,
        name: &str,
        arguments: &[CoreExpr],
        result: &CoreType,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<usize> {
        if self.program_wasm_library || scope.has_unproven_type_scope() {
            return Err("Rust backend: std/test requires a proven native value scope".into());
        }
        let (parameters, expected, dictionaries) = signature(name).unwrap();
        if arguments.len() > parameters.len() || evidence.len() != dictionaries.len() {
            return Err("Rust backend: std/test source or dictionary arity mismatch".into());
        }
        let supplied = arguments
            .iter()
            .map(|argument| {
                local::method_argument_type(argument)
                    .ok_or_else(|| "Rust backend: missing std/test argument type".into())
            })
            .collect::<Result<Vec<_>>>()?;
        let full = arrow(&supplied, result.clone());
        if self.declared_method_type_kind(&full, scope) != Some(0) {
            return Err("Rust backend: std/test requires value-kind arguments and result".into());
        }
        let mut full = self.local_canonical_type(&full, scope)?;
        if name == "std/test::test" {
            if let CoreType::Function { result, .. } = &mut full {
                if let CoreType::Function { parameter, .. } = result.as_mut() {
                    self.test_body_type(parameter, &parameters[1])?;
                    *parameter = Box::new(parameters[1].clone());
                }
            }
        }
        let mut bindings = BTreeMap::new();
        self.match_local_type(
            &arrow(&parameters, expected),
            &full,
            &BTreeMap::from([
                ("\0test_value".into(), 0),
                ("\0test_environment".into(), 0),
                ("\0test_source_environment".into(), 0),
                ("\0test_error".into(), 0),
            ]),
            &mut bindings,
            scope,
        )?;
        if name == "std/test::expectFailure" {
            self.test_environment_provides(
                &bindings["\0test_environment"],
                &bindings["\0test_source_environment"],
            )?;
        }
        for (selected, (class, variable)) in evidence.iter().zip(dictionaries) {
            if class == "Eq" {
                self.validate_eq_evidence(selected, scope)?;
            } else {
                self.validate_display_child(selected, class, scope)?;
            }
            let [actual] = selected.constraint.arguments.as_slice() else {
                return Err("Rust backend: std/test dictionary argument arity mismatch".into());
            };
            if bindings.get(variable) != Some(&self.local_canonical_type(actual, scope)?) {
                return Err("Rust backend: std/test dictionary value type mismatch".into());
            }
        }
        Ok(parameters.len())
    }

    fn test_environment_provides(&self, provided: &CoreType, required: &CoreType) -> Result<()> {
        if provided == required {
            return Ok(());
        }
        if let (
            CoreType::Record {
                closed: true,
                fields: provided,
            },
            CoreType::Record {
                closed: true,
                fields: required,
            },
        ) = (provided, required)
        {
            if required.iter().all(|field| provided.contains(field)) {
                return Ok(());
            }
        }
        Err("Rust backend: std/test result does not provide the source environment".into())
    }

    fn test_body_type(&self, actual: &CoreType, expected: &CoreType) -> Result<()> {
        let CoreType::Named { name, arguments } = actual else {
            return Err("Rust backend: test body requires canonical Effect".into());
        };
        let CoreType::Named {
            arguments: expected,
            ..
        } = expected
        else {
            unreachable!()
        };
        if name != "std/prelude::Effect" || arguments.len() != 3 {
            return Err("Rust backend: test body Effect owner or arity mismatch".into());
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
            return Err("Rust backend: test body requires proven TestEnvironment services".into());
        };
        if fields.iter().any(|field| !provided.contains(field))
            || (arguments[1] != expected[1] && arguments[1] != value("std/prelude::Never"))
            || (arguments[2] != expected[2] && arguments[2] != value("std/prelude::Never"))
        {
            return Err("Rust backend: test body environment, failure or success mismatch".into());
        }
        Ok(())
    }

    fn test_callable(
        &mut self,
        name: &str,
        arity: usize,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<String> {
        let dictionaries = "\0test_dictionaries".to_owned();
        let names = (0..arity)
            .map(|i| format!("\0test_argument_{i}"))
            .collect::<Vec<_>>();
        let args = names
            .iter()
            .map(|name| format!("{}.clone()", id(name)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = format!(
            "native_test_operation({}, native_map_dictionaries(&{}), vec![{args}])",
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

    pub(super) fn test_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        ty: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if signature(name).is_none() || scope.contains(name) {
            return Ok(None);
        }
        let arity = self.test_type(name, &[], ty, evidence, scope)?;
        self.capabilities.insert("test".into());
        Ok(Some(self.test_callable(name, arity, evidence, scope)?))
    }

    pub(super) fn test_call(&mut self, expr: &CoreExpr, scope: &Scope) -> Result<Option<String>> {
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
                    return Err("Rust backend: unproven std/test source staging".into());
                }
                if !deferred_evidence_parameters.is_empty() {
                    if arguments.len() + deferred_evidence_parameters.len()
                        != signature(callee).unwrap().0.len()
                    {
                        return Err("Rust backend: std/test deferred source arity mismatch".into());
                    }
                    let mut rest = type_ref;
                    for deferred in deferred_evidence_parameters {
                        let CoreType::Function { parameter, result } = rest else {
                            return Err("Rust backend: missing std/test deferred stage".into());
                        };
                        if self.local_canonical_type(parameter, scope)?
                            != self.local_canonical_type(deferred, scope)?
                        {
                            return Err(
                                "Rust backend: std/test deferred source type mismatch".into()
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
        let arity = self.test_type(name, arguments, &result, evidence, scope)?;
        if saturated && arguments.len() != arity {
            return Err("Rust backend: std/test Effect source arity mismatch".into());
        }
        if let Some(checked) = checked {
            if checked.parameters.len() != arity {
                return Err("Rust backend: std/test declaration arity mismatch".into());
            }
            let full = arrow(&checked.parameters, checked.result.clone());
            if checked.type_parameters.len() != checked.instantiated_type_arguments.len() {
                return Err("Rust backend: std/test requires proven type arguments".into());
            }
            let full = if checked.type_parameters.is_empty() {
                full
            } else {
                let mut substitutions = BTreeMap::new();
                let mut names = BTreeSet::new();
                for (parameter, argument) in checked
                    .type_parameters
                    .iter()
                    .zip(&checked.instantiated_type_arguments)
                {
                    if parameter.arity != 0
                        || !names.insert(&parameter.name)
                        || self.declared_method_type_kind(argument, scope) != Some(0)
                    {
                        return Err("Rust backend: std/test requires value type arguments".into());
                    }
                    substitutions.insert(
                        parameter.name.clone(),
                        self.local_canonical_type(argument, scope)?,
                    );
                }
                display::canonical_type(&full, self.owner, &substitutions, self.type_names, false)
                    .ok_or("Rust backend: unproven std/test declaration type")?
            };
            self.test_type(name, &[], &full, evidence, scope)?;
            if !checked.argument_types.is_empty() {
                if checked.argument_types.len() != arguments.len() {
                    return Err("Rust backend: std/test checked argument arity mismatch".into());
                }
                for (checked, argument) in checked.argument_types.iter().zip(arguments) {
                    let actual = local::method_argument_type(argument).unwrap();
                    if self.local_canonical_type(checked, scope)?
                        != self.local_canonical_type(&actual, scope)?
                    {
                        return Err("Rust backend: std/test checked argument type mismatch".into());
                    }
                }
            }
        }
        self.capabilities.insert("test".into());
        if arguments.len() == arity {
            return Ok(Some(format!(
                "native_test_operation({}, {}, vec![{}])",
                quoted(name),
                self.evidence(evidence, scope)?,
                self.expressions(arguments, scope)?
            )));
        }
        let callable = self.test_callable(name, arity, evidence, scope)?;
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
