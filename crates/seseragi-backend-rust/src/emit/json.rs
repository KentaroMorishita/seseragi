use super::*;

fn signature(name: &str) -> Option<(Vec<CoreType>, CoreType)> {
    let named = |name: &str, arguments| CoreType::Named {
        name: name.into(),
        arguments,
    };
    let json = named("std/json::Json", vec![]);
    let path = named("std/json::JsonPathSegment", vec![]);
    let error = named("std/json::JsonParseError", vec![]);
    let text = named("std/prelude::String", vec![]);
    let int = named("std/prelude::Int", vec![]);
    let array = |value| named("std/prelude::Array", vec![value]);
    let record = |fields: Vec<(&str, CoreType)>| CoreType::Record {
        closed: true,
        fields: fields
            .into_iter()
            .map(|(name, type_ref)| seseragi_lowering::CoreRecordField {
                name: name.into(),
                optional: false,
                type_ref,
            })
            .collect(),
    };
    Some(match name {
        "std/json::parse" => (vec![text], named("std/prelude::Either", vec![error, json])),
        "std/json::stringify" => (vec![json], text),
        "std/json::JsonNull" => (vec![], json),
        "std/json::JsonBool" => (vec![named("std/prelude::Bool", vec![])], json),
        "std/json::JsonNumber" => (vec![named("std/decimal::Decimal", vec![])], json),
        "std/json::JsonString" => (vec![text], json),
        "std/json::JsonArray" => (vec![array(json.clone())], json),
        "std/json::JsonObject" => (vec![named("std/map::Map", vec![text, json.clone()])], json),
        "std/json::JsonField" => (vec![text], path),
        "std/json::JsonIndex" => (vec![int], path),
        "std/json::InvalidJsonSyntax" => (
            vec![record(vec![("message", text), ("offset", int)])],
            error,
        ),
        "std/json::DuplicateJsonField" => (
            vec![record(vec![("field", text), ("path", array(path))])],
            error,
        ),
        _ => return None,
    })
}
fn remaining(parameters: &[CoreType], output: &CoreType) -> CoreType {
    parameters
        .iter()
        .rev()
        .fold(output.clone(), |result, parameter| CoreType::Function {
            parameter: Box::new(parameter.clone()),
            result: Box::new(result),
        })
}

fn compatible_input(actual: &CoreType, expected: &CoreType) -> bool {
    match (actual, expected) {
        (
            CoreType::Record { fields, .. },
            CoreType::Record {
                fields: required, ..
            },
        ) => required.iter().all(|expected| {
            fields.iter().any(|actual| {
                actual.name == expected.name
                    && actual.optional == expected.optional
                    && actual.type_ref == expected.type_ref
            })
        }),
        (
            CoreType::Function { parameter, result },
            CoreType::Function {
                parameter: required,
                result: output,
            },
        ) => compatible_input(parameter, required) && result == output,
        _ => actual == expected,
    }
}
impl Backend<'_> {
    fn json_type(&self, actual: &CoreType, expected: &CoreType, scope: &Scope) -> Result<()> {
        if self.program_wasm_library {
            return Err("program WASM: native JSON requires a target-specific ABI proof".into());
        }
        if self.declared_method_type_kind(actual, scope) != Some(0)
            || !compatible_input(&self.local_canonical_type(actual, scope)?, expected)
        {
            return Err("Rust backend: unproven canonical JSON argument/result type".into());
        }
        Ok(())
    }
    fn json_capabilities(&mut self) {
        self.capabilities.insert("json".into());
    }
    fn json_callable(name: &str, arity: usize) -> String {
        if arity == 0 {
            format!("native_json_operation({}, vec![])", quoted(name))
        } else {
            format!(
                "V::fun(vec![], |_,value| native_json_operation({},vec![value]))",
                quoted(name)
            )
        }
    }
    pub(super) fn json_reference(
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
            return Err("Rust backend: JSON operation with unsupported evidence".into());
        }
        self.json_type(type_ref, &remaining(&parameters, &result), scope)?;
        self.json_capabilities();
        Ok(Some(Self::json_callable(name, parameters.len())))
    }
    pub(super) fn json_call(&mut self, expr: &CoreExpr, scope: &Scope) -> Result<Option<String>> {
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
            return Err("Rust backend: unproven JSON source application".into());
        }
        for (argument, expected) in arguments.iter().zip(&parameters) {
            let actual = local::method_argument_type(argument)
                .ok_or("Rust backend: missing JSON argument type")?;
            self.json_type(&actual, expected, scope)?;
        }
        self.json_type(
            type_ref,
            &remaining(&parameters[arguments.len()..], &result),
            scope,
        )?;
        self.json_capabilities();
        if arguments.len() == parameters.len() {
            self.direct_calls += 1;
            Ok(Some(format!(
                "native_json_operation({},vec![{}])",
                quoted(callee),
                self.expressions(arguments, scope)?
            )))
        } else {
            Ok(Some(Self::json_callable(callee, parameters.len())))
        }
    }
}
