use super::*;

fn signature(name: &str) -> Option<(Vec<CoreType>, CoreType)> {
    let named = |name: &str, arguments| CoreType::Named {
        name: name.into(),
        arguments,
    };
    let decimal = named("std/decimal::Decimal", vec![]);
    let text = named("std/prelude::String", vec![]);
    Some(match name {
        "std/decimal::parse" => (
            vec![text],
            named(
                "std/prelude::Either",
                vec![named("std/decimal::DecimalParseError", vec![]), decimal],
            ),
        ),
        "std/decimal::fromInt" => (vec![named("std/prelude::Int", vec![])], decimal),
        "std/decimal::toIntExact" => (
            vec![decimal],
            named(
                "std/prelude::Either",
                vec![
                    named("std/decimal::DecimalConversionError", vec![]),
                    named("std/prelude::Int", vec![]),
                ],
            ),
        ),
        "std/decimal::toFloat" => (
            vec![decimal],
            named(
                "std/prelude::Either",
                vec![
                    named("std/decimal::DecimalConversionError", vec![]),
                    named("std/prelude::Float", vec![]),
                ],
            ),
        ),
        "std/decimal::fromFloat" => (
            vec![
                named("std/decimal::DecimalContext", vec![]),
                named("std/prelude::Float", vec![]),
            ],
            named(
                "std/prelude::Either",
                vec![
                    named("std/decimal::DecimalConversionError", vec![]),
                    decimal,
                ],
            ),
        ),
        "std/decimal::DecimalNotIntegral"
        | "std/decimal::DecimalOutsideIntRange"
        | "std/decimal::DecimalOutsideFloatRange"
        | "std/decimal::FloatNotFinite" => {
            (vec![], named("std/decimal::DecimalConversionError", vec![]))
        }
        "std/decimal::divideExact" => (
            vec![decimal.clone(), decimal.clone()],
            named(
                "std/prelude::Either",
                vec![
                    named("std/decimal::DecimalArithmeticError", vec![]),
                    decimal,
                ],
            ),
        ),
        "std/decimal::divide" => (
            vec![
                named("std/decimal::DecimalContext", vec![]),
                decimal.clone(),
                decimal.clone(),
            ],
            named(
                "std/prelude::Either",
                vec![
                    named("std/decimal::DecimalArithmeticError", vec![]),
                    decimal,
                ],
            ),
        ),
        "std/decimal::quantize" => (
            vec![
                named("std/prelude::Int", vec![]),
                named("std/number::RoundingMode", vec![]),
                decimal.clone(),
            ],
            decimal,
        ),
        "std/decimal::DecimalDivisionByZero" | "std/decimal::NonTerminatingDecimal" => {
            (vec![], named("std/decimal::DecimalArithmeticError", vec![]))
        }
        "std/decimal::context" => (
            vec![
                named("std/prelude::Int", vec![]),
                named("std/number::RoundingMode", vec![]),
            ],
            named(
                "std/prelude::Either",
                vec![
                    named("std/decimal::DecimalContextError", vec![]),
                    named("std/decimal::DecimalContext", vec![]),
                ],
            ),
        ),
        "std/decimal::precision" => (
            vec![named("std/decimal::DecimalContext", vec![])],
            named("std/prelude::Int", vec![]),
        ),
        "std/decimal::rounding" => (
            vec![named("std/decimal::DecimalContext", vec![])],
            named("std/number::RoundingMode", vec![]),
        ),
        "std/decimal::NonPositiveDecimalPrecision" => (
            vec![named("std/prelude::Int", vec![])],
            named("std/decimal::DecimalContextError", vec![]),
        ),
        _ => return None,
    })
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
    pub(super) fn decimal_trait_dictionary(
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
        let function = match identity.as_str() {
            "std/decimal::Eq" => "native_decimal_eq_dictionary()",
            "std/decimal::Hash" => "native_decimal_hash_dictionary()",
            "std/decimal::Ord" => "native_decimal_ord_dictionary()",
            "std/decimal::Zero" => "native_decimal_constant_dictionary(false)",
            "std/decimal::One" => "native_decimal_constant_dictionary(true)",
            _ => return Ok(None),
        };
        if self.program_wasm_library || !type_arguments.is_empty() || !evidence_arguments.is_empty()
        {
            return Err(
                "Rust backend: unproven native Decimal dictionary type, children or target".into(),
            );
        }
        if identity == "std/decimal::Eq" {
            self.validate_eq_evidence(selected, scope)?;
        } else if identity == "std/decimal::Hash" {
            self.validate_hash_evidence(selected, scope)?;
        } else if identity == "std/decimal::Ord" {
            self.validate_ord_evidence(selected, scope)?;
        } else {
            let [actual] = selected.constraint.arguments.as_slice() else {
                return Err("Rust backend: Decimal constant constraint arity".into());
            };
            let name = if identity == "std/decimal::Zero" {
                "Zero"
            } else {
                "One"
            };
            if !lists::standard_trait(selected, name, actual) {
                return Err("Rust backend: unproven Decimal constant trait owner".into());
            }
            self.decimal_type(
                actual,
                &CoreType::Named {
                    name: "std/decimal::Decimal".into(),
                    arguments: vec![],
                },
                scope,
            )?;
        }
        self.capabilities.insert("decimal".into());
        Ok(Some(function.into()))
    }

    pub(super) fn validate_decimal_dispatch(&self, expr: &CoreExpr, scope: &Scope) -> Result<()> {
        let CoreExpr::Call {
            arguments,
            evidence,
            trait_dispatch: Some(dispatch),
            type_ref,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            ..
        } = expr
        else {
            return Ok(());
        };
        let Some(selected) = evidence.first() else {
            return Ok(());
        };
        let CoreInstanceEvidence::Standard { identity, .. } = &selected.evidence else {
            return Ok(());
        };
        let (trait_identity, method) = match identity.as_str() {
            "std/decimal::Eq" => ("std/prelude::Eq", "eq"),
            "std/decimal::Hash" => ("std/prelude::Hash", "hash"),
            "std/decimal::Ord" => ("std/prelude::Ord", "compare"),
            "std/decimal::Zero" => ("std/prelude::Zero", "zero"),
            "std/decimal::One" => ("std/prelude::One", "one"),
            _ => return Ok(()),
        };
        if evidence.len() != 1
            || dispatch.trait_identity != trait_identity
            || dispatch.method != method
            || dispatch.method_contract.is_some()
        {
            return Err("Rust backend: Decimal dictionary dispatch mismatch".into());
        }
        if matches!(method, "zero" | "one") {
            if arguments.len() > 1
                || !deferred_evidence_parameters.is_empty()
                || !deferred_evidence_type_constructor_parameters.is_empty()
                || evidence_argument_index.is_some()
            {
                return Err("Rust backend: unproven Decimal constant source stages".into());
            }
            let full = if let Some(argument) = arguments.first() {
                CoreType::Function {
                    parameter: Box::new(
                        local::method_argument_type(argument)
                            .ok_or("Rust backend: unproven Decimal constant Unit argument")?,
                    ),
                    result: Box::new(type_ref.clone()),
                }
            } else {
                type_ref.clone()
            };
            self.validate_decimal_constant_value(dispatch, selected, &full, scope)?;
        } else if method == "compare" {
            if arguments.len() > 2
                || !deferred_evidence_parameters.is_empty()
                || !deferred_evidence_type_constructor_parameters.is_empty()
                || evidence_argument_index.is_some()
            {
                return Err("Rust backend: unproven Decimal Ord source stages".into());
            }
            let full = arguments
                .iter()
                .rev()
                .try_fold(type_ref.clone(), |result, argument| {
                    Ok::<_, RustBackendError>(CoreType::Function {
                        parameter: Box::new(
                            local::method_argument_type(argument)
                                .ok_or("Rust backend: unproven Decimal Ord argument")?,
                        ),
                        result: Box::new(result),
                    })
                })?;
            self.validate_ord_method_value(dispatch, selected, &full, scope)?;
        }
        Ok(())
    }

    pub(super) fn validate_decimal_constant_value(
        &self,
        dispatch: &seseragi_lowering::CoreTraitDispatch,
        selected: &CoreCallEvidence,
        actual: &CoreType,
        scope: &Scope,
    ) -> Result<()> {
        let CoreInstanceEvidence::Standard { identity, .. } = &selected.evidence else {
            return Ok(());
        };
        let (name, method) = match identity.as_str() {
            "std/decimal::Zero" => ("Zero", "zero"),
            "std/decimal::One" => ("One", "one"),
            _ => return Ok(()),
        };
        if dispatch.trait_identity != format!("std/prelude::{name}")
            || dispatch.method != method
            || dispatch.method_contract.is_some()
        {
            return Err("Rust backend: unproven Decimal constant dispatch".into());
        }
        self.decimal_type(
            actual,
            &CoreType::Function {
                parameter: Box::new(CoreType::Named {
                    name: "std/prelude::Unit".into(),
                    arguments: vec![],
                }),
                result: Box::new(CoreType::Named {
                    name: "std/decimal::Decimal".into(),
                    arguments: vec![],
                }),
            },
            scope,
        )
    }
    pub(super) fn decimal_comparison_binary(
        &mut self,
        operator: &str,
        left: &CoreExpr,
        right: &CoreExpr,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if !comparisons::is_comparison(operator) {
            return Ok(None);
        }
        let standard = evidence
            .first()
            .and_then(|selected| match &selected.evidence {
                CoreInstanceEvidence::Standard { identity, .. } => Some(identity.as_str()),
                _ => None,
            });
        if standard != Some("std/decimal::Ord") {
            let decimal_operand = [left, right].iter().any(|argument| {
                let Some(actual) = local::method_argument_type(argument) else { return false; };
                matches!(self.local_canonical_type(&actual, scope), Ok(CoreType::Named { name, arguments }) if name == "std/decimal::Decimal" && arguments.is_empty())
            });
            if (standard.is_some() || evidence.is_empty()) && decimal_operand {
                return Err(
                    "Rust backend: Decimal operand requires its selected canonical Ord provider"
                        .into(),
                );
            }
            return Ok(None);
        }
        let [selected] = evidence else {
            return Err("Rust backend: Decimal comparison dictionary arity".into());
        };
        self.validate_ord_evidence(selected, scope)?;
        self.comparison_result_type(type_ref, 2, evidence, scope)?;
        for argument in [left, right] {
            let actual = local::method_argument_type(argument)
                .ok_or("Rust backend: unproven Decimal comparison argument")?;
            self.decimal_type(
                &actual,
                &CoreType::Named {
                    name: "std/decimal::Decimal".into(),
                    arguments: vec![],
                },
                scope,
            )?;
        }
        let dictionary = self
            .decimal_trait_dictionary(selected, scope)?
            .expect("proved Decimal Ord");
        let stage = self.fresh();
        Ok(Some(format!("{{ let {stage}=({dictionary}).field(\"compare\").apply({}); native_ordering_comparison({}, {stage}.apply({})) }}", self.expr(left,scope)?,quoted(operator),self.expr(right,scope)?)))
    }

    pub(super) fn decimal_show_dictionary(
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
            "Show<std/decimal::Decimal>" => "Show",
            "Debug<std/decimal::Decimal>" => "Debug",
            _ => return Ok(None),
        };
        let [actual] = selected.constraint.arguments.as_slice() else {
            return Err("Rust backend: Decimal Show constraint arity".into());
        };
        if !type_arguments.is_empty()
            || !evidence_arguments.is_empty()
            || !lists::standard_trait(selected, method, actual)
        {
            return Err("Rust backend: unproven Decimal Show evidence".into());
        }
        self.decimal_type(
            actual,
            &CoreType::Named {
                name: "std/decimal::Decimal".into(),
                arguments: vec![],
            },
            scope,
        )?;
        self.capabilities.insert("decimal".into());
        Ok(Some(format!(
            "native_decimal_display_dictionary({})",
            method == "Debug"
        )))
    }
    pub(super) fn validate_decimal_show_value(
        &self,
        dispatch: &seseragi_lowering::CoreTraitDispatch,
        selected: &CoreCallEvidence,
        actual: &CoreType,
        scope: &Scope,
    ) -> Result<()> {
        let method = match &selected.evidence {
            CoreInstanceEvidence::Standard { identity, .. }
                if identity == "Show<std/decimal::Decimal>" =>
            {
                "show"
            }
            CoreInstanceEvidence::Standard { identity, .. }
                if identity == "Debug<std/decimal::Decimal>" =>
            {
                "debug"
            }
            _ => return Ok(()),
        };
        let expected = if method == "show" {
            "std/prelude::Show"
        } else {
            "std/prelude::Debug"
        };
        if dispatch.trait_identity != expected
            || dispatch.method != method
            || dispatch.method_contract.is_some()
        {
            return Err("Rust backend: unproven Decimal display dispatch".into());
        }
        self.decimal_type(
            actual,
            &CoreType::Function {
                parameter: Box::new(CoreType::Named {
                    name: "std/decimal::Decimal".into(),
                    arguments: vec![],
                }),
                result: Box::new(CoreType::Named {
                    name: "std/prelude::String".into(),
                    arguments: vec![],
                }),
            },
            scope,
        )
    }
    pub(super) fn validate_decimal_show_call(&self, expr: &CoreExpr, scope: &Scope) -> Result<()> {
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
        if !evidence.iter().any(|selected| matches!(&selected.evidence,CoreInstanceEvidence::Standard {identity,..} if matches!(identity.as_str(),"Show<std/decimal::Decimal>"|"Debug<std/decimal::Decimal>"))) {return Ok(());}
        let [selected] = evidence.as_slice() else {
            return Err("Rust backend: Decimal Show dictionary arity".into());
        };
        let dispatch = trait_dispatch
            .as_ref()
            .ok_or("Rust backend: missing Decimal Show dispatch")?;
        if arguments.len() > 1
            || !deferred_evidence_parameters.is_empty()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || evidence_argument_index.is_some()
        {
            return Err("Rust backend: unproven Decimal Show source stages".into());
        }
        let full = if let Some(argument) = arguments.first() {
            CoreType::Function {
                parameter: Box::new(
                    local::method_argument_type(argument)
                        .ok_or("Rust backend: missing Decimal Show argument type")?,
                ),
                result: Box::new(type_ref.clone()),
            }
        } else {
            type_ref.clone()
        };
        self.validate_decimal_show_value(dispatch, selected, &full, scope)
    }
    pub(super) fn decimal_type(
        &self,
        actual: &CoreType,
        expected: &CoreType,
        scope: &Scope,
    ) -> Result<()> {
        if self.program_wasm_library {
            return Err("program WASM: native Decimal requires a target-specific ABI proof".into());
        }
        if self.declared_method_type_kind(actual, scope) != Some(0)
            || self.local_canonical_type(actual, scope)? != *expected
        {
            return Err("Rust backend: unproven canonical Decimal argument/result type".into());
        }
        Ok(())
    }

    fn decimal_callable(name: &str, arity: usize) -> String {
        let names: Vec<_> = (0..arity)
            .map(|i| format!("\0decimal_argument_{i}"))
            .collect();
        let arguments = names
            .iter()
            .map(|name| format!("{}.clone()", id(name)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = format!(
            "native_decimal_standard_operation({}, vec![{arguments}])",
            quoted(name)
        );
        for i in (0..arity).rev() {
            body = Self::closure(&names[i], &body, &names[..i].iter().cloned().collect());
        }
        body
    }

    pub(super) fn decimal_reference(
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
        if !evidence.is_empty() {
            return Err("Rust backend: Decimal operation with unsupported evidence".into());
        }
        self.decimal_type(type_ref, &remaining_type(&parameters, &result), scope)?;
        self.capabilities.insert("decimal".into());
        Ok(Some(Self::decimal_callable(name, parameters.len())))
    }

    pub(super) fn decimal_call(
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
        if arguments.len() > parameters.len()
            || !evidence.is_empty()
            || trait_dispatch.is_some()
            || !deferred_evidence_parameters.is_empty()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || evidence_argument_index.is_some()
        {
            return Err("Rust backend: unproven Decimal source application".into());
        }
        for (argument, expected) in arguments.iter().zip(&parameters) {
            let actual = local::method_argument_type(argument)
                .ok_or("Rust backend: missing Decimal argument type")?;
            self.decimal_type(&actual, expected, scope)?;
        }
        self.decimal_type(
            type_ref,
            &remaining_type(&parameters[arguments.len()..], &result),
            scope,
        )?;
        self.capabilities.insert("decimal".into());
        if arguments.len() == parameters.len() {
            self.direct_calls += 1;
            return Ok(Some(format!(
                "native_decimal_standard_operation({}, vec![{}])",
                quoted(callee),
                self.expressions(arguments, scope)?
            )));
        }
        let value = self.fresh();
        let mut output = format!(
            "{{ let mut {value} = {};",
            Self::decimal_callable(callee, parameters.len())
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
