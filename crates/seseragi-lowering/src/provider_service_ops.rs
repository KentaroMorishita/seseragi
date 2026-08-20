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
        "std/http/server::jsonResponse",
        "http-server.jsonResponse",
        "_ssrg_http_server_jsonResponse",
        "@seseragi/runtime/http-server",
        "jsonResponse"
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
];

pub(crate) fn runtime_provider_service_operation(
    canonical: &str,
) -> Option<RuntimeProviderServiceOperation> {
    OPERATIONS
        .iter()
        .copied()
        .find(|operation| operation.canonical == canonical)
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
