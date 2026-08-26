#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RuntimeProviderServiceOperation {
    pub(crate) canonical: &'static str,
    pub(crate) runtime_feature: &'static str,
    pub(crate) local_name: &'static str,
    pub(crate) module: &'static str,
    pub(crate) export_name: &'static str,
    pub(crate) source_map_name: &'static str,
    pub(crate) type_argument_sources: &'static [usize],
}

macro_rules! operation {
    ($canonical:expr, $feature:expr, $local:expr, $module:expr, $export:expr) => {
        RuntimeProviderServiceOperation {
            canonical: $canonical,
            runtime_feature: $feature,
            local_name: $local,
            module: $module,
            export_name: $export,
            source_map_name: $export,
            type_argument_sources: &[],
        }
    };
}

macro_rules! http_operation {
    ($name:literal, $feature:literal) => {
        operation!(
            concat!("std/http::", $name),
            concat!("http-client.", $feature),
            concat!("_ssrg_http_client_", $name),
            "@seseragi/runtime/http-client",
            $name
        )
    };
}

macro_rules! navigation_operation {
    ($name:literal, $feature:literal) => {
        operation!(
            concat!("std/web/navigation::", $name),
            concat!("web.navigation.", $feature),
            concat!("_ssrg_navigation_", $name),
            "@seseragi/runtime/navigation",
            $name
        )
    };
}

macro_rules! storage_operation {
    ($name:literal, $feature:literal) => {
        operation!(
            concat!("std/web/storage::", $name),
            concat!("web.storage.", $feature),
            concat!("_ssrg_storage_", $name),
            "@seseragi/runtime/storage",
            $name
        )
    };
}

macro_rules! sse_operation {
    ($name:literal, $feature:literal) => {
        operation!(
            concat!("std/sse::", $name),
            concat!("sse.", $feature),
            concat!("_ssrg_sse_", $name),
            "@seseragi/runtime/sse",
            $name
        )
    };
}

macro_rules! multipart_operation {
    ($name:literal, $feature:literal) => {
        operation!(
            concat!("std/http/multipart::", $name),
            concat!("http.multipart.", $feature),
            concat!("_ssrg_multipart_", $name),
            "@seseragi/runtime/multipart",
            $name
        )
    };
}

macro_rules! web_file_operation {
    ($name:literal, $feature:literal) => {
        operation!(
            concat!("std/web/file::", $name),
            concat!("web.file.", $feature),
            concat!("_ssrg_web_file_", $name),
            "@seseragi/runtime/web-file",
            $name
        )
    };
}

macro_rules! postgres_operation {
    ($name:literal, $feature:literal) => {
        operation!(
            concat!("seseragi/postgres::", $name),
            concat!("postgres.", $feature),
            concat!("_ssrg_postgres_", $name),
            "@seseragi/runtime/postgres",
            $name
        )
    };
}

macro_rules! sqlite_operation {
    ($name:literal, $feature:literal) => {
        operation!(
            concat!("seseragi/sqlite::", $name),
            concat!("sqlite.", $feature),
            concat!("_ssrg_sqlite_", $name),
            "@seseragi/runtime/sqlite",
            $name
        )
    };
}

const OPERATIONS: &[RuntimeProviderServiceOperation] = &[
    operation!(
        "std/clock::now",
        "clock.now",
        "_ssrg_clock_now",
        "@seseragi/runtime/clock",
        "now"
    ),
    operation!(
        "std/clock::sleep",
        "clock.sleep",
        "_ssrg_clock_sleep",
        "@seseragi/runtime/clock",
        "sleep"
    ),
    navigation_operation!("InvalidUrl", "error.invalid-url"),
    navigation_operation!("UnsupportedUrlScheme", "error.unsupported-scheme"),
    navigation_operation!("UrlContainsUserInfo", "error.user-info"),
    navigation_operation!("InvalidPercentEncoding", "error.percent-encoding"),
    navigation_operation!("CrossOriginNavigation", "failure.cross-origin"),
    navigation_operation!("NavigationUnavailable", "failure.unavailable"),
    navigation_operation!("NavigationSecurityFailure", "failure.security"),
    navigation_operation!("parseUrl", "url.parse"),
    navigation_operation!("resolveUrl", "url.resolve"),
    navigation_operation!("renderUrl", "url.render"),
    navigation_operation!("urlOrigin", "url.origin"),
    navigation_operation!("pathSegments", "url.path-segments"),
    navigation_operation!("withPathSegments", "url.with-path-segments"),
    navigation_operation!("urlQuery", "url.query"),
    navigation_operation!("withQuery", "url.with-query"),
    navigation_operation!("urlFragment", "url.fragment"),
    navigation_operation!("withFragment", "url.with-fragment"),
    navigation_operation!("withoutFragment", "url.without-fragment"),
    navigation_operation!("emptyQuery", "query.empty"),
    navigation_operation!("parseQuery", "query.parse"),
    navigation_operation!("appendQuery", "query.append"),
    navigation_operation!("setQuery", "query.set"),
    navigation_operation!("removeQuery", "query.remove"),
    navigation_operation!("queryValues", "query.values"),
    navigation_operation!("queryEntries", "query.entries"),
    navigation_operation!("renderQuery", "query.render"),
    navigation_operation!("toWebUrl", "url.to-web-url"),
    navigation_operation!("locationUrl", "location.url"),
    navigation_operation!("current", "current"),
    navigation_operation!("push", "push"),
    navigation_operation!("replace", "replace"),
    navigation_operation!("back", "back"),
    navigation_operation!("forward", "forward"),
    navigation_operation!("locationSignal", "location-signal"),
    navigation_operation!("errorMessage", "error-message"),
    storage_operation!("Local", "area.local"),
    storage_operation!("Session", "area.session"),
    storage_operation!("StorageQuotaExceeded", "failure.quota"),
    storage_operation!("StorageSecurityFailure", "failure.security"),
    storage_operation!("StorageUnavailable", "failure.unavailable"),
    storage_operation!("get", "get"),
    storage_operation!("set", "set"),
    storage_operation!("remove", "remove"),
    storage_operation!("clear", "clear"),
    storage_operation!("keys", "keys"),
    storage_operation!("errorMessage", "error-message"),
    http_operation!("InvalidHttpUrl", "error.invalid-url"),
    http_operation!("UnsupportedHttpScheme", "error.unsupported-scheme"),
    http_operation!("HttpUrlContainsUserInfo", "error.url-user-info"),
    http_operation!("HttpUrlContainsFragment", "error.url-fragment"),
    http_operation!("InvalidHttpMethod", "error.invalid-method"),
    http_operation!("InvalidHeaderName", "error.invalid-header-name"),
    http_operation!("InvalidHeaderValue", "error.invalid-header-value"),
    http_operation!("ManagedHttpHeader", "error.managed-header"),
    http_operation!("InvalidHttpStatus", "error.invalid-status"),
    http_operation!("InvalidHttpBodyLimit", "error.invalid-body-limit"),
    http_operation!("HttpDnsFailure", "failure.dns"),
    http_operation!("HttpConnectionFailure", "failure.connection"),
    http_operation!("HttpTlsFailure", "failure.tls"),
    http_operation!("HttpProtocolFailure", "failure.protocol"),
    http_operation!("HttpRequestBodyFailure", "failure.request-body"),
    http_operation!("HttpRequestLengthMismatch", "failure.length-mismatch"),
    http_operation!(
        "HttpResponseBodyLimitExceeded",
        "failure.response-body-limit"
    ),
    http_operation!("HttpClientUnavailable", "failure.unavailable"),
    http_operation!("HttpVersionUnknown", "version.unknown"),
    http_operation!("Http1_0", "version.http-1-0"),
    http_operation!("Http1_1", "version.http-1-1"),
    http_operation!("Http2", "version.http-2"),
    http_operation!("Http3", "version.http-3"),
    http_operation!("InformationalResponse", "event.informational"),
    http_operation!("ResponseStarted", "event.response-started"),
    http_operation!("ResponseBodyChunk", "event.body-chunk"),
    http_operation!("ResponseTrailers", "event.trailers"),
    http_operation!("get", "method.get"),
    http_operation!("head", "method.head"),
    http_operation!("post", "method.post"),
    http_operation!("put", "method.put"),
    http_operation!("patch", "method.patch"),
    http_operation!("delete", "method.delete"),
    http_operation!("options", "method.options"),
    http_operation!("connect", "method.connect"),
    http_operation!("trace", "method.trace"),
    http_operation!("customMethod", "method.custom"),
    http_operation!("methodText", "method.text"),
    http_operation!("status", "status.build"),
    http_operation!("statusCode", "status.code"),
    http_operation!("isInformational", "status.informational"),
    http_operation!("isSuccess", "status.success"),
    http_operation!("isRedirection", "status.redirection"),
    http_operation!("isClientError", "status.client-error"),
    http_operation!("isServerError", "status.server-error"),
    http_operation!("parseUrl", "url.parse"),
    http_operation!("renderUrl", "url.render"),
    http_operation!("emptyHeaders", "headers.empty"),
    http_operation!("appendHeader", "headers.append"),
    http_operation!("setHeader", "headers.set"),
    http_operation!("removeHeader", "headers.remove"),
    http_operation!("headerValues", "headers.values"),
    http_operation!("headerEntries", "headers.entries"),
    http_operation!("request", "request.create"),
    http_operation!("withRequestHeader", "request.with-header"),
    http_operation!("withoutRequestHeader", "request.without-header"),
    http_operation!("emptyBody", "body.empty"),
    http_operation!("bytesBody", "body.bytes"),
    http_operation!("streamBody", "body.stream"),
    http_operation!("exchange", "exchange"),
    http_operation!("bodyLimit", "body-limit.build"),
    http_operation!("defaultBodyLimit", "body-limit.default"),
    http_operation!("sendBytes", "send-bytes"),
    http_operation!("sendEmpty", "send-empty"),
    http_operation!("responseStatus", "response.status"),
    http_operation!("responseHeaders", "response.headers"),
    http_operation!("responseBody", "response.body"),
    operation!(
        "std/http::errorMessage",
        "http-client.errorMessage",
        "_ssrg_http_client_error_message",
        "@seseragi/runtime/http-client",
        "errorMessage"
    ),
    operation!(
        "std/http/server::requestMethod",
        "http-server.request-method",
        "_ssrg_http_server_requestMethod",
        "@seseragi/runtime/http-server",
        "requestMethod"
    ),
    operation!(
        "std/http/server::requestUrl",
        "http-server.request-url",
        "_ssrg_http_server_requestUrl",
        "@seseragi/runtime/http-server",
        "requestUrl"
    ),
    operation!(
        "std/http/server::requestPath",
        "http-server.request-path",
        "_ssrg_http_server_requestPath",
        "@seseragi/runtime/http-server",
        "requestPath"
    ),
    operation!(
        "std/http/server::requestQuery",
        "http-server.request-query",
        "_ssrg_http_server_requestQuery",
        "@seseragi/runtime/http-server",
        "requestQuery"
    ),
    operation!(
        "std/http/server::requestHeaders",
        "http-server.request-headers",
        "_ssrg_http_server_requestHeaders",
        "@seseragi/runtime/http-server",
        "requestHeaders"
    ),
    operation!(
        "std/http/server::requestHeaderValues",
        "http-server.request-header-values",
        "_ssrg_http_server_requestHeaderValues",
        "@seseragi/runtime/http-server",
        "requestHeaderValues"
    ),
    operation!(
        "std/http/server::requestBody",
        "http-server.request-body",
        "_ssrg_http_server_requestBody",
        "@seseragi/runtime/http-server",
        "requestBody"
    ),
    operation!(
        "std/http/server::header",
        "http-server.header",
        "_ssrg_http_server_header",
        "@seseragi/runtime/http-server",
        "header"
    ),
    operation!(
        "std/http/server::emptyResponse",
        "http-server.empty-response",
        "_ssrg_http_server_emptyResponse",
        "@seseragi/runtime/http-server",
        "emptyResponse"
    ),
    operation!(
        "std/http/server::bytesResponse",
        "http-server.bytes-response",
        "_ssrg_http_server_bytesResponse",
        "@seseragi/runtime/http-server",
        "bytesResponse"
    ),
    operation!(
        "std/http/server::streamResponse",
        "http-server.stream-response",
        "_ssrg_http_server_streamResponse",
        "@seseragi/runtime/http-server",
        "streamResponse"
    ),
    operation!(
        "std/http/server::textResponse",
        "http-server.text-response",
        "_ssrg_http_server_textResponse",
        "@seseragi/runtime/http-server",
        "textResponse"
    ),
    operation!(
        "std/http/server::jsonResponse",
        "http-server.jsonResponse",
        "_ssrg_http_server_jsonResponse",
        "@seseragi/runtime/http-server",
        "jsonResponse"
    ),
    operation!(
        "std/http/server::pureHandler",
        "http-server.pure-handler",
        "_ssrg_http_server_pureHandler",
        "@seseragi/runtime/http-server",
        "pureHandler"
    ),
    operation!(
        "std/http/server::recoverHandler",
        "http-server.recover-handler",
        "_ssrg_http_server_recoverHandler",
        "@seseragi/runtime/http-server",
        "recoverHandler"
    ),
    operation!(
        "std/http/server::errorMessage",
        "http-server.errorMessage",
        "_ssrg_http_server_errorMessage",
        "@seseragi/runtime/http-server",
        "errorMessage"
    ),
    operation!(
        "std/http/server::listen",
        "http-server.listen",
        "_ssrg_http_server_listen",
        "@seseragi/runtime/http-server",
        "listen"
    ),
    operation!(
        "std/http/server::serveOnce",
        "http-server.serveOnce",
        "_ssrg_http_server_serveOnce",
        "@seseragi/runtime/http-server",
        "serveOnce"
    ),
    operation!(
        "std/http/server::close",
        "http-server.close",
        "_ssrg_http_server_close",
        "@seseragi/runtime/http-server",
        "close"
    ),
    multipart_operation!("InvalidMultipartFieldName", "error.field-name"),
    multipart_operation!("InvalidMultipartFileName", "error.file-name"),
    multipart_operation!("InvalidMultipartMimeType", "error.mime-type"),
    multipart_operation!("empty", "empty"),
    multipart_operation!("appendText", "append-text"),
    multipart_operation!("appendBytes", "append-bytes"),
    multipart_operation!("appendBody", "append-body"),
    multipart_operation!("contentType", "content-type"),
    multipart_operation!("body", "body"),
    web_file_operation!("InvalidBlobMimeType", "error.mime-type"),
    web_file_operation!("BlobReadLimitExceeded", "error.limit"),
    web_file_operation!("BlobReadFailure", "error.read"),
    web_file_operation!("fromBytes", "from-bytes"),
    web_file_operation!("asBlob", "as-blob"),
    web_file_operation!("name", "name"),
    web_file_operation!("mimeType", "mime-type"),
    web_file_operation!("sizeBytes", "size-bytes"),
    web_file_operation!("lastModifiedMillis", "last-modified"),
    web_file_operation!("readBytes", "read-bytes"),
    web_file_operation!("readChunks", "read-chunks"),
    web_file_operation!("body", "body"),
    sse_operation!("InvalidSseEventName", "error.invalid-event-name"),
    sse_operation!("InvalidSseEventId", "error.invalid-event-id"),
    sse_operation!("InvalidSseRetryMillis", "error.invalid-retry"),
    sse_operation!("InvalidSseComment", "error.invalid-comment"),
    sse_operation!("InvalidSseDecodeLimit", "error.invalid-limit"),
    sse_operation!("SseUnexpectedStatus", "failure.status"),
    sse_operation!("SseInvalidContentType", "failure.content-type"),
    sse_operation!("SseInvalidUtf8", "failure.utf8"),
    sse_operation!("SseEventTooLarge", "failure.event-too-large"),
    sse_operation!("SseMalformedId", "failure.id"),
    sse_operation!("SseMalformedRetry", "failure.retry"),
    sse_operation!("SseMalformedHttpEvents", "failure.http-events"),
    sse_operation!("event", "event.create"),
    sse_operation!("withEventName", "event.with-name"),
    sse_operation!("withId", "event.with-id"),
    sse_operation!("withRetryMillis", "event.with-retry"),
    sse_operation!("eventData", "event.data"),
    sse_operation!("eventName", "event.name"),
    sse_operation!("eventId", "event.id"),
    sse_operation!("eventRetryMillis", "event.retry"),
    sse_operation!("encode", "encode"),
    sse_operation!("keepAlive", "keepalive"),
    sse_operation!("decodeLimit", "decode-limit.build"),
    sse_operation!("defaultDecodeLimit", "decode-limit.default"),
    sse_operation!("withLastEventId", "request.last-event-id"),
    sse_operation!("events", "events"),
    sse_operation!("response", "response"),
    operation!(
        "std/websocket::connect",
        "websocket.connect",
        "_ssrg_websocket_connect",
        "@seseragi/runtime/websocket",
        "connect"
    ),
    operation!(
        "std/websocket::messages",
        "websocket.messages",
        "_ssrg_websocket_messages",
        "@seseragi/runtime/websocket",
        "messages"
    ),
    operation!(
        "std/websocket::sendText",
        "websocket.send-text",
        "_ssrg_websocket_sendText",
        "@seseragi/runtime/websocket",
        "sendText"
    ),
    operation!(
        "std/websocket::sendBytes",
        "websocket.send-bytes",
        "_ssrg_websocket_sendBytes",
        "@seseragi/runtime/websocket",
        "sendBytes"
    ),
    operation!(
        "std/websocket::closeConnection",
        "websocket.close-connection",
        "_ssrg_websocket_closeConnection",
        "@seseragi/runtime/websocket",
        "closeConnection"
    ),
    operation!(
        "std/websocket::selectedProtocol",
        "websocket.selected-protocol",
        "_ssrg_websocket_selectedProtocol",
        "@seseragi/runtime/websocket",
        "selectedProtocol"
    ),
    operation!(
        "std/websocket::foldEvent",
        "websocket.fold-event",
        "_ssrg_websocket_foldEvent",
        "@seseragi/runtime/websocket",
        "foldEvent"
    ),
    operation!(
        "std/websocket::closeCode",
        "websocket.close-code",
        "_ssrg_websocket_closeCode",
        "@seseragi/runtime/websocket",
        "closeCode"
    ),
    operation!(
        "std/websocket::closeReason",
        "websocket.close-reason",
        "_ssrg_websocket_closeReason",
        "@seseragi/runtime/websocket",
        "closeReason"
    ),
    operation!(
        "std/websocket::closeWasClean",
        "websocket.close-clean",
        "_ssrg_websocket_closeWasClean",
        "@seseragi/runtime/websocket",
        "closeWasClean"
    ),
    operation!(
        "std/websocket::errorMessage",
        "websocket.error-message",
        "_ssrg_websocket_errorMessage",
        "@seseragi/runtime/websocket",
        "errorMessage"
    ),
    operation!(
        "std/websocket/server::listen",
        "websocket.server-listen",
        "_ssrg_websocket_server_listen",
        "@seseragi/runtime/websocket",
        "listen"
    ),
    operation!(
        "std/websocket/server::closeServer",
        "websocket.server-close",
        "_ssrg_websocket_server_closeServer",
        "@seseragi/runtime/websocket",
        "closeServer"
    ),
    postgres_operation!("textValue", "value.text"),
    postgres_operation!("intValue", "value.int"),
    postgres_operation!("floatValue", "value.float"),
    postgres_operation!("boolValue", "value.bool"),
    postgres_operation!("bytesValue", "value.bytes"),
    postgres_operation!("nullValue", "value.null"),
    postgres_operation!("emptyValues", "value.empty"),
    postgres_operation!("string", "decoder.string"),
    postgres_operation!("int", "decoder.int"),
    postgres_operation!("float", "decoder.float"),
    postgres_operation!("bool", "decoder.bool"),
    postgres_operation!("bytes", "decoder.bytes"),
    postgres_operation!("map2", "decoder.map2"),
    postgres_operation!("openPool", "pool.open"),
    postgres_operation!("query", "query"),
    postgres_operation!("transactionQuery", "transaction.query"),
    postgres_operation!("transaction", "transaction.run"),
    postgres_operation!("openCursor", "cursor.open"),
    postgres_operation!("fetch", "cursor.fetch"),
    postgres_operation!("closeCursor", "cursor.close"),
    postgres_operation!("closePool", "pool.close"),
    sqlite_operation!("textValue", "value.text"),
    sqlite_operation!("intValue", "value.int"),
    sqlite_operation!("floatValue", "value.float"),
    sqlite_operation!("boolValue", "value.bool"),
    sqlite_operation!("bytesValue", "value.bytes"),
    sqlite_operation!("nullValue", "value.null"),
    sqlite_operation!("emptyValues", "value.empty"),
    sqlite_operation!("string", "decoder.string"),
    sqlite_operation!("int", "decoder.int"),
    sqlite_operation!("float", "decoder.float"),
    sqlite_operation!("bool", "decoder.bool"),
    sqlite_operation!("bytes", "decoder.bytes"),
    sqlite_operation!("map2", "decoder.map2"),
    sqlite_operation!("openMemory", "database.open-memory"),
    sqlite_operation!("openFile", "database.open-file"),
    sqlite_operation!("query", "query"),
    sqlite_operation!("execute", "execute"),
    sqlite_operation!("transactionQuery", "transaction.query"),
    sqlite_operation!("transactionExecute", "transaction.execute"),
    sqlite_operation!("transactionThen", "transaction.then"),
    sqlite_operation!("transaction", "transaction.run"),
    sqlite_operation!("close", "database.close"),
];

pub(crate) fn runtime_provider_service_operation(
    canonical: &str,
) -> Option<RuntimeProviderServiceOperation> {
    let normalized = stable_package_operation_identity(canonical);
    OPERATIONS
        .iter()
        .copied()
        .find(|operation| operation.canonical == canonical || operation.canonical == normalized)
}

fn stable_package_operation_identity(canonical: &str) -> String {
    let Some((package, tail)) = canonical.split_once("::") else {
        return canonical.to_owned();
    };
    let Some((name, version)) = package.rsplit_once('@') else {
        return canonical.to_owned();
    };
    if semver::Version::parse(version).is_err() {
        return canonical.to_owned();
    }
    let Some((module, operation)) = tail.rsplit_once("::") else {
        return canonical.to_owned();
    };
    if module == "lib" {
        format!("{name}::{operation}")
    } else {
        format!("{name}/{module}::{operation}")
    }
}

pub(crate) fn runtime_provider_service_value_operation(
    name: &str,
    type_ref: &CoreType,
) -> Option<RuntimeProviderServiceOperation> {
    if let Some(operation) = runtime_provider_service_operation(name) {
        return Some(operation);
    }
    let is_navigation_query = matches!(type_ref, CoreType::ExternalNamed { canonical, .. } if canonical == "std/web/navigation::Query");
    if is_navigation_query && name.contains('.') {
        let member = name.rsplit('.').next()?;
        return OPERATIONS.iter().copied().find(|operation| {
            operation.runtime_feature == "web.navigation.query.empty"
                && operation.canonical.rsplit_once("::").map(|(_, name)| name) == Some(member)
        });
    }
    let is_http_method = matches!(type_ref, CoreType::ExternalNamed { canonical, .. } if canonical == "std/http::Method");
    if !is_http_method || !name.contains('.') {
        return None;
    }
    let member = name.rsplit('.').next()?;
    OPERATIONS.iter().copied().find(|operation| {
        operation.runtime_feature.starts_with("http-client.method.")
            && operation.canonical.rsplit_once("::").map(|(_, name)| name) == Some(member)
    })
}

pub(crate) fn runtime_provider_service_operation_for_feature(
    feature: &str,
) -> Option<RuntimeProviderServiceOperation> {
    OPERATIONS
        .iter()
        .copied()
        .find(|operation| operation.runtime_feature == feature)
}
use crate::CoreType;
