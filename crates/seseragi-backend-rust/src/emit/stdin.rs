use super::*;

fn named(name: &str, arguments: Vec<CoreType>) -> CoreType {
    CoreType::Named {
        name: name.into(),
        arguments,
    }
}

fn signature(name: &str) -> Option<(Vec<CoreType>, CoreType)> {
    let ty = |name| named(name, vec![]);
    let size = || ty("std/stdin::ReadSize");
    let limit = || ty("std/stdin::LineLimit");
    let environment = || CoreType::Record {
        closed: true,
        fields: vec![seseragi_lowering::CoreRecordField {
            name: "stdin".into(),
            optional: false,
            type_ref: ty("std/prelude::Stdin"),
        }],
    };
    let reading = |container, value| {
        named(
            container,
            vec![environment(), ty("std/prelude::StdinError"), value],
        )
    };
    let maybe = |value| named("std/prelude::Maybe", vec![value]);
    Some(match name {
        "std/stdin::readSize" | "std/stdin::lineLimit" => (
            vec![ty("std/prelude::Int")],
            named(
                "std/prelude::Either",
                vec![
                    ty("std/stdin::StdinConfigError"),
                    if name.ends_with("readSize") {
                        size()
                    } else {
                        limit()
                    },
                ],
            ),
        ),
        "std/stdin::defaultReadSize" => (vec![ty("std/prelude::Unit")], size()),
        "std/stdin::defaultLineLimit" => (vec![ty("std/prelude::Unit")], limit()),
        "std/stdin::readChunk" => (
            vec![size()],
            reading("std/prelude::Effect", maybe(ty("std/bytes::Bytes"))),
        ),
        "std/stdin::readLineWith" => (
            vec![limit()],
            reading("std/prelude::Effect", maybe(ty("std/prelude::String"))),
        ),
        "std/stdin::readLine" => (
            vec![ty("std/prelude::Unit")],
            reading("std/prelude::Effect", maybe(ty("std/prelude::String"))),
        ),
        "std/prelude::readLine" | "stdin.readLine" => (
            vec![],
            reading("std/prelude::Effect", maybe(ty("std/prelude::String"))),
        ),
        "std/stdin::lines" => (
            vec![limit()],
            reading("std/stream::Stream", ty("std/prelude::String")),
        ),
        _ => return None,
    })
}

fn arrow(parameters: &[CoreType], result: CoreType) -> CoreType {
    parameters
        .iter()
        .rev()
        .fold(result, |result, parameter| CoreType::Function {
            parameter: Box::new(parameter.clone()),
            result: Box::new(result),
        })
}

impl Backend<'_> {
    pub(super) fn stdin_reference(
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
        if self.program_wasm_library {
            return Err("program WASM: native Stdin requires a target-specific ABI proof".into());
        }
        if !evidence.is_empty()
            || self.declared_method_type_kind(type_ref, scope) != Some(0)
            || self.local_canonical_type(type_ref, scope)? != arrow(&parameters, result)
        {
            return Err("Rust backend: unproven canonical Stdin reference contract".into());
        }
        self.require_operation(name);
        if name.ends_with("readChunk")
            || name.ends_with("readLine")
            || name.ends_with("readLineWith")
            || name.ends_with("lines")
        {
            self.capabilities.insert("stdin".into());
        }
        let names: Vec<_> = (0..parameters.len())
            .map(|i| format!("\0stdin_argument_{i}"))
            .collect();
        let arguments = names
            .iter()
            .map(|name| format!("{}.clone()", id(name)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = format!(
            "native_stdin_operation({}, vec![{arguments}])",
            quoted(name)
        );
        for i in (0..names.len()).rev() {
            body = Self::closure(&names[i], &body, &names[..i].iter().cloned().collect());
        }
        Ok(Some(body))
    }

    pub(super) fn stdin_call(&mut self, expr: &CoreExpr, scope: &Scope) -> Result<Option<String>> {
        let (name, arguments, result, valid) = match expr {
            CoreExpr::Call {
                callee,
                arguments,
                type_ref,
                evidence,
                trait_dispatch,
                deferred_evidence_parameters,
                deferred_evidence_type_constructor_parameters,
                evidence_argument_index,
                ..
            } => (
                callee,
                arguments,
                type_ref.clone(),
                evidence.is_empty()
                    && trait_dispatch.is_none()
                    && deferred_evidence_parameters.is_empty()
                    && deferred_evidence_type_constructor_parameters.is_empty()
                    && evidence_argument_index.is_none(),
            ),
            CoreExpr::EffectInvoke {
                callee,
                arguments,
                requirements,
                failure,
                success,
                evidence,
                ..
            } => (
                callee,
                arguments,
                named(
                    "std/prelude::Effect",
                    vec![requirements.clone(), failure.clone(), success.clone()],
                ),
                evidence.is_empty(),
            ),
            CoreExpr::EffectOperation {
                operation,
                arguments,
                requirements,
                failure,
                success,
                ..
            } => (
                operation,
                arguments,
                named(
                    "std/prelude::Effect",
                    vec![requirements.clone(), failure.clone(), success.clone()],
                ),
                true,
            ),
            _ => return Ok(None),
        };
        let Some((parameters, expected)) = signature(name) else {
            return Ok(None);
        };
        if scope.contains(name) {
            return Ok(None);
        }
        if !valid || arguments.len() > parameters.len() {
            return Err("Rust backend: unproven Stdin source application".into());
        }
        for (argument, expected) in arguments.iter().zip(&parameters) {
            let actual = local::method_argument_type(argument)
                .ok_or("Rust backend: missing Stdin argument type")?;
            if self.declared_method_type_kind(&actual, scope) != Some(0)
                || self.local_canonical_type(&actual, scope)? != *expected
            {
                return Err("Rust backend: unproven canonical Stdin argument".into());
            }
        }
        if self.local_canonical_type(&result, scope)?
            != arrow(&parameters[arguments.len()..], expected)
        {
            return Err("Rust backend: unproven canonical Stdin result".into());
        }
        let function = self
            .stdin_reference(
                name,
                &[],
                &arrow(&parameters[..arguments.len()], result),
                scope,
            )?
            .unwrap();
        let temporary = self.fresh();
        let mut body = format!("{{ let mut {temporary} = {function};");
        for argument in arguments {
            body.push_str(&format!(
                "{temporary} = {temporary}.apply({});",
                self.expr(argument, scope)?
            ));
        }
        body.push_str(&format!("{temporary} }}"));
        Ok(Some(body))
    }
}
