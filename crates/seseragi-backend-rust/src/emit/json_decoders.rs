use super::*;

fn arity(name: &str) -> Option<usize> {
    Some(match name {
        "std/json::field"
        | "std/json::optionalField"
        | "std/json::index"
        | "std/json::map"
        | "std/json::flatMap" => 2,
        "std/json::array" | "std/json::record" | "std/json::oneOf" => 1,
        _ => return None,
    })
}
fn named(name: &str, arguments: Vec<CoreType>) -> CoreType {
    CoreType::Named {
        name: name.into(),
        arguments,
    }
}
fn function(parameter: CoreType, result: CoreType) -> CoreType {
    CoreType::Function {
        parameter: Box::new(parameter),
        result: Box::new(result),
    }
}
fn decoder(value: CoreType) -> CoreType {
    function(
        named("std/json::Json", vec![]),
        named(
            "std/prelude::Either",
            vec![named("std/json::DecodeError", vec![]), value],
        ),
    )
}
fn element<'a>(value: &'a CoreType, owner: &str) -> Result<&'a CoreType> {
    match value {
        CoreType::Named { name, arguments } if name == owner && arguments.len() == 1 => {
            Ok(&arguments[0])
        }
        _ => Err("Rust backend: unproven JSON decoder container owner or arity".into()),
    }
}
fn result(value: &CoreType) -> Result<&CoreType> {
    let CoreType::Function { parameter, result } = value else {
        return Err("Rust backend: missing JSON decoder function stage".into());
    };
    if **parameter != named("std/json::Json", vec![]) {
        return Err("Rust backend: unproven JSON decoder input owner".into());
    }
    match &**result {
        CoreType::Named { name, arguments }
            if name == "std/prelude::Either"
                && arguments.len() == 2
                && arguments[0] == named("std/json::DecodeError", vec![]) =>
        {
            Ok(&arguments[1])
        }
        _ => Err("Rust backend: unproven JSON decoder result owner or error".into()),
    }
}

impl Backend<'_> {
    fn json_decoder_type(
        &self,
        name: &str,
        arguments: &[CoreExpr],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<usize> {
        if self.program_wasm_library {
            return Err(
                "program WASM: native JSON decoders require a target-specific ABI proof".into(),
            );
        }
        let arity = arity(name).expect("known decoder");
        if arguments.len() > arity + 1 || scope.has_unproven_type_scope() {
            return Err("Rust backend: unproven JSON decoder source arity or scope".into());
        }
        let mut full = type_ref.clone();
        for argument in arguments.iter().rev() {
            full = function(
                local::method_argument_type(argument)
                    .ok_or("Rust backend: missing JSON decoder argument type")?,
                full,
            );
        }
        if self.declared_method_type_kind(&full, scope) != Some(0) {
            return Err("Rust backend: unproven JSON decoder argument/result kind".into());
        }
        let full = self.local_canonical_type(&full, scope)?;
        let mut remaining = &full;
        let mut parameters = vec![];
        for _ in 0..arity {
            let CoreType::Function { parameter, result } = remaining else {
                return Err("Rust backend: missing JSON decoder source stage".into());
            };
            parameters.push(&**parameter);
            remaining = result;
        }
        let output = result(remaining)?;
        let text = named("std/prelude::String", vec![]);
        let valid = match name {
            "std/json::field" | "std/json::index" => {
                *parameters[0]
                    == if name.ends_with("index") {
                        named("std/prelude::Int", vec![])
                    } else {
                        text
                    }
                    && result(parameters[1])? == output
            }
            "std/json::optionalField" => {
                *parameters[0] == text
                    && result(parameters[1])? == element(output, "std/prelude::Maybe")?
            }
            "std/json::array" => result(parameters[0])? == element(output, "std/prelude::Array")?,
            "std/json::oneOf" => result(element(parameters[0], "std/prelude::Array")?)? == output,
            "std/json::record" => {
                let input = element(parameters[0], "std/prelude::Array")?;
                let output = element(output, "std/prelude::Array")?;
                match (input, output) {
                    (CoreType::Tuple { elements: input }, CoreType::Tuple { elements: output })
                        if input.len() == 2 && output.len() == 2 =>
                    {
                        input[0] == text && output[0] == text && result(&input[1])? == &output[1]
                    }
                    _ => false,
                }
            }
            "std/json::map" | "std/json::flatMap" => {
                let input = result(parameters[1])?;
                let CoreType::Function { parameter, result } = parameters[0] else {
                    return Err("Rust backend: missing JSON decoder transform stage".into());
                };
                **parameter == *input
                    && **result
                        == if name.ends_with("flatMap") {
                            decoder(output.clone())
                        } else {
                            output.clone()
                        }
            }
            _ => unreachable!(),
        };
        if !valid {
            return Err("Rust backend: unproven canonical JSON decoder signature".into());
        }
        Ok(arity)
    }
    fn json_decoder_callable(name: &str, arity: usize) -> String {
        if arity == 1 {
            format!(
                "V::fun(vec![],|_,value|native_json_decoder_operation({},vec![value]))",
                quoted(name)
            )
        } else {
            format!("V::fun(vec![],|_,first|V::fun(vec![first],|captures,second|native_json_decoder_operation({},vec![captures[0].clone(),second])))",quoted(name))
        }
    }
    pub(super) fn json_decoder_call(
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
        if arity(callee).is_none() || scope.contains(callee) {
            return Ok(None);
        }
        if !evidence.is_empty()
            || trait_dispatch.is_some()
            || !deferred_evidence_parameters.is_empty()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || evidence_argument_index.is_some()
        {
            return Err("Rust backend: unproven JSON decoder dictionaries or source stages".into());
        }
        let arity = self.json_decoder_type(callee, arguments, type_ref, scope)?;
        self.capabilities.insert("json".into());
        if arguments.len() == arity {
            self.direct_calls += 1;
            return Ok(Some(format!(
                "native_json_decoder_operation({},vec![{}])",
                quoted(callee),
                self.expressions(arguments, scope)?
            )));
        }
        let stage = self.fresh();
        let mut code = format!(
            "{{let mut {stage}={};",
            Self::json_decoder_callable(callee, arity)
        );
        for argument in arguments {
            code.push_str(&format!(
                "{stage}={stage}.apply({});",
                self.expr(argument, scope)?
            ));
        }
        code.push_str(&format!("{stage}}}"));
        Ok(Some(code))
    }
    pub(super) fn json_decoder_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if arity(name).is_none() || scope.contains(name) {
            return Ok(None);
        }
        if !evidence.is_empty() {
            return Err("Rust backend: unproven JSON decoder reference evidence".into());
        }
        let arity = self.json_decoder_type(name, &[], type_ref, scope)?;
        self.capabilities.insert("json".into());
        Ok(Some(Self::json_decoder_callable(name, arity)))
    }
}
