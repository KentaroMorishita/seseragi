use super::*;

fn primitive_operator(identity: &str) -> Option<(&'static str, &'static str, &'static str)> {
    Some(match identity {
        "std/int::Add" => ("Add", "add", "Int"),
        "std/int::Sub" => ("Sub", "sub", "Int"),
        "std/int::Mul" => ("Mul", "mul", "Int"),
        "std/int::Div" => ("Div", "div", "Int"),
        "std/int::Rem" => ("Rem", "rem", "Int"),
        "std/int::Pow" => ("Pow", "pow", "Int"),
        "std/float::Pow" => ("Pow", "pow", "Float"),
        "std/big-int::Add" => ("Add", "add", "BigInt"),
        "std/big-int::Sub" => ("Sub", "sub", "BigInt"),
        "std/big-int::Mul" => ("Mul", "mul", "BigInt"),
        "std/big-int::Div" => ("Div", "div", "BigInt"),
        "std/big-int::Rem" => ("Rem", "rem", "BigInt"),
        "std/big-int::Pow" => ("Pow", "pow", "BigInt"),
        "std/decimal::Add" => ("Add", "add", "Decimal"),
        "std/decimal::Sub" => ("Sub", "sub", "Decimal"),
        "std/decimal::Mul" => ("Mul", "mul", "Decimal"),
        "std/string::Add" => ("Add", "add", "String"),
        _ => return None,
    })
}

pub(super) fn big_int_binary(evidence: &[CoreCallEvidence], operator: &str) -> bool {
    matches!(evidence, [CoreCallEvidence { evidence: CoreInstanceEvidence::Standard { identity, .. }, .. }] if identity.starts_with("std/big-int::"))
        && primitive_reference(operator, evidence).is_some()
}

pub(super) fn decimal_binary(evidence: &[CoreCallEvidence], operator: &str) -> bool {
    matches!(evidence, [CoreCallEvidence { evidence: CoreInstanceEvidence::Standard { identity, .. }, .. }] if identity.starts_with("std/decimal::"))
        && primitive_reference(operator, evidence).is_some()
}

pub(super) fn primitive_reference(
    operator: &str,
    evidence: &[CoreCallEvidence],
) -> Option<&'static str> {
    let [selected] = evidence else { return None };
    let CoreInstanceEvidence::Standard { identity, .. } = &selected.evidence else {
        return None;
    };
    let (_, method, _) = primitive_operator(identity)?;
    (matches!(
        (operator, method),
        ("+", "add") | ("-", "sub") | ("*", "mul") | ("/", "div") | ("%", "rem") | ("**", "pow")
    ))
    .then_some(method)
}

pub(super) fn valid_dispatch(selected: &CoreCallEvidence, identity: &str, method: &str) -> bool {
    let CoreInstanceEvidence::Standard {
        identity: provider, ..
    } = &selected.evidence
    else {
        return true;
    };
    let Some((trait_name, expected, _)) = primitive_operator(provider) else {
        return true;
    };
    identity == format!("std/prelude::{trait_name}") && method == expected
}

impl Backend<'_> {
    pub(super) fn validate_opaque_numeric_operator_provider(
        &self,
        operator: &str,
        operands: [&CoreExpr; 2],
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<()> {
        if !matches!(operator, "+" | "-" | "*" | "/" | "%" | "**")
            || big_int_binary(evidence, operator)
            || decimal_binary(evidence, operator)
        {
            return Ok(());
        }
        if (evidence.is_empty() || evidence.iter().any(|e| matches!(e.evidence, CoreInstanceEvidence::Standard { .. })))
            && operands.iter().any(|argument| {
                let Some(actual) = local::method_argument_type(argument) else { return false; };
                matches!(self.local_canonical_type(&actual, scope), Ok(CoreType::Named { name, arguments }) if matches!(name.as_str(), "std/big-int::BigInt" | "std/decimal::Decimal") && arguments.is_empty())
            }) {
            return Err("Rust backend: opaque numeric arithmetic requires its canonical selected provider".into());
        }
        Ok(())
    }

    pub(super) fn validate_primitive_operator_value(
        &self,
        dispatch: &seseragi_lowering::CoreTraitDispatch,
        selected: &CoreCallEvidence,
        actual: &CoreType,
        scope: &Scope,
    ) -> Result<()> {
        let Some(dictionary) = self.primitive_operator_dictionary(selected, scope) else {
            return Ok(());
        };
        dictionary?;
        if !valid_dispatch(selected, &dispatch.trait_identity, &dispatch.method)
            || dispatch.method_contract.is_some()
        {
            return Err("Rust backend: unproven canonical arithmetic method value".into());
        }
        self.validate_primitive_operator_result(selected, 0, actual, scope)
    }

    pub(super) fn validate_primitive_operator_binary(
        &self,
        operator: &str,
        operands: [&CoreExpr; 2],
        evidence: &[CoreCallEvidence],
        result_type: &CoreType,
        scope: &Scope,
    ) -> Result<()> {
        let [selected] = evidence else {
            return Err("Rust backend: primitive arithmetic requires one dictionary".into());
        };
        if primitive_reference(operator, evidence).is_none() {
            return Err("Rust backend: primitive arithmetic operator/provider mismatch".into());
        }
        self.validate_primitive_operator_result(selected, 2, result_type, scope)?;
        for (expected, operand) in selected.constraint.arguments.iter().zip(operands) {
            let actual = local::method_argument_type(operand)
                .ok_or("Rust backend: unproven primitive arithmetic operand type")?;
            self.match_local_type(
                expected,
                &actual,
                &BTreeMap::new(),
                &mut BTreeMap::new(),
                scope,
            )?;
        }
        Ok(())
    }

    pub(super) fn primitive_operator_dictionary(
        &self,
        selected: &CoreCallEvidence,
        scope: &Scope,
    ) -> Option<Result<String>> {
        let CoreInstanceEvidence::Standard {
            identity,
            type_arguments,
            evidence_arguments,
        } = &selected.evidence
        else {
            return None;
        };
        let (trait_name, method, primitive) = primitive_operator(identity)?;
        Some((|| {
            if self.program_wasm_library
                && matches!(
                    identity.as_str(),
                    "std/int::Div" | "std/int::Rem" | "std/int::Pow" | "std/float::Pow"
                )
            {
                let operation = match identity.as_str() {
                    "std/int::Div" => "division",
                    "std/int::Rem" => "remainder",
                    _ => "power",
                };
                return Err(format!(
                    "program WASM: native {primitive} {operation} requires a target-specific arithmetic proof"
                )
                .into());
            }
            if selected.constraint.name != trait_name
                || selected
                    .constraint
                    .trait_identity
                    .as_deref()
                    .is_some_and(|identity| identity != format!("std/prelude::{trait_name}"))
                || selected.constraint.arguments.len() != 3
                || !type_arguments.is_empty()
                || !evidence_arguments.is_empty()
            {
                return Err(
                    "Rust backend: invalid canonical arithmetic dictionary evidence".into(),
                );
            }
            for (index, ty) in selected.constraint.arguments.iter().enumerate() {
                if primitive == "Decimal" {
                    self.decimal_type(
                        ty,
                        &CoreType::Named {
                            name: "std/decimal::Decimal".into(),
                            arguments: vec![],
                        },
                        scope,
                    )?;
                    continue;
                }
                if primitive == "BigInt" {
                    let expected = if method == "pow" && index == 1 {
                        "std/prelude::Int"
                    } else {
                        "std/big-int::BigInt"
                    };
                    self.big_int_type(
                        ty,
                        &CoreType::Named {
                            name: expected.into(),
                            arguments: vec![],
                        },
                        scope,
                    )?;
                    continue;
                }
                if !matches!(self.local_canonical_type(ty, scope)?, CoreType::Named { name, arguments } if name == format!("std/prelude::{primitive}") && arguments.is_empty())
                {
                    return Err("Rust backend: primitive arithmetic cannot stand for a nominal or scoped binder".into());
                }
            }
            let factory = match primitive {
                "Float" => "native_float_dictionary",
                "BigInt" => "native_big_int_arithmetic_dictionary",
                "Decimal" => "native_decimal_arithmetic_dictionary",
                _ => "native_operator_dictionary",
            };
            Ok(format!("{factory}({})", quoted(identity)))
        })())
    }

    pub(super) fn validate_primitive_operator_result(
        &self,
        selected: &CoreCallEvidence,
        supplied: usize,
        result_type: &CoreType,
        scope: &Scope,
    ) -> Result<()> {
        let Some(dictionary) = self.primitive_operator_dictionary(selected, scope) else {
            return Ok(());
        };
        dictionary?;
        if supplied > 2 {
            return Err("Rust backend: primitive arithmetic source arity mismatch".into());
        }
        let mut result = result_type;
        for expected in selected.constraint.arguments.iter().take(2).skip(supplied) {
            let CoreType::Function {
                parameter,
                result: next,
            } = result
            else {
                return Err("Rust backend: primitive arithmetic result stage mismatch".into());
            };
            self.match_local_type(
                expected,
                parameter,
                &BTreeMap::new(),
                &mut BTreeMap::new(),
                scope,
            )?;
            result = next;
        }
        self.match_local_type(
            &selected.constraint.arguments[2],
            result,
            &BTreeMap::new(),
            &mut BTreeMap::new(),
            scope,
        )?;
        Ok(())
    }

    pub(super) fn validate_primitive_operator_call(
        &self,
        expr: &CoreExpr,
        scope: &Scope,
    ) -> Result<()> {
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
        let Some(dictionary) = self.primitive_operator_dictionary(selected, scope) else {
            return Ok(());
        };
        dictionary?;
        if evidence.len() != 1
            || !valid_dispatch(selected, &dispatch.trait_identity, &dispatch.method)
            || dispatch.method_contract.is_some()
            || !deferred_evidence_parameters.is_empty()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || evidence_argument_index.is_some()
        {
            return Err("Rust backend: unproven primitive arithmetic dispatch or staging".into());
        }
        self.validate_primitive_operator_result(selected, arguments.len(), type_ref, scope)?;
        for (expected, argument) in selected.constraint.arguments.iter().zip(arguments) {
            let actual = local::method_argument_type(argument)
                .ok_or("Rust backend: unproven primitive arithmetic argument type")?;
            self.match_local_type(
                expected,
                &actual,
                &BTreeMap::new(),
                &mut BTreeMap::new(),
                scope,
            )?;
        }
        Ok(())
    }
}
