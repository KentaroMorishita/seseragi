use super::*;

fn text_signature(name: &str) -> Option<(Vec<CoreType>, CoreType)> {
    let string = CoreType::Named {
        name: "std/prelude::String".into(),
        arguments: vec![],
    };
    let strings = CoreType::Named {
        name: "std/prelude::Array".into(),
        arguments: vec![string.clone()],
    };
    let boolean = CoreType::Named {
        name: "std/prelude::Bool".into(),
        arguments: vec![],
    };
    match name {
        "std/text::lines" => Some((vec![string], strings)),
        "std/text::join" => Some((vec![string.clone(), strings], string)),
        "std/text::split" => Some((vec![string.clone(), string], strings)),
        "std/text::trim" => Some((vec![string.clone()], string)),
        "std/text::contains" | "std/text::startsWith" | "std/text::endsWith" => {
            Some((vec![string.clone(), string], boolean))
        }
        "std/text::replace" => Some((vec![string.clone(); 3], string)),
        _ => None,
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
    fn text_target(&self, name: &str) -> Result<()> {
        if self.program_wasm_library && !matches!(name, "std/text::lines" | "std/text::join") {
            return Err(
                "program WASM: native text extension requires a target-specific proof".into(),
            );
        }
        Ok(())
    }

    fn text_callable(name: &str, arity: usize) -> String {
        let names: Vec<_> = (0..arity).map(|i| format!("\0text_argument_{i}")).collect();
        let arguments = names
            .iter()
            .map(|n| format!("{}.clone()", id(n)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = format!("native_text_operation({}, vec![{arguments}])", quoted(name));
        for i in (0..arity).rev() {
            body = Self::closure(&names[i], &body, &names[..i].iter().cloned().collect());
        }
        body
    }

    fn text_type_matches(
        &self,
        actual: &CoreType,
        expected: &CoreType,
        scope: &Scope,
    ) -> Result<()> {
        if self.declared_method_type_kind(actual, scope) != Some(0)
            || self.local_canonical_type(actual, scope)? != *expected
        {
            return Err("Rust backend: unproven canonical text argument/result type".into());
        }
        Ok(())
    }

    pub(super) fn text_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let Some((parameters, result)) = text_signature(name) else {
            return Ok(None);
        };
        if scope.contains(name) {
            return Ok(None);
        }
        self.text_target(name)?;
        if !evidence.is_empty() {
            return Err("Rust backend: text operation with unsupported evidence".into());
        }
        self.text_type_matches(type_ref, &remaining_type(&parameters, &result), scope)?;
        self.capabilities.insert("text".into());
        Ok(Some(Self::text_callable(name, parameters.len())))
    }

    pub(super) fn text_call(&mut self, expr: &CoreExpr, scope: &Scope) -> Result<Option<String>> {
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
        let Some((parameters, result)) = text_signature(callee) else {
            return Ok(None);
        };
        if scope.contains(callee) {
            return Ok(None);
        }
        self.text_target(callee)?;
        if arguments.len() > parameters.len()
            || !evidence.is_empty()
            || trait_dispatch.is_some()
            || !deferred_evidence_parameters.is_empty()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || evidence_argument_index.is_some()
        {
            return Err("Rust backend: unproven text source application".into());
        }
        for (argument, expected) in arguments.iter().zip(&parameters) {
            let actual = local::method_argument_type(argument)
                .ok_or("Rust backend: missing text argument type")?;
            self.text_type_matches(&actual, expected, scope)?;
        }
        self.text_type_matches(
            type_ref,
            &remaining_type(&parameters[arguments.len()..], &result),
            scope,
        )?;
        self.capabilities.insert("text".into());
        if arguments.len() == parameters.len() {
            self.direct_calls += 1;
            return Ok(Some(format!(
                "native_text_operation({}, vec![{}])",
                quoted(callee),
                self.expressions(arguments, scope)?
            )));
        }
        let value = self.fresh();
        let mut output = format!(
            "{{ let mut {value} = {};",
            Self::text_callable(callee, parameters.len())
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
