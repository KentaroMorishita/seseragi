#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RuntimeHttpServerOperation {
    pub(crate) canonical: &'static str,
    pub(crate) runtime_feature: &'static str,
    pub(crate) local_name: &'static str,
    pub(crate) module: &'static str,
    pub(crate) export_name: &'static str,
    pub(crate) source_map_name: &'static str,
    pub(crate) type_argument_sources: &'static [usize],
}

const MODULE: &str = "@seseragi/runtime/http-server";

macro_rules! operation {
    ($name:literal) => {
        RuntimeHttpServerOperation {
            canonical: concat!("std/http/server::", $name),
            runtime_feature: concat!("http-server.", $name),
            local_name: concat!("_ssrg_http_server_", $name),
            module: MODULE,
            export_name: $name,
            source_map_name: $name,
            type_argument_sources: &[],
        }
    };
}

const OPERATIONS: &[RuntimeHttpServerOperation] = &[
    operation!("jsonResponse"),
    operation!("errorMessage"),
    operation!("listen"),
    operation!("serveOnce"),
    operation!("close"),
];

pub(crate) fn runtime_http_server_operation(canonical: &str) -> Option<RuntimeHttpServerOperation> {
    OPERATIONS
        .iter()
        .copied()
        .find(|operation| operation.canonical == canonical)
}

pub(crate) fn runtime_http_server_operation_for_feature(
    feature: &str,
) -> Option<RuntimeHttpServerOperation> {
    OPERATIONS
        .iter()
        .copied()
        .find(|operation| operation.runtime_feature == feature)
}
