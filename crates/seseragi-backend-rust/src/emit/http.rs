use super::*;

// The `std/http` values and functions that need no HttpClient. Each one maps
// to `native_http_operation` with its arguments in source order.

fn ty(name: &str) -> CoreType {
    CoreType::Named {
        name: name.into(),
        arguments: vec![],
    }
}
fn applied(name: &str, arguments: Vec<CoreType>) -> CoreType {
    CoreType::Named {
        name: name.into(),
        arguments,
    }
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

fn signature(name: &str) -> Option<(Vec<CoreType>, CoreType)> {
    let string = || ty("std/prelude::String");
    let int = || ty("std/prelude::Int");
    let bool = || ty("std/prelude::Bool");
    let method = || ty("std/http::Method");
    let status = || ty("std/http::Status");
    let headers = || ty("std/http::Headers");
    let url = || ty("std/http::HttpUrl");
    let request = || ty("std/http::Request");
    let response = || ty("std/http::Response");
    let limit = || ty("std/http::HttpBodyLimit");
    // An exchange needs the HttpClient service and fails with HttpError.
    let sent = || {
        applied(
            "std/prelude::Effect",
            vec![
                CoreType::Record {
                    closed: true,
                    fields: vec![seseragi_lowering::CoreRecordField {
                        name: "httpClient".into(),
                        optional: false,
                        type_ref: ty("std/http::HttpClient"),
                    }],
                },
                ty("std/http::HttpError"),
                response(),
            ],
        )
    };
    let built = |success: CoreType| {
        applied(
            "std/prelude::Either",
            vec![ty("std/http::HttpBuildError"), success],
        )
    };
    Some(match name {
        "std/http::get" | "std/http::head" | "std/http::post" | "std/http::put"
        | "std/http::patch" | "std/http::delete" | "std/http::options" | "std/http::connect"
        | "std/http::trace" => (vec![], method()),
        "std/http::HttpVersionUnknown"
        | "std/http::Http1_0"
        | "std/http::Http1_1"
        | "std/http::Http2"
        | "std/http::Http3" => (vec![], ty("std/http::HttpVersion")),
        "std/http::emptyHeaders" => (vec![], headers()),
        "std/http::customMethod" => (vec![string()], built(method())),
        "std/http::methodText" => (vec![method()], string()),
        "std/http::status" => (vec![int()], built(status())),
        "std/http::statusCode" => (vec![status()], int()),
        "std/http::isInformational"
        | "std/http::isSuccess"
        | "std/http::isRedirection"
        | "std/http::isClientError"
        | "std/http::isServerError" => (vec![status()], bool()),
        "std/http::parseUrl" => (vec![string()], built(url())),
        "std/http::renderUrl" => (vec![url()], string()),
        "std/http::appendHeader" | "std/http::setHeader" => {
            (vec![string(), string(), headers()], built(headers()))
        }
        "std/http::removeHeader" => (vec![string(), headers()], headers()),
        "std/http::headerValues" => (
            vec![string(), headers()],
            applied("std/prelude::Array", vec![string()]),
        ),
        "std/http::headerEntries" => (
            vec![headers()],
            applied(
                "std/prelude::Array",
                vec![CoreType::Tuple {
                    elements: vec![string(), string()],
                }],
            ),
        ),
        "std/http::request" => (vec![method(), url()], request()),
        "std/http::withRequestHeader" => (vec![string(), string(), request()], built(request())),
        "std/http::withoutRequestHeader" => (vec![string(), request()], request()),
        "std/http::bodyLimit" => (vec![int()], built(limit())),
        "std/http::defaultBodyLimit" => (vec![ty("std/prelude::Unit")], limit()),
        "std/http::responseStatus" => (vec![response()], status()),
        "std/http::responseHeaders" => (vec![response()], headers()),
        "std/http::responseBody" => (vec![response()], ty("std/bytes::Bytes")),
        "std/http::errorMessage" => (vec![ty("std/http::HttpError")], string()),
        "std/http::sendBytes" => (vec![limit(), ty("std/bytes::Bytes"), request()], sent()),
        "std/http::sendEmpty" => (vec![limit(), request()], sent()),
        _ => return None,
    })
}

impl Backend<'_> {
    pub(super) fn http_reference(
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
            return Err("Rust backend: std/http is not admitted in program WASM".into());
        }
        if !evidence.is_empty()
            || self.local_canonical_type(type_ref, scope)? != arrow(&parameters, result)
        {
            return Err("Rust backend: unproven native std/http reference contract".into());
        }
        self.capabilities.insert("http".into());
        if matches!(name, "std/http::sendBytes" | "std/http::sendEmpty") {
            self.capabilities.insert("effect-sync".into());
        }
        let names: Vec<_> = (0..parameters.len())
            .map(|i| format!("\0http_argument_{i}"))
            .collect();
        let arguments = names
            .iter()
            .map(|name| format!("{}.clone()", id(name)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = format!("native_http_operation({}, vec![{arguments}])", quoted(name));
        for i in (0..names.len()).rev() {
            body = Self::closure(&names[i], &body, &names[..i].iter().cloned().collect());
        }
        Ok(Some(body))
    }

    pub(super) fn http_call(&mut self, expr: &CoreExpr, scope: &Scope) -> Result<Option<String>> {
        let (callee, arguments, type_ref, valid) = match expr {
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
                applied(
                    "std/prelude::Effect",
                    vec![requirements.clone(), failure.clone(), success.clone()],
                ),
                evidence.is_empty(),
            ),
            _ => return Ok(None),
        };
        let Some((parameters, expected)) = signature(callee) else {
            return Ok(None);
        };
        if scope.contains(callee) {
            return Ok(None);
        }
        if !valid || arguments.len() > parameters.len() {
            return Err("Rust backend: unproven native std/http call".into());
        }
        for (argument, parameter) in arguments.iter().zip(&parameters) {
            let actual = local::method_argument_type(argument)
                .ok_or("Rust backend: missing std/http argument type")?;
            if self.local_canonical_type(&actual, scope)? != *parameter {
                return Err("Rust backend: native std/http canonical argument mismatch".into());
            }
        }
        if self.local_canonical_type(&type_ref, scope)?
            != arrow(&parameters[arguments.len()..], expected.clone())
        {
            return Err("Rust backend: native std/http canonical result mismatch".into());
        }
        let function = self
            .http_reference(callee, &[], &arrow(&parameters, expected), scope)?
            .unwrap();
        let temporary = self.fresh();
        let mut value = format!("{{ let mut {temporary} = {function};");
        for argument in arguments {
            value.push_str(&format!(
                "{temporary} = {temporary}.apply({});",
                self.expr(argument, scope)?
            ));
        }
        value.push_str(&format!("{temporary} }}"));
        Ok(Some(value))
    }
}
