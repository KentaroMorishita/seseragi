use super::*;

fn arity(name: &str) -> Option<usize> {
    match name {
        "std/effect::Continue" | "std/effect::Break" | "std/effect::ScheduleStop" => Some(0),
        "std/effect::attempt"
        | "std/effect::ScheduleContinue"
        | "std/effect::NegativeRecurrences"
        | "std/effect::schedule"
        | "std/effect::recurs"
        | "std/effect::whileInput" => Some(1),
        "std/effect::spaced" | "std/effect::retry" | "std/effect::repeat" => Some(2),
        "std/effect::fromMaybe"
        | "std/effect::forEachUntil"
        | "std/effect::race"
        | "std/effect::timeout" => Some(2),
        "std/effect::timeoutFail" => Some(3),
        _ => None,
    }
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
/// Effect control operations that wait on the native Clock.
fn uses_clock(name: &str) -> bool {
    matches!(
        name,
        "std/effect::timeout"
            | "std/effect::timeoutFail"
            | "std/effect::retry"
            | "std/effect::repeat"
    )
}

/// `requirements & { clock: Clock }`, merged as the checker merges rows.
fn with_clock(requirements: &CoreType) -> Result<CoreType> {
    let mut operands = match requirements {
        CoreType::RequirementMerge { operands } => operands.clone(),
        other => vec![other.clone()],
    };
    operands.push(CoreType::Record {
        closed: true,
        fields: vec![seseragi_lowering::CoreRecordField {
            name: "clock".into(),
            optional: false,
            type_ref: named("std/clock::Clock", vec![]),
        }],
    });
    let mut fields = BTreeMap::new();
    let mut other = Vec::new();
    for operand in operands {
        match operand {
            CoreType::Record {
                closed: true,
                fields: row,
            } => {
                for field in row {
                    if fields
                        .insert(field.name.clone(), field.clone())
                        .is_some_and(|previous| previous != field)
                    {
                        return Err("Rust backend: conflicting Clock requirement".into());
                    }
                }
            }
            operand => other.push(operand),
        }
    }
    let row = CoreType::Record {
        closed: true,
        fields: fields.into_values().collect(),
    };
    Ok(if other.is_empty() {
        row
    } else {
        other.push(row);
        CoreType::RequirementMerge { operands: other }
    })
}

/// `ty` with every record's fields in name order.
fn records_in_name_order(ty: &CoreType) -> CoreType {
    match ty {
        CoreType::Record { closed, fields } => {
            let mut fields = fields
                .iter()
                .map(|field| seseragi_lowering::CoreRecordField {
                    type_ref: records_in_name_order(&field.type_ref),
                    ..field.clone()
                })
                .collect::<Vec<_>>();
            fields.sort_by(|left, right| left.name.cmp(&right.name));
            CoreType::Record {
                closed: *closed,
                fields,
            }
        }
        CoreType::Named { name, arguments } => CoreType::Named {
            name: name.clone(),
            arguments: arguments.iter().map(records_in_name_order).collect(),
        },
        CoreType::Tuple { elements } => CoreType::Tuple {
            elements: elements.iter().map(records_in_name_order).collect(),
        },
        CoreType::Function { parameter, result } => CoreType::Function {
            parameter: Box::new(records_in_name_order(parameter)),
            result: Box::new(records_in_name_order(result)),
        },
        CoreType::RequirementMerge { operands } => CoreType::RequirementMerge {
            operands: operands.iter().map(records_in_name_order).collect(),
        },
        other => other.clone(),
    }
}

fn effect_arguments(ty: &CoreType) -> Result<&[CoreType]> {
    match ty {
        CoreType::Named { name, arguments }
            if name == "std/prelude::Effect" && arguments.len() == 3 =>
        {
            Ok(arguments)
        }
        _ => Err("Rust backend: Effect control requires canonical Effect".into()),
    }
}

impl Backend<'_> {
    fn effect_control_type(
        &self,
        name: &str,
        arguments: &[CoreExpr],
        result: &CoreType,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<usize> {
        if self.program_wasm_library {
            return Err(
                "program WASM: native Effect control requires a target-specific ABI proof".into(),
            );
        }
        let arity = arity(name).expect("known Effect control");
        if scope.has_unproven_type_scope() || arguments.len() > arity {
            return Err("Rust backend: unproven Effect control source application".into());
        }
        let supplied = arguments
            .iter()
            .map(|argument| {
                local::method_argument_type(argument)
                    .ok_or_else(|| "Rust backend: missing Effect control argument type".into())
            })
            .collect::<Result<Vec<_>>>()?;
        let full = function(&supplied, result.clone());
        if self.declared_method_type_kind(&full, scope) != Some(0) {
            return Err("Rust backend: Effect control requires value-kind source/result".into());
        }
        let full = self.local_canonical_type(&full, scope)?;
        let mut output = &full;
        let mut parameters = Vec::new();
        for _ in 0..arity {
            let CoreType::Function { parameter, result } = output else {
                return Err("Rust backend: missing Effect control source arity".into());
            };
            parameters.push(parameter.as_ref().clone());
            output = result;
        }
        let control = named("std/effect::LoopControl", vec![]);
        let expected = match name {
            "std/effect::Continue" | "std/effect::Break" => control,
            "std/effect::attempt" => {
                let slots = effect_arguments(&parameters[0])?;
                named(
                    "std/prelude::Effect",
                    vec![
                        slots[0].clone(),
                        named("std/prelude::Never", vec![]),
                        named("std/prelude::Either", slots[1..].to_vec()),
                    ],
                )
            }
            "std/effect::fromMaybe" => {
                let CoreType::Named { name, arguments } = &parameters[1] else {
                    return Err("Rust backend: fromMaybe requires canonical Maybe".into());
                };
                if name != "std/prelude::Maybe" || arguments.len() != 1 {
                    return Err(
                        "Rust backend: fromMaybe canonical Maybe owner/arity mismatch".into(),
                    );
                }
                named(
                    "std/prelude::Effect",
                    vec![
                        CoreType::Record {
                            closed: true,
                            fields: vec![],
                        },
                        parameters[0].clone(),
                        arguments[0].clone(),
                    ],
                )
            }
            "std/effect::forEachUntil" => {
                let [selected] = evidence else {
                    return Err("Rust backend: forEachUntil requires selected Iterable".into());
                };
                self.validate_iterable_evidence(selected, scope)?;
                let CoreType::Function { parameter, result } = &parameters[0] else {
                    return Err("Rust backend: forEachUntil requires an action".into());
                };
                let slots = effect_arguments(result)?;
                if self.local_canonical_type(&selected.constraint.arguments[0], scope)?
                    != parameters[1]
                    || self.local_canonical_type(&selected.constraint.arguments[1], scope)?
                        != **parameter
                    || slots[2] != control
                {
                    return Err(
                        "Rust backend: forEachUntil source, element or LoopControl mismatch".into(),
                    );
                }
                named(
                    "std/prelude::Effect",
                    vec![
                        slots[0].clone(),
                        slots[1].clone(),
                        named("std/prelude::Unit", vec![]),
                    ],
                )
            }
            "std/effect::timeout" | "std/effect::timeoutFail" => {
                let offset = usize::from(name == "std/effect::timeoutFail");
                let slots = effect_arguments(&parameters[offset + 1])?;
                if parameters[offset] != named("std/time::Duration", vec![])
                    || (offset == 1 && parameters[0] != slots[1])
                {
                    return Err("Rust backend: timeout Duration or error type mismatch".into());
                }
                named(
                    "std/prelude::Effect",
                    vec![
                        with_clock(&slots[0])?,
                        slots[1].clone(),
                        if offset == 0 {
                            named("std/prelude::Maybe", vec![slots[2].clone()])
                        } else {
                            slots[2].clone()
                        },
                    ],
                )
            }
            "std/effect::ScheduleStop" => named("std/effect::ScheduleDecision", vec![]),
            "std/effect::ScheduleContinue" => {
                if parameters[0] != named("std/time::Duration", vec![]) {
                    return Err("Rust backend: ScheduleContinue requires a Duration".into());
                }
                named("std/effect::ScheduleDecision", vec![])
            }
            "std/effect::NegativeRecurrences" => {
                if parameters[0] != named("std/prelude::Int", vec![]) {
                    return Err("Rust backend: NegativeRecurrences requires an Int".into());
                }
                named("std/effect::ScheduleError", vec![])
            }
            "std/effect::schedule" | "std/effect::whileInput" => {
                let input = match (&parameters[0], name) {
                    (CoreType::Function { parameter, result }, "std/effect::schedule")
                        if **parameter == named("std/prelude::Int", vec![]) =>
                    {
                        match result.as_ref() {
                            CoreType::Function { parameter, result }
                                if **result == named("std/effect::ScheduleDecision", vec![]) =>
                            {
                                parameter.as_ref().clone()
                            }
                            _ => return Err("Rust backend: schedule requires a decision".into()),
                        }
                    }
                    (CoreType::Function { parameter, result }, "std/effect::whileInput")
                        if **result == named("std/prelude::Bool", vec![]) =>
                    {
                        parameter.as_ref().clone()
                    }
                    _ => return Err("Rust backend: unproven Schedule constructor".into()),
                };
                named("std/effect::Schedule", vec![input])
            }
            "std/effect::recurs" | "std/effect::spaced" => {
                if parameters[0] != named("std/prelude::Int", vec![])
                    || (name == "std/effect::spaced"
                        && parameters[1] != named("std/time::Duration", vec![]))
                {
                    return Err("Rust backend: Schedule count or delay mismatch".into());
                }
                // The input type comes only from the result.
                match output {
                    CoreType::Named { name, arguments }
                        if name == "std/prelude::Either"
                            && arguments.len() == 2
                            && arguments[0] == named("std/effect::ScheduleError", vec![])
                            && matches!(&arguments[1], CoreType::Named { name, arguments }
                                if name == "std/effect::Schedule" && arguments.len() == 1) =>
                    {
                        output.clone()
                    }
                    _ => return Err("Rust backend: Schedule result mismatch".into()),
                }
            }
            "std/effect::retry" | "std/effect::repeat" => {
                let slots = effect_arguments(&parameters[1])?;
                let observed = if name == "std/effect::retry" {
                    &slots[1]
                } else {
                    &slots[2]
                };
                if parameters[0] != named("std/effect::Schedule", vec![observed.clone()]) {
                    return Err("Rust backend: Schedule input does not match the Effect".into());
                }
                named(
                    "std/prelude::Effect",
                    vec![with_clock(&slots[0])?, slots[1].clone(), slots[2].clone()],
                )
            }
            "std/effect::race" => {
                effect_arguments(&parameters[0])?;
                if parameters[0] != parameters[1] {
                    return Err("Rust backend: race requires two Effects of one type".into());
                }
                parameters[0].clone()
            }
            _ => unreachable!(),
        };
        if name != "std/effect::forEachUntil" && !evidence.is_empty() {
            return Err("Rust backend: unexpected Effect control dictionary".into());
        }
        // Requirement rows are records, which list fields in any order.
        if records_in_name_order(output) != records_in_name_order(&expected) {
            return Err("Rust backend: Effect control canonical result mismatch".into());
        }
        Ok(arity)
    }

    fn effect_control_callable(
        &mut self,
        name: &str,
        arity: usize,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<String> {
        let dictionary = "\0effect_control_dictionaries".to_owned();
        let names: Vec<_> = (0..arity)
            .map(|i| format!("\0effect_control_argument_{i}"))
            .collect();
        let args = names
            .iter()
            .map(|n| format!("{}.clone()", id(n)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = format!(
            "native_effect_control({}, native_map_dictionaries(&{}), vec![{args}])",
            quoted(name),
            id(&dictionary)
        );
        for i in (0..arity).rev() {
            let mut captures: Scope = names[..i].iter().cloned().collect();
            captures.insert(dictionary.clone());
            body = Self::closure(&names[i], &body, &captures);
        }
        let dictionaries = self.evidence(evidence, scope)?;
        Ok(format!(
            "{{ let {}=V::Tuple(Shared::new({dictionaries})); {body} }}",
            id(&dictionary)
        ))
    }

    pub(super) fn effect_control_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        origin: &seseragi_lowering::SourceSpan,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if arity(name).is_none() || scope.contains(name) {
            return Ok(None);
        }
        let arity = self.effect_control_type(name, &[], type_ref, evidence, scope)?;
        if uses_clock(name) {
            self.require_clock("std/clock::sleep", origin)?;
        }
        self.capabilities.insert("effect-sync".into());
        Ok(Some(
            self.effect_control_callable(name, arity, evidence, scope)?,
        ))
    }

    pub(super) fn effect_control_call(
        &mut self,
        expr: &CoreExpr,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let (name, arguments, result, evidence) = match expr {
            CoreExpr::Call {
                callee,
                arguments,
                type_ref,
                evidence,
                ..
            } => (callee, arguments, type_ref.clone(), evidence.as_slice()),
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
                evidence.as_slice(),
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
                &[][..],
            ),
            _ => return Ok(None),
        };
        if arity(name).is_none() || scope.contains(name) {
            return Ok(None);
        }
        if let CoreExpr::Call {
            trait_dispatch,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            ..
        } = expr
        {
            if trait_dispatch.is_some()
                || !deferred_evidence_type_constructor_parameters.is_empty()
                || evidence_argument_index.is_some()
            {
                return Err("Rust backend: unproven Effect control source stages".into());
            }
            if !deferred_evidence_parameters.is_empty() {
                if arguments.len() + deferred_evidence_parameters.len() != arity(name).unwrap() {
                    return Err(
                        "Rust backend: Effect control deferred source arity mismatch".into(),
                    );
                }
                let mut remaining = &result;
                for deferred in deferred_evidence_parameters {
                    let CoreType::Function { parameter, result } = remaining else {
                        return Err(
                            "Rust backend: missing Effect control deferred source stage".into()
                        );
                    };
                    if self.local_canonical_type(parameter, scope)?
                        != self.local_canonical_type(deferred, scope)?
                    {
                        return Err(
                            "Rust backend: Effect control deferred source type mismatch".into()
                        );
                    }
                    remaining = result;
                }
            }
        }
        let arity = self.effect_control_type(name, arguments, &result, evidence, scope)?;
        if uses_clock(name) {
            self.require_clock("std/clock::sleep", core_expr_origin(expr))?;
        }
        self.capabilities.insert("effect-sync".into());
        if arguments.len() == arity {
            self.direct_calls += 1;
            let dictionaries = self.evidence(evidence, scope)?;
            return Ok(Some(format!(
                "native_effect_control({}, {dictionaries}, vec![{}])",
                quoted(name),
                self.expressions(arguments, scope)?
            )));
        }
        let value = self.fresh();
        let callable = self.effect_control_callable(name, arity, evidence, scope)?;
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
