use super::*;

fn is_math(name: &str) -> bool {
    matches!(
        name,
        "std/math::sqrt" | "std/math::sin" | "std/math::cos" | "std/math::log"
    )
}

fn float() -> CoreType {
    CoreType::Named {
        name: "std/prelude::Float".into(),
        arguments: vec![],
    }
}

impl Backend<'_> {
    fn math_type(&self, actual: &CoreType, supplied: usize, scope: &Scope) -> Result<()> {
        if self.program_wasm_library {
            return Err(
                "program WASM: native math requires a target-specific arithmetic proof".into(),
            );
        }
        let expected = if supplied == 0 {
            CoreType::Function {
                parameter: Box::new(float()),
                result: Box::new(float()),
            }
        } else {
            float()
        };
        if self.declared_method_type_kind(actual, scope) != Some(0)
            || self.local_canonical_type(actual, scope)? != expected
        {
            return Err("Rust backend: unproven canonical math argument/result type".into());
        }
        Ok(())
    }

    fn math_callable(name: &str) -> String {
        format!(
            "V::fun(vec![], |_, value| native_math_operation({}, value))",
            quoted(name)
        )
    }

    pub(super) fn math_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if !is_math(name) || scope.contains(name) {
            return Ok(None);
        }
        if !evidence.is_empty() {
            return Err("Rust backend: math operation with unsupported evidence".into());
        }
        self.math_type(type_ref, 0, scope)?;
        self.capabilities.insert("math".into());
        Ok(Some(Self::math_callable(name)))
    }

    pub(super) fn math_call(&mut self, expr: &CoreExpr, scope: &Scope) -> Result<Option<String>> {
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
        if !is_math(callee) || scope.contains(callee) {
            return Ok(None);
        }
        if arguments.len() > 1
            || !evidence.is_empty()
            || trait_dispatch.is_some()
            || !deferred_evidence_parameters.is_empty()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || evidence_argument_index.is_some()
        {
            return Err("Rust backend: unproven math source application".into());
        }
        self.math_type(type_ref, arguments.len(), scope)?;
        self.capabilities.insert("math".into());
        if let Some(argument) = arguments.first() {
            let actual = local::method_argument_type(argument)
                .ok_or("Rust backend: missing math argument type")?;
            self.math_type(&actual, 1, scope)?;
            self.direct_calls += 1;
            Ok(Some(format!(
                "native_math_operation({}, {})",
                quoted(callee),
                self.expr(argument, scope)?
            )))
        } else {
            Ok(Some(Self::math_callable(callee)))
        }
    }
}
