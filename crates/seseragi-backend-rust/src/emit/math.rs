use super::*;

fn math_arity(name: &str) -> Option<usize> {
    Some(match name {
        "std/math::pi" | "std/math::e" | "std/math::tau" => 0,
        "std/math::sqrt" | "std/math::sin" | "std/math::cos" | "std/math::log"
        | "std/math::tan" | "std/math::asin" | "std/math::acos" | "std/math::atan"
        | "std/math::exp" | "std/math::log2" | "std/math::log10" | "std/math::cbrt"
        | "std/math::sinh" | "std/math::cosh" | "std/math::tanh" => 1,
        "std/math::atan2" | "std/math::hypot" => 2,
        _ => return None,
    })
}

fn float() -> CoreType {
    CoreType::Named {
        name: "std/prelude::Float".into(),
        arguments: vec![],
    }
}

impl Backend<'_> {
    fn math_type(
        &self,
        actual: &CoreType,
        supplied: usize,
        arity: usize,
        scope: &Scope,
    ) -> Result<()> {
        if self.program_wasm_library {
            return Err(
                "program WASM: native math requires a target-specific arithmetic proof".into(),
            );
        }
        let expected = (supplied..arity).fold(float(), |result, _| CoreType::Function {
            parameter: Box::new(float()),
            result: Box::new(result),
        });
        if self.declared_method_type_kind(actual, scope) != Some(0)
            || self.local_canonical_type(actual, scope)? != expected
        {
            return Err("Rust backend: unproven canonical math argument/result type".into());
        }
        Ok(())
    }

    fn math_callable(name: &str, arity: usize) -> String {
        let names: Vec<_> = (0..arity).map(|i| format!("\0math_argument_{i}")).collect();
        let arguments = names
            .iter()
            .map(|name| format!("{}.clone()", id(name)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = format!(
            "native_math_standard_operation({}, vec![{arguments}])",
            quoted(name)
        );
        for i in (0..arity).rev() {
            body = Self::closure(&names[i], &body, &names[..i].iter().cloned().collect());
        }
        body
    }

    pub(super) fn math_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let Some(arity) = math_arity(name) else {
            return Ok(None);
        };
        if scope.contains(name) {
            return Ok(None);
        }
        if !evidence.is_empty() {
            return Err("Rust backend: math operation with unsupported evidence".into());
        }
        self.math_type(type_ref, 0, arity, scope)?;
        self.capabilities.insert("math".into());
        Ok(Some(Self::math_callable(name, arity)))
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
        let Some(arity) = math_arity(callee) else {
            return Ok(None);
        };
        if scope.contains(callee) {
            return Ok(None);
        }
        if arguments.len() > arity
            || !evidence.is_empty()
            || trait_dispatch.is_some()
            || !deferred_evidence_parameters.is_empty()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || evidence_argument_index.is_some()
        {
            return Err("Rust backend: unproven math source application".into());
        }
        self.math_type(type_ref, arguments.len(), arity, scope)?;
        self.capabilities.insert("math".into());
        for argument in arguments {
            let actual = local::method_argument_type(argument)
                .ok_or("Rust backend: missing math argument type")?;
            self.math_type(&actual, arity, arity, scope)?;
        }
        if arguments.len() == arity {
            self.direct_calls += 1;
            Ok(Some(format!(
                "native_math_standard_operation({}, vec![{}])",
                quoted(callee),
                self.expressions(arguments, scope)?
            )))
        } else {
            let value = self.fresh();
            let mut output = format!(
                "{{ let mut {value} = {};",
                Self::math_callable(callee, arity)
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
}
