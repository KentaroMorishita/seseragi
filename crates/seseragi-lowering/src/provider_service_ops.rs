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
    ($canonical:literal, $feature:literal, $local:literal, $module:literal, $export:literal) => {
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
    operation!(
        "std/http::get",
        "http-client.get",
        "_ssrg_http_client_get",
        "@seseragi/runtime/http-client",
        "get"
    ),
    operation!(
        "std/http::status",
        "http-client.status",
        "_ssrg_http_client_status",
        "@seseragi/runtime/http-client",
        "status"
    ),
    operation!(
        "std/http::bodyText",
        "http-client.bodyText",
        "_ssrg_http_client_body_text",
        "@seseragi/runtime/http-client",
        "bodyText"
    ),
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

pub(crate) fn runtime_provider_service_operation_for_feature(
    feature: &str,
) -> Option<RuntimeProviderServiceOperation> {
    OPERATIONS
        .iter()
        .copied()
        .find(|operation| operation.runtime_feature == feature)
}
