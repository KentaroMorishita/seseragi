use super::*;

fn primitive_operator(identity: &str) -> Option<(&'static str, &'static str, &'static str)> {
    Some(match identity {
        "std/int::Add" => ("Add", "add", "Int"),
        "std/int::Sub" => ("Sub", "sub", "Int"),
        "std/int::Mul" => ("Mul", "mul", "Int"),
        "std/int::Div" => ("Div", "div", "Int"),
        "std/string::Add" => ("Add", "add", "String"),
        _ => return None,
    })
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
        ("+", "add") | ("-", "sub") | ("*", "mul") | ("/", "div")
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
        let (trait_name, _, primitive) = primitive_operator(identity)?;
        Some((|| {
            if self.program_wasm_library && identity == "std/int::Div" {
                return Err(
                    "program WASM: native Int division requires a target-specific arithmetic proof"
                        .into(),
                );
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
            for ty in &selected.constraint.arguments {
                if !matches!(self.local_canonical_type(ty, scope)?, CoreType::Named { name, arguments } if name == format!("std/prelude::{primitive}") && arguments.is_empty())
                {
                    return Err("Rust backend: primitive arithmetic cannot stand for a nominal or scoped binder".into());
                }
            }
            Ok(format!("native_operator_dictionary({})", quoted(identity)))
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
