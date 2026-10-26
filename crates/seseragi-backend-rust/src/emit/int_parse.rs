use super::*;

pub(super) fn constructor_slots(name: &str) -> Option<Vec<u32>> {
    // This concrete standard error owner is part of the admitted parse ABI.
    // It has no type parameters; applied prefixes remain kind-checked normally.
    (name == "std/int::IntParseError").then(Vec::new)
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
    Some(match name {
        "std/int::parse" => (
            vec![named("std/prelude::String")],
            CoreType::Named {
                name: "std/prelude::Either".into(),
                arguments: vec![error, named("std/prelude::Int")],
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
            self.int_parse_type_matches(&actual, &parameter, scope)?;
        }
        // A saved method keeps its one source argument. A saturated call must
        // produce String; dictionary selection never consumes that argument.
        let parameters = [parameter];
        self.int_parse_type_matches(
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

    fn int_parse_callable(name: &str, arity: usize) -> String {
        let names: Vec<_> = (0..arity)
            .map(|i| format!("\0int_parse_argument_{i}"))
            .collect();
        let arguments = names
            .iter()
            .map(|n| format!("{}.clone()", id(n)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = format!(
            "native_int_parse_operation({}, vec![{arguments}])",
            quoted(name)
        );
        for i in (0..arity).rev() {
            body = Self::closure(&names[i], &body, &names[..i].iter().cloned().collect());
        }
        body
    }

    fn int_parse_type_matches(
        &self,
        actual: &CoreType,
        expected: &CoreType,
        scope: &Scope,
    ) -> Result<()> {
        if self.declared_method_type_kind(actual, scope) != Some(0)
            || self.local_canonical_type(actual, scope)? != *expected
        {
            return Err("Rust backend: unproven canonical Int parse argument/result type".into());
        }
        Ok(())
    }

    pub(super) fn int_parse_reference(
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
            return Err("Rust backend: Int parse operation with unsupported evidence".into());
        }
        self.int_parse_type_matches(type_ref, &remaining_type(&parameters, &result), scope)?;
        self.capabilities.insert("int-parse".into());
        Ok(Some(Self::int_parse_callable(name, parameters.len())))
    }

    pub(super) fn int_parse_call(
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
            return Err("Rust backend: unproven Int parse source application".into());
        }
        for (argument, expected) in arguments.iter().zip(&parameters) {
            let actual = local::method_argument_type(argument)
                .ok_or("Rust backend: missing Int parse argument type")?;
            self.int_parse_type_matches(&actual, expected, scope)?;
        }
        self.int_parse_type_matches(
            type_ref,
            &remaining_type(&parameters[arguments.len()..], &result),
            scope,
        )?;
        self.capabilities.insert("int-parse".into());
        if arguments.len() == parameters.len() {
            self.direct_calls += 1;
            return Ok(Some(format!(
                "native_int_parse_operation({}, vec![{}])",
                quoted(callee),
                self.expressions(arguments, scope)?
            )));
        }
        let value = self.fresh();
        let mut output = format!(
            "{{ let mut {value} = {};",
            Self::int_parse_callable(callee, parameters.len())
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
