use super::*;

pub(super) fn constructor_slots(name: &str) -> Option<Vec<u32>> {
    // Closed numeric API owners take no type parameters. Applied prefixes
    // remain kind-checked normally, including saved conversion functions.
    matches!(
        name,
        "std/int::IntParseError"
            | "std/int::IntDivisionError"
            | "std/int::IntPowerError"
            | "std/float::FloatParseError"
            | "std/float::FloatConversionError"
            | "std/number::RoundingMode"
    )
    .then(Vec::new)
}

pub(super) fn is_show_dictionary(identity: &str) -> bool {
    identity == "Show<std/int::IntParseError>"
}

fn signature(name: &str) -> Option<(Vec<CoreType>, CoreType)> {
    let named = |name: &str| CoreType::Named {
        name: name.into(),
        arguments: vec![],
    };
    let error = named("std/int::IntParseError");
    let float_error = named("std/float::FloatParseError");
    let int = named("std/prelude::Int");
    let either_int = |error: &str| CoreType::Named {
        name: "std/prelude::Either".into(),
        arguments: vec![named(error), int.clone()],
    };
    Some(match name {
        "std/int::checkedAdd" | "std/int::checkedSubtract" | "std/int::checkedMultiply" => (
            vec![int.clone(), int.clone()],
            CoreType::Named {
                name: "std/prelude::Maybe".into(),
                arguments: vec![int],
            },
        ),
        "std/int::saturatingAdd"
        | "std/int::saturatingSubtract"
        | "std/int::saturatingMultiply"
        | "std/int::minimum"
        | "std/int::maximum" => (vec![int.clone(), int.clone()], int),
        "std/int::checkedDivide" | "std/int::checkedRemainder" => (
            vec![int.clone(), int.clone()],
            either_int("std/int::IntDivisionError"),
        ),
        "std/int::checkedPower" | "std/int::saturatingPower" => (
            vec![int.clone(), int.clone()],
            either_int("std/int::IntPowerError"),
        ),
        "std/int::abs" | "std/int::sign" => (vec![int.clone()], int),
        "std/int::clamp" => (vec![int.clone(), int.clone(), int.clone()], int),
        "std/int::IntDivisionByZero" => (vec![], named("std/int::IntDivisionError")),
        "std/int::IntPowerOverflow" => (vec![], named("std/int::IntPowerError")),
        "std/int::NegativeIntExponent" => (vec![int], named("std/int::IntPowerError")),
        "std/float::toInt" => (
            vec![
                named("std/number::RoundingMode"),
                named("std/prelude::Float"),
            ],
            CoreType::Named {
                name: "std/prelude::Either".into(),
                arguments: vec![
                    named("std/float::FloatConversionError"),
                    named("std/prelude::Int"),
                ],
            },
        ),
        "std/float::roundIntegral" => (
            vec![
                named("std/number::RoundingMode"),
                named("std/prelude::Float"),
            ],
            named("std/prelude::Float"),
        ),
        "std/float::FloatNotFinite" | "std/float::FloatOutsideIntRange" => {
            (vec![], named("std/float::FloatConversionError"))
        }
        "std/number::HalfEven"
        | "std/number::HalfUp"
        | "std/number::TowardZero"
        | "std/number::AwayFromZero"
        | "std/number::Floor"
        | "std/number::Ceiling" => (vec![], named("std/number::RoundingMode")),
        "std/float::parse" => (
            vec![named("std/prelude::String")],
            CoreType::Named {
                name: "std/prelude::Either".into(),
                arguments: vec![float_error, named("std/prelude::Float")],
            },
        ),
        "std/float::EmptyFloat" | "std/float::FloatParseOverflow" => (vec![], float_error),
        "std/float::InvalidFloat" => (
            vec![CoreType::Record {
                closed: true,
                fields: vec![seseragi_lowering::CoreRecordField {
                    name: "offset".into(),
                    optional: false,
                    type_ref: named("std/prelude::Int"),
                }],
            }],
            float_error,
        ),
        "std/int::parse" => (
            vec![named("std/prelude::String")],
            CoreType::Named {
                name: "std/prelude::Either".into(),
                arguments: vec![error, named("std/prelude::Int")],
            },
        ),
        "std/int::parseRadix" => (
            vec![named("std/prelude::Int"), named("std/prelude::String")],
            CoreType::Named {
                name: "std/prelude::Either".into(),
                arguments: vec![error, named("std/prelude::Int")],
            },
        ),
        "std/int::minValue" | "std/int::maxValue" => {
            (vec![named("std/prelude::Unit")], named("std/prelude::Int"))
        }
        "std/int::format" => (
            vec![named("std/prelude::Int")],
            named("std/prelude::String"),
        ),
        "std/int::formatRadix" => (
            vec![named("std/prelude::Int"), named("std/prelude::Int")],
            CoreType::Named {
                name: "std/prelude::Either".into(),
                arguments: vec![error, named("std/prelude::String")],
            },
        ),
        "std/int::EmptyInt" | "std/int::IntOutsideRange" => (vec![], error),
        "std/int::InvalidIntRadix" => (vec![named("std/prelude::Int")], error),
        "std/int::InvalidIntDigit" => (
            vec![CoreType::Record {
                closed: true,
                fields: ["offset", "radix"]
                    .into_iter()
                    .map(|name| seseragi_lowering::CoreRecordField {
                        name: name.into(),
                        optional: false,
                        type_ref: named("std/prelude::Int"),
                    })
                    .collect(),
            }],
            error,
        ),
        _ => return None,
    })
}

// Called only after the exact canonical signature above was admitted.
fn parse_capability(name: &str) -> &'static str {
    match name {
        "std/int::checkedAdd"
        | "std/int::checkedSubtract"
        | "std/int::checkedMultiply"
        | "std/int::saturatingAdd"
        | "std/int::saturatingSubtract"
        | "std/int::saturatingMultiply"
        | "std/int::minimum"
        | "std/int::maximum"
        | "std/int::checkedDivide"
        | "std/int::checkedRemainder"
        | "std/int::checkedPower"
        | "std/int::saturatingPower"
        | "std/int::abs"
        | "std/int::clamp"
        | "std/int::sign"
        | "std/int::IntDivisionByZero"
        | "std/int::IntPowerOverflow"
        | "std/int::NegativeIntExponent" => "int-checked",
        "std/int::minValue"
        | "std/int::maxValue"
        | "std/int::parseRadix"
        | "std/int::format"
        | "std/int::formatRadix" => "int-text",
        "std/float::toInt"
        | "std/float::roundIntegral"
        | "std/float::FloatNotFinite"
        | "std/float::FloatOutsideIntRange"
        | "std/number::HalfEven"
        | "std/number::HalfUp"
        | "std/number::TowardZero"
        | "std/number::AwayFromZero"
        | "std/number::Floor"
        | "std/number::Ceiling" => "float-conversion",
        "std/float::parse"
        | "std/float::EmptyFloat"
        | "std/float::InvalidFloat"
        | "std/float::FloatParseOverflow" => "float-parse",
        _ => "int-parse",
    }
}

fn remaining_type(parameters: &[CoreType], result: &CoreType) -> CoreType {
    parameters
        .iter()
        .rev()
        .fold(result.clone(), |result, parameter| CoreType::Function {
            parameter: Box::new(parameter.clone()),
            result: Box::new(result),
        })
}

impl Backend<'_> {
    pub(super) fn validate_int_parse_show_call(
        &self,
        expr: &CoreExpr,
        scope: &Scope,
    ) -> Result<()> {
        let CoreExpr::Call {
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
            return Ok(());
        };
        if !evidence.iter().any(|selected| {
            matches!(&selected.evidence, CoreInstanceEvidence::Standard { identity, .. }
                if is_show_dictionary(identity))
        }) {
            return Ok(());
        }
        let valid_dispatch = trait_dispatch.as_ref().is_some_and(|dispatch| {
            dispatch.trait_identity == "std/prelude::Show"
                && dispatch.method == "show"
                && dispatch.method_contract.is_none()
        });
        if evidence.len() != 1
            || !valid_dispatch
            || arguments.len() > 1
            || !deferred_evidence_parameters.is_empty()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || evidence_argument_index.is_some()
        {
            return Err("Rust backend: unproven Int parse Show source application".into());
        }
        let parameter = CoreType::Named {
            name: "std/int::IntParseError".into(),
            arguments: vec![],
        };
        let result = CoreType::Named {
            name: "std/prelude::String".into(),
            arguments: vec![],
        };
        if let Some(argument) = arguments.first() {
            let actual = local::method_argument_type(argument)
                .ok_or("Rust backend: missing Int parse Show argument type")?;
            self.number_parse_type_matches(&actual, &parameter, scope)?;
        }
        // A saved method keeps its one source argument. A saturated call must
        // produce String; dictionary selection never consumes that argument.
        let parameters = [parameter];
        self.number_parse_type_matches(
            type_ref,
            &remaining_type(&parameters[arguments.len()..], &result),
            scope,
        )
    }

    pub(super) fn int_parse_show_dictionary(
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
        if !is_show_dictionary(identity) {
            return Ok(None);
        }
        let [actual] = selected.constraint.arguments.as_slice() else {
            return Err("Rust backend: Int parse Show constraint arity".into());
        };
        let expected = CoreType::Named {
            name: "std/int::IntParseError".into(),
            arguments: vec![],
        };
        if !type_arguments.is_empty()
            || !evidence_arguments.is_empty()
            || !lists::standard_trait(selected, "Show", actual)
            || self.declared_method_type_kind(actual, scope) != Some(0)
            || self.local_canonical_type(actual, scope)? != expected
        {
            return Err("Rust backend: unproven canonical Int parse Show evidence".into());
        }
        Ok(Some("native_int_parse_error_show_dictionary()".into()))
    }

    fn number_parse_callable(name: &str, arity: usize) -> String {
        let names: Vec<_> = (0..arity)
            .map(|i| format!("\0number_parse_argument_{i}"))
            .collect();
        let arguments = names
            .iter()
            .map(|n| format!("{}.clone()", id(n)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = format!(
            "native_number_parse_operation({}, vec![{arguments}])",
            quoted(name)
        );
        for i in (0..arity).rev() {
            body = Self::closure(&names[i], &body, &names[..i].iter().cloned().collect());
        }
        body
    }

    fn number_parse_type_matches(
        &self,
        actual: &CoreType,
        expected: &CoreType,
        scope: &Scope,
    ) -> Result<()> {
        if self.declared_method_type_kind(actual, scope) != Some(0)
            || self.local_canonical_type(actual, scope)? != *expected
        {
            return Err(
                "Rust backend: unproven canonical number parse argument/result type".into(),
            );
        }
        Ok(())
    }

    pub(super) fn number_parse_reference(
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
        if self.program_wasm_library && parse_capability(name) == "float-conversion" {
            return Err("Rust backend: Float conversion is not admitted in program WASM".into());
        }
        if self.program_wasm_library && matches!(parse_capability(name), "int-text" | "int-checked")
        {
            return Err(
                "Rust backend: Int standard operations are not admitted in program WASM".into(),
            );
        }
        if !evidence.is_empty() {
            return Err("Rust backend: number parse operation with unsupported evidence".into());
        }
        self.number_parse_type_matches(type_ref, &remaining_type(&parameters, &result), scope)?;
        self.capabilities.insert(parse_capability(name).into());
        Ok(Some(Self::number_parse_callable(name, parameters.len())))
    }

    pub(super) fn number_parse_call(
        &mut self,
        expr: &CoreExpr,
        scope: &Scope,
    ) -> Result<Option<String>> {
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
        let Some((parameters, result)) = signature(callee) else {
            return Ok(None);
        };
        if scope.contains(callee) {
            return Ok(None);
        }
        if self.program_wasm_library && parse_capability(callee) == "float-conversion" {
            return Err("Rust backend: Float conversion is not admitted in program WASM".into());
        }
        if self.program_wasm_library
            && matches!(parse_capability(callee), "int-text" | "int-checked")
        {
            return Err(
                "Rust backend: Int standard operations are not admitted in program WASM".into(),
            );
        }
        if arguments.len() > parameters.len()
            || !evidence.is_empty()
            || trait_dispatch.is_some()
            || !deferred_evidence_parameters.is_empty()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || evidence_argument_index.is_some()
        {
            return Err("Rust backend: unproven number parse source application".into());
        }
        for (argument, expected) in arguments.iter().zip(&parameters) {
            let actual = local::method_argument_type(argument)
                .ok_or("Rust backend: missing number parse argument type")?;
            self.number_parse_type_matches(&actual, expected, scope)?;
        }
        self.number_parse_type_matches(
            type_ref,
            &remaining_type(&parameters[arguments.len()..], &result),
            scope,
        )?;
        self.capabilities.insert(parse_capability(callee).into());
        if arguments.len() == parameters.len() {
            self.direct_calls += 1;
            return Ok(Some(format!(
                "native_number_parse_operation({}, vec![{}])",
                quoted(callee),
                self.expressions(arguments, scope)?
            )));
        }
        let value = self.fresh();
        let mut output = format!(
            "{{ let mut {value} = {};",
            Self::number_parse_callable(callee, parameters.len())
        );
        for argument in arguments {
            output.push_str(&format!(
                "{value} = {value}.apply({});",
                self.expr(argument, scope)?
            ));
        }
        output.push_str(&format!("{value} }}"));
        Ok(Some(output))
    }
}
