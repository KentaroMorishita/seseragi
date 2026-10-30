use super::*;

fn signature(name: &str) -> Option<(Vec<CoreType>, CoreType)> {
    let named = |name: &str, arguments| CoreType::Named {
        name: name.into(),
        arguments,
    };
    let big = named("std/big-int::BigInt", vec![]);
    let text = named("std/prelude::String", vec![]);
    let int = named("std/prelude::Int", vec![]);
    let error = named("std/big-int::BigIntParseError", vec![]);
    Some(match name {
        "std/big-int::parse" => (
            vec![text],
            named(
                "std/prelude::Either",
                vec![named("std/big-int::BigIntParseError", vec![]), big],
            ),
        ),
        "std/big-int::format" => (vec![big], text),
        "std/big-int::fromInt" => (vec![int], big),
        "std/big-int::parseRadix" => (
            vec![int, text],
            named("std/prelude::Either", vec![error, big]),
        ),
        "std/big-int::formatRadix" => (
            vec![int, big],
            named("std/prelude::Either", vec![error, text]),
        ),
        "std/big-int::toInt" => (
            vec![big],
            named(
                "std/prelude::Either",
                vec![named("std/big-int::BigIntConversionError", vec![]), int],
            ),
        ),
        "std/big-int::EmptyBigInt" => (vec![], error),
        "std/big-int::BigIntOutsideIntRange" => {
            (vec![], named("std/big-int::BigIntConversionError", vec![]))
        }
        "std/big-int::InvalidBigIntRadix" => (vec![int], error),
        "std/big-int::InvalidBigIntDigit" => (
            vec![CoreType::Record {
                closed: true,
                fields: ["offset", "radix"]
                    .into_iter()
                    .map(|name| seseragi_lowering::CoreRecordField {
                        name: name.into(),
                        optional: false,
                        type_ref: int.clone(),
                    })
                    .collect(),
            }],
            error,
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
    fn big_int_type(&self, actual: &CoreType, expected: &CoreType, scope: &Scope) -> Result<()> {
        if self.program_wasm_library {
            return Err("program WASM: native BigInt requires a target-specific ABI proof".into());
        }
        if self.declared_method_type_kind(actual, scope) != Some(0)
            || self.local_canonical_type(actual, scope)? != *expected
        {
            return Err("Rust backend: unproven canonical BigInt argument/result type".into());
        }
        Ok(())
    }

    fn big_int_callable(name: &str, arity: usize) -> String {
        let names: Vec<_> = (0..arity)
            .map(|i| format!("\0big_int_argument_{i}"))
            .collect();
        let arguments = names
            .iter()
            .map(|name| format!("{}.clone()", id(name)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = format!(
            "native_big_int_standard_operation({}, vec![{arguments}])",
            quoted(name)
        );
        for i in (0..arity).rev() {
            body = Self::closure(&names[i], &body, &names[..i].iter().cloned().collect());
        }
        body
    }

    pub(super) fn big_int_reference(
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
            return Err("Rust backend: BigInt operation with unsupported evidence".into());
        }
        self.big_int_type(type_ref, &remaining_type(&parameters, &result), scope)?;
        self.capabilities.insert("big-int".into());
        Ok(Some(Self::big_int_callable(name, parameters.len())))
    }

    pub(super) fn big_int_call(
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
            return Err("Rust backend: unproven BigInt source application".into());
        }
        for (argument, expected) in arguments.iter().zip(&parameters) {
            let actual = local::method_argument_type(argument)
                .ok_or("Rust backend: missing BigInt argument type")?;
            self.big_int_type(&actual, expected, scope)?;
        }
        self.big_int_type(
            type_ref,
            &remaining_type(&parameters[arguments.len()..], &result),
            scope,
        )?;
        self.capabilities.insert("big-int".into());
        if arguments.len() == parameters.len() {
            self.direct_calls += 1;
            return Ok(Some(format!(
                "native_big_int_standard_operation({}, vec![{}])",
                quoted(callee),
                self.expressions(arguments, scope)?
            )));
        }
        let value = self.fresh();
        let mut output = format!(
            "{{ let mut {value} = {};",
            Self::big_int_callable(callee, parameters.len())
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
