use super::*;

fn signature(name: &str) -> Option<(CoreType, CoreType)> {
    let named = |name: &str, arguments| CoreType::Named {
        name: name.into(),
        arguments,
    };
    let big = named("std/big-int::BigInt", vec![]);
    let text = named("std/prelude::String", vec![]);
    Some(match name {
        "std/big-int::parse" => (
            text,
            named(
                "std/prelude::Either",
                vec![named("std/big-int::BigIntParseError", vec![]), big],
            ),
        ),
        "std/big-int::format" => (big, text),
        "std/big-int::fromInt" => (named("std/prelude::Int", vec![]), big),
        _ => return None,
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

    fn big_int_callable(name: &str) -> String {
        format!(
            "V::fun(vec![], |_, value| native_big_int_operation({}, value))",
            quoted(name)
        )
    }

    pub(super) fn big_int_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let Some((parameter, result)) = signature(name) else {
            return Ok(None);
        };
        if scope.contains(name) {
            return Ok(None);
        }
        if !evidence.is_empty() {
            return Err("Rust backend: BigInt operation with unsupported evidence".into());
        }
        self.big_int_type(
            type_ref,
            &CoreType::Function {
                parameter: Box::new(parameter),
                result: Box::new(result),
            },
            scope,
        )?;
        self.capabilities.insert("big-int".into());
        Ok(Some(Self::big_int_callable(name)))
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
        let Some((parameter, result)) = signature(callee) else {
            return Ok(None);
        };
        if scope.contains(callee) {
            return Ok(None);
        }
        if arguments.len() > 1
            || !evidence.is_empty()
            || trait_dispatch.is_some()
            || !deferred_evidence_parameters.is_empty()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || evidence_argument_index.is_some()
        {
            return Err("Rust backend: unproven BigInt source application".into());
        }
        let Some(argument) = arguments.first() else {
            return self.big_int_reference(callee, evidence, type_ref, scope);
        };
        let actual = local::method_argument_type(argument)
            .ok_or("Rust backend: missing BigInt argument type")?;
        self.big_int_type(&actual, &parameter, scope)?;
        self.big_int_type(type_ref, &result, scope)?;
        self.capabilities.insert("big-int".into());
        self.direct_calls += 1;
        Ok(Some(format!(
            "native_big_int_operation({}, {})",
            quoted(callee),
            self.expr(argument, scope)?
        )))
    }
}
