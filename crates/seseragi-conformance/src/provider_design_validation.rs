use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

pub(crate) fn check_provider_design_validation_case(case: &Path) -> Result<(), String> {
    let raw = fs::read_to_string(case.join("validation.json"))
        .map_err(|error| format!("failed to read provider design validation: {error}"))?;
    let validation: DesignValidation = serde_json::from_str(&raw)
        .map_err(|error| format!("failed to parse provider design validation: {error}"))?;
    if validation.schema != 1
        || validation.kind != "provider-design-validation"
        || validation.identity != "seseragi/provider-system"
    {
        return Err("provider design validation envelope is not canonical".to_owned());
    }
    check_capabilities(&validation.capabilities)?;
    check_set(
        &validation.diagnostics,
        &["missing", "ambiguous", "target", "contract", "abi"],
    )?;
    check_set(
        &validation.conformance,
        &[
            "success",
            "typed-failure",
            "defect",
            "cancellation",
            "cleanup",
            "concurrency",
            "invalid-value",
            "mismatch",
            "ambiguity",
        ],
    )?;
    let expected = [
        "contract-artifact",
        "manifest-resolution",
        "typescript-bridge",
        "target-diagnostics",
        "provider-package",
        "clock",
        "http-server",
        "http-client-node",
        "filesystem",
        "postgresql",
        "conformance-guide",
    ];
    if validation.handoff != expected {
        return Err("provider implementation handoff order is not canonical".to_owned());
    }
    let handler_raw = fs::read_to_string(case.join("http-server-handler.json"))
        .map_err(|error| format!("failed to read HTTP server handler contract: {error}"))?;
    let handler: HttpServerHandlerContract = serde_json::from_str(&handler_raw)
        .map_err(|error| format!("failed to parse HTTP server handler contract: {error}"))?;
    check_http_server_handler_contract(&handler)?;
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DesignValidation {
    schema: u32,
    kind: String,
    identity: String,
    capabilities: Vec<Capability>,
    diagnostics: Vec<String>,
    conformance: Vec<String>,
    handoff: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Capability {
    name: String,
    #[serde(rename = "applicationApi")]
    application_api: String,
    contract: String,
    abi: String,
    provider: String,
    properties: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HttpServerHandlerContract {
    schema: u32,
    kind: String,
    identity: String,
    application: HttpServerHandlerApplication,
    lifecycle: HttpServerHandlerLifecycle,
    failures: Vec<HttpServerHandlerFailure>,
    concurrency: Vec<String>,
    acceptance: Vec<String>,
    #[serde(rename = "nonGoals")]
    non_goals: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HttpServerHandlerApplication {
    #[serde(rename = "handlerSignature")]
    handler_signature: String,
    #[serde(rename = "serverBoundary")]
    server_boundary: String,
    #[serde(rename = "listenEnvironment")]
    listen_environment: String,
    #[serde(rename = "listenFailure")]
    listen_failure: String,
    #[serde(rename = "pureAdapter")]
    pure_adapter: String,
    #[serde(rename = "failureAdapter")]
    failure_adapter: String,
    #[serde(rename = "providerContract")]
    provider_contract: String,
    #[serde(rename = "typescriptAbi")]
    typescript_abi: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HttpServerHandlerLifecycle {
    #[serde(rename = "requestParent")]
    request_parent: String,
    #[serde(rename = "requestOwns")]
    request_owns: Vec<String>,
    shutdown: Vec<String>,
    #[serde(rename = "serveOnce")]
    serve_once: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HttpServerHandlerFailure {
    source: String,
    outcome: String,
    channel: String,
}

fn check_http_server_handler_contract(contract: &HttpServerHandlerContract) -> Result<(), String> {
    if contract.schema != 1
        || contract.kind != "http-server-handler-contract"
        || contract.identity != "std/http/server::Handler"
    {
        return Err("HTTP server handler contract envelope is not canonical".to_owned());
    }
    let application = &contract.application;
    if application.handler_signature
        != "Handler<R, E> = HttpServerRequest -> Effect<R, E, HttpServerResponse>"
        || application.server_boundary != "Handler<R, Never>"
        || application.listen_environment != "R & { httpServer: HttpServer }"
        || application.listen_failure != "HttpServerError"
        || application.pure_adapter != "pureHandler"
        || application.failure_adapter != "recoverHandler"
        || application.provider_contract != "std/http/server::HttpServer#{listen,close}@1"
        || application.typescript_abi != "seseragi/provider-abi/typescript@1"
    {
        return Err("HTTP server handler application boundary is not canonical".to_owned());
    }
    if contract.lifecycle.request_parent != "server-resource-scope"
        || contract.lifecycle.request_owns
            != ["handler-execution", "response-write", "resource-scope"]
        || contract.lifecycle.shutdown
            != [
                "stop-accepting",
                "cancel-request-scopes",
                "await-handler-and-finalizers",
                "discard-late-responses",
                "close-listener",
            ]
        || contract.lifecycle.serve_once
            != [
                "claim-one-request",
                "stop-accepting",
                "await-response-write-and-cleanup",
                "close-listener",
            ]
    {
        return Err("HTTP server handler lifecycle is not canonical".to_owned());
    }
    let failures = BTreeMap::from([
        ("listen", ("typed-failure", "HttpServerError")),
        (
            "handler-typed-failure",
            ("response", "explicit-recoverHandler"),
        ),
        ("request-boundary", ("defect", "runtime-diagnostic")),
        ("response-write", ("defect", "runtime-diagnostic")),
        ("cancellation", ("cancellation", "effect-lifecycle")),
        ("handler-defect", ("defect", "runtime-diagnostic")),
    ]);
    let actual_failures = contract
        .failures
        .iter()
        .map(|failure| {
            (
                failure.source.as_str(),
                (failure.outcome.as_str(), failure.channel.as_str()),
            )
        })
        .collect::<BTreeMap<_, _>>();
    if actual_failures != failures || contract.failures.len() != failures.len() {
        return Err("HTTP server handler failure boundary is not canonical".to_owned());
    }
    check_set(
        &contract.concurrency,
        &[
            "one-scope-per-request",
            "parallel-independent-handlers",
            "reentrant-handler",
            "no-late-response-write",
        ],
    )?;
    check_set(
        &contract.acceptance,
        &[
            "effect-success",
            "typed-failure-recovered",
            "pure-handler",
            "concurrent-requests",
            "server-close-cancels-in-flight",
            "request-resource-cleanup",
            "late-completion-discarded",
            "json-db-http-clock-chain",
        ],
    )?;
    check_set(
        &contract.non_goals,
        &[
            "router",
            "middleware",
            "authentication",
            "websocket-sse",
            "streaming-body",
            "provider-redesign",
        ],
    )?;
    Ok(())
}

fn check_capabilities(capabilities: &[Capability]) -> Result<(), String> {
    let expected = BTreeMap::from([
        (
            "clock",
            (
                "std/clock::{now,sleep}",
                "std/clock::Clock#{now,sleep}",
                BTreeSet::from(["one-shot", "cancellation"]),
            ),
        ),
        (
            "http-client",
            (
                "std/http::{sendBytes,sendEmpty}",
                "std/http::HttpClient#send",
                BTreeSet::from(["request-response", "copied-bytes", "cancellation"]),
            ),
        ),
        (
            "http-server",
            (
                "std/http/server::{listen,serveOnce,close}",
                "std/http/server::HttpServer#{listen,close}",
                BTreeSet::from(["handler", "resource", "shutdown"]),
            ),
        ),
        (
            "filesystem",
            (
                "std/fs::{readBytes,readChunks}",
                "std/fs::FileSystem#{openRead,read,close}",
                BTreeSet::from(["opaque-handle", "bytes", "resource", "cleanup"]),
            ),
        ),
        (
            "postgresql",
            (
                "PostgreSQL-specific package API",
                "acme/postgres::Postgres#{openPool,query,openCursor,fetch,closeCursor,closePool}",
                BTreeSet::from(["external-driver", "pool", "row", "cursor"]),
            ),
        ),
    ]);
    let mut actual = BTreeMap::new();
    for capability in capabilities {
        let api = capability.application_api.to_ascii_lowercase();
        if api.contains("provider")
            || api.contains("bun")
            || api.contains("node")
            || capability.contract.is_empty()
            || capability.abi != "seseragi/provider-abi/typescript@1"
            || capability.provider.is_empty()
        {
            return Err(format!(
                "provider design capability leaks implementation details: {}",
                capability.name
            ));
        }
        actual.insert(
            capability.name.as_str(),
            (
                capability.application_api.as_str(),
                capability.contract.as_str(),
                capability
                    .properties
                    .iter()
                    .map(String::as_str)
                    .collect::<BTreeSet<_>>(),
            ),
        );
    }
    if actual != expected {
        return Err("provider design must cover all five capability shapes".to_owned());
    }
    Ok(())
}

fn check_set(values: &[String], expected: &[&str]) -> Result<(), String> {
    if values.iter().map(String::as_str).collect::<BTreeSet<_>>()
        != expected.iter().copied().collect::<BTreeSet<_>>()
    {
        return Err("provider design coverage set is incomplete".to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{check_http_server_handler_contract, check_provider_design_validation_case};
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn accepts_committed_design_validation() {
        let case = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/spec/artifacts/provider-design-validation-schema-1/system");
        check_provider_design_validation_case(&case).unwrap();
    }

    #[test]
    fn rejects_handler_failure_crossing_the_server_boundary() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../examples/spec/artifacts/provider-design-validation-schema-1/system/http-server-handler.json",
        );
        let raw = fs::read_to_string(path).unwrap();
        let mut contract = serde_json::from_str::<super::HttpServerHandlerContract>(&raw).unwrap();
        contract.application.server_boundary = "Handler<R, E>".to_owned();
        assert_eq!(
            check_http_server_handler_contract(&contract).unwrap_err(),
            "HTTP server handler application boundary is not canonical"
        );
    }
}
