use super::*;

fn regex_signature(name: &str) -> Option<(Vec<CoreType>, CoreType)> {
    let ty = |name: &str| CoreType::Named {
        name: name.into(),
        arguments: vec![],
    };
    let string = ty("std/prelude::String");
    let regex = ty("std/regex::Regex");
    let compiled = CoreType::Named {
        name: "std/prelude::Either".into(),
        arguments: vec![ty("std/regex::RegexCompileError"), regex.clone()],
    };
    Some(match name {
        "std/regex::defaultOptions" => {
            (vec![ty("std/prelude::Unit")], ty("std/regex::RegexOptions"))
        }
        "std/regex::compile" => (vec![string], compiled),
        "std/regex::compileWith" => (vec![ty("std/regex::RegexOptions"), string], compiled),
        "std/regex::isMatch" => (vec![regex, string], ty("std/prelude::Bool")),
        "std/regex::find" => (
            vec![regex, string],
            CoreType::Named {
                name: "std/prelude::Maybe".into(),
                arguments: vec![ty("std/regex::RegexMatch")],
            },
        ),
        "std/regex::findAll" => (
            vec![regex, string],
            CoreType::Named {
                name: "std/prelude::Array".into(),
                arguments: vec![ty("std/regex::RegexMatch")],
            },
        ),
        "std/regex::split" => (
            vec![regex, string.clone()],
            CoreType::Named {
                name: "std/prelude::Array".into(),
                arguments: vec![string],
            },
        ),
        "std/regex::replaceAll" => (vec![regex, string.clone(), string.clone()], string),
        "std/regex::replaceAllWith" => (
            vec![
                regex,
                CoreType::Function {
                    parameter: Box::new(ty("std/regex::RegexMatch")),
                    result: Box::new(string.clone()),
                },
                string.clone(),
            ],
            string,
        ),
        "std/regex::escape" => (vec![string.clone()], string),
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
    fn regex_target(&self, _name: &str) -> Result<()> {
        if self.program_wasm_library {
            return Err("program WASM: native Regex requires a target-specific ABI proof".into());
        }
        Ok(())
    }

    fn regex_callable(name: &str, arity: usize) -> String {
        let names: Vec<_> = (0..arity)
            .map(|i| format!("\0regex_argument_{i}"))
            .collect();
        let arguments = names
            .iter()
            .map(|n| format!("{}.clone()", id(n)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = format!(
            "native_regex_operation({}, vec![{arguments}])",
            quoted(name)
        );
        for i in (0..arity).rev() {
            body = Self::closure(&names[i], &body, &names[..i].iter().cloned().collect());
        }
        body
    }

    fn regex_type_matches(
        &self,
        actual: &CoreType,
        expected: &CoreType,
        scope: &Scope,
    ) -> Result<()> {
        if self.declared_method_type_kind(actual, scope) != Some(0)
            || self.local_canonical_type(actual, scope)? != *expected
        {
            return Err("Rust backend: unproven canonical Regex argument/result type".into());
        }
        Ok(())
    }

    pub(super) fn regex_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let Some((parameters, result)) = regex_signature(name) else {
            return Ok(None);
        };
        if scope.contains(name) {
            return Ok(None);
        }
        self.regex_target(name)?;
        if !evidence.is_empty() {
            return Err("Rust backend: Regex operation with unsupported evidence".into());
        }
        self.regex_type_matches(type_ref, &remaining_type(&parameters, &result), scope)?;
        self.capabilities.insert("regex".into());
        Ok(Some(Self::regex_callable(name, parameters.len())))
    }

    pub(super) fn regex_call(&mut self, expr: &CoreExpr, scope: &Scope) -> Result<Option<String>> {
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
        let Some((parameters, result)) = regex_signature(callee) else {
            return Ok(None);
        };
        if scope.contains(callee) {
            return Ok(None);
        }
        self.regex_target(callee)?;
        if arguments.len() > parameters.len()
            || !evidence.is_empty()
            || trait_dispatch.is_some()
            || !deferred_evidence_parameters.is_empty()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || evidence_argument_index.is_some()
        {
            return Err("Rust backend: unproven Regex source application".into());
        }
        for (argument, expected) in arguments.iter().zip(&parameters) {
            let actual = local::method_argument_type(argument)
                .ok_or("Rust backend: missing Regex argument type")?;
            self.regex_type_matches(&actual, expected, scope)?;
        }
        self.regex_type_matches(
            type_ref,
            &remaining_type(&parameters[arguments.len()..], &result),
            scope,
        )?;
        self.capabilities.insert("regex".into());
        if arguments.len() == parameters.len() {
            self.direct_calls += 1;
            return Ok(Some(format!(
                "native_regex_operation({}, vec![{}])",
                quoted(callee),
                self.expressions(arguments, scope)?
            )));
        }
        let value = self.fresh();
        let mut output = format!(
            "{{ let mut {value} = {};",
            Self::regex_callable(callee, parameters.len())
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
