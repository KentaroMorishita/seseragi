use super::*;

fn value(name: &str) -> CoreType {
    CoreType::Named {
        name: name.into(),
        arguments: vec![],
    }
}

fn signature(name: &str) -> Option<(Vec<CoreType>, CoreType)> {
    let date = || value("std/time::LocalDate");
    let time = || value("std/time::LocalTime");
    let local = || value("std/time::LocalDateTime");
    let offset = || value("std/time::UtcOffset");
    let zoned = || value("std/time::OffsetDateTime");
    let instant = || value("std/time::Instant");
    let int = || value("std/prelude::Int");
    let text = || value("std/prelude::String");
    let zone = || value("std/time::TimeZone");
    let zoned_time = || value("std/time::ZonedDateTime");
    let resolution = || value("std/time::LocalResolution");
    let record = |fields: Vec<(&str, CoreType)>| CoreType::Record {
        closed: true,
        fields: fields
            .into_iter()
            .map(|(name, type_ref)| seseragi_lowering::CoreRecordField {
                name: name.into(),
                optional: false,
                type_ref,
            })
            .collect(),
    };
    let effect = |failure, success| CoreType::Named {
        name: "std/prelude::Effect".into(),
        arguments: vec![
            record(vec![("timeZones", value("std/time::TimeZones"))]),
            failure,
            success,
        ],
    };
    let either = |success| CoreType::Named {
        name: "std/prelude::Either".into(),
        arguments: vec![value("std/time::DateTimeError"), success],
    };
    Some(match name {
        "std/time::localDate" => (vec![int(), int(), int()], either(date())),
        "std/time::localTime" => (vec![int(), int(), int(), int()], either(time())),
        "std/time::localDateTime" => (vec![date(), time()], local()),
        "std/time::utcOffset" => (vec![int()], either(offset())),
        "std/time::parseLocalDate" => (vec![text()], either(date())),
        "std/time::parseLocalTime" => (vec![text()], either(time())),
        "std/time::parseLocalDateTime" => (vec![text()], either(local())),
        "std/time::parseOffsetDateTime" => (vec![text()], either(zoned())),
        "std/time::formatLocalDate" => (vec![date()], text()),
        "std/time::formatLocalTime" => (vec![time()], text()),
        "std/time::formatLocalDateTime" => (vec![local()], text()),
        "std/time::formatOffsetDateTime" => (vec![zoned()], text()),
        "std/time::atOffset" => (vec![offset(), instant()], zoned()),
        "std/time::offsetInstant" => (vec![zoned()], instant()),
        "std/time::offsetLocalDateTime" => (vec![zoned()], local()),
        "std/time::databaseVersion" => (
            vec![value("std/prelude::Unit")],
            effect(value("std/prelude::Never"), text()),
        ),
        "std/time::loadTimeZone" => (
            vec![text()],
            effect(value("std/time::TimeZoneError"), zone()),
        ),
        "std/time::timeZoneId" | "std/time::timeZoneVersion" => (vec![zone()], text()),
        "std/time::atTimeZone" => (vec![instant(), zone()], zoned_time()),
        "std/time::resolveLocal" => (vec![local(), zone()], resolution()),
        "std/time::zonedInstant" => (vec![zoned_time()], instant()),
        "std/time::zonedLocalDateTime" => (vec![zoned_time()], local()),
        "std/time::zonedOffset" => (vec![zoned_time()], offset()),
        "std/time::zonedTimeZone" => (vec![zoned_time()], zone()),
        "std/time::Unique" => (vec![zoned_time()], resolution()),
        "std/time::Ambiguous" => (
            vec![record(vec![
                ("earlier", zoned_time()),
                ("later", zoned_time()),
            ])],
            resolution(),
        ),
        "std/time::Gap" => (
            vec![record(vec![
                ("transition", instant()),
                ("offsetBefore", offset()),
                ("offsetAfter", offset()),
            ])],
            resolution(),
        ),
        _ => return None,
    })
}

fn timezone_operation(name: &str) -> bool {
    matches!(
        name,
        "std/time::databaseVersion"
            | "std/time::loadTimeZone"
            | "std/time::timeZoneId"
            | "std/time::timeZoneVersion"
            | "std/time::atTimeZone"
            | "std/time::resolveLocal"
            | "std/time::zonedInstant"
            | "std/time::zonedLocalDateTime"
            | "std/time::zonedOffset"
            | "std/time::zonedTimeZone"
            | "std/time::Unique"
            | "std/time::Ambiguous"
            | "std/time::Gap"
    )
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

impl Backend<'_> {
    fn time_type_matches(
        &self,
        actual: &CoreType,
        expected: &CoreType,
        scope: &Scope,
    ) -> Result<()> {
        if self.declared_method_type_kind(actual, scope) != Some(0)
            || self.local_canonical_type(actual, scope)? != *expected
        {
            return Err("Rust backend: unproven canonical calendar argument or result".into());
        }
        Ok(())
    }

    fn require_time_calendar(&mut self, name: &str) -> Result<()> {
        if self.program_wasm_library {
            return Err(
                "Rust backend: native calendar APIs are not admitted in program WASM".into(),
            );
        }
        self.capabilities.insert("time-calendar".into());
        if timezone_operation(name) {
            self.capabilities.insert("time-zones".into());
        }
        if matches!(name, "std/time::databaseVersion" | "std/time::loadTimeZone") {
            self.capabilities.insert("effect-sync".into());
        }
        Ok(())
    }

    fn time_callable(name: &str, arity: usize) -> String {
        let names = (0..arity)
            .map(|i| format!("\0time_argument_{i}"))
            .collect::<Vec<_>>();
        let args = names
            .iter()
            .map(|name| format!("{}.clone()", id(name)))
            .collect::<Vec<_>>()
            .join(",");
        let operation = if timezone_operation(name) {
            "native_timezone_operation"
        } else {
            "native_time_operation"
        };
        let mut result = format!("{operation}({}, vec![{args}])", quoted(name));
        for i in (0..arity).rev() {
            result = Self::closure(&names[i], &result, &names[..i].iter().cloned().collect());
        }
        result
    }

    pub(super) fn time_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        ty: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if scope.contains(name) {
            return Ok(None);
        }
        let Some((parameters, result)) = signature(name) else {
            return Ok(None);
        };
        if !evidence.is_empty() {
            return Err("Rust backend: calendar APIs have no dictionary evidence".into());
        }
        self.time_type_matches(ty, &arrow(&parameters, &result), scope)?;
        self.require_time_calendar(name)?;
        Ok(Some(Self::time_callable(name, parameters.len())))
    }

    pub(super) fn time_call(&mut self, expr: &CoreExpr, scope: &Scope) -> Result<Option<String>> {
        let (callee, arguments, type_ref, valid, callee_signature) = match expr {
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
                CoreType::Named {
                    name: "std/prelude::Effect".into(),
                    arguments: vec![requirements.clone(), failure.clone(), success.clone()],
                },
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
                CoreType::Named {
                    name: "std/prelude::Effect".into(),
                    arguments: vec![requirements.clone(), failure.clone(), success.clone()],
                },
                true,
                None,
            ),
            _ => return Ok(None),
        };
        if scope.contains(callee) {
            return Ok(None);
        }
        let Some((parameters, expected)) = signature(callee) else {
            return Ok(None);
        };
        if !valid || arguments.len() > parameters.len() {
            return Err("Rust backend: unproven calendar source arity or evidence".into());
        }
        if let Some(signature) = callee_signature {
            if !signature.type_parameters.is_empty()
                || !signature.instantiated_type_arguments.is_empty()
                || signature.parameters.len() != parameters.len()
            {
                return Err("Rust backend: unproven calendar declaration source arity".into());
            }
            for (actual, expected) in signature.parameters.iter().zip(&parameters) {
                self.time_type_matches(actual, expected, scope)?;
            }
            self.time_type_matches(&signature.result, &expected, scope)?;
            if !signature.argument_types.is_empty() {
                if signature.argument_types.len() != arguments.len() {
                    return Err("Rust backend: unproven calendar actual argument arity".into());
                }
                for (actual, expected) in signature.argument_types.iter().zip(&parameters) {
                    self.time_type_matches(actual, expected, scope)?;
                }
            }
        }
        for (argument, expected) in arguments.iter().zip(&parameters) {
            let actual = local::method_argument_type(argument)
                .ok_or("Rust backend: missing calendar argument type")?;
            self.time_type_matches(&actual, expected, scope)?;
        }
        self.time_type_matches(
            &type_ref,
            &arrow(&parameters[arguments.len()..], &expected),
            scope,
        )?;
        self.require_time_calendar(callee)?;
        let temporary = self.fresh();
        let mut output = format!(
            "{{ let mut {temporary} = {};",
            Self::time_callable(callee, parameters.len())
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
