const RUNTIME_PROVIDED_MODULES: &[&str] = &[
    "std/number",
    "std/int",
    "std/float",
    "std/array",
    "std/list",
    "std/non-empty-list",
    "std/bytes",
    "std/json",
    "std/console",
    "std/log",
    "std/stdin",
    "std/effect",
    "std/test",
    "std/deferred",
    "std/queue",
    "std/semaphore",
    "std/ref",
    "std/stream",
    "std/text",
    "std/path",
    "std/process",
    "std/child-process",
    "std/random",
    "std/entropy",
    "std/fs",
    "std/web/html",
    "std/web/file",
    "std/web/navigation",
    "std/web/storage",
    "std/web/dom",
    "std/signal",
    "std/clock",
    "std/time",
    "std/http",
    "std/http/server",
    "std/http/multipart",
    "std/sse",
    "std/websocket",
    "std/websocket/server",
];

/// Returns every standard module with a concrete lowering/runtime connection.
///
/// The conformance parity gate compares this implementation inventory with the
/// canonical project registry so an importable module cannot remain green
/// after its backend connection is removed.
pub fn runtime_provided_modules() -> &'static [&'static str] {
    RUNTIME_PROVIDED_MODULES
}

pub(crate) fn is_runtime_provided_module(module: &str) -> bool {
    runtime_provided_modules().contains(&module)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_compiler_owned_runtime_modules() {
        assert!(is_runtime_provided_module("std/web/html"));
        assert!(is_runtime_provided_module("std/web/file"));
        assert!(is_runtime_provided_module("std/web/dom"));
        assert!(is_runtime_provided_module("std/signal"));
        assert!(is_runtime_provided_module("std/array"));
        assert!(is_runtime_provided_module("std/list"));
        assert!(is_runtime_provided_module("std/non-empty-list"));
        assert!(is_runtime_provided_module("std/bytes"));
        assert!(is_runtime_provided_module("std/json"));
        assert!(is_runtime_provided_module("std/effect"));
        assert!(is_runtime_provided_module("std/deferred"));
        assert!(is_runtime_provided_module("std/queue"));
        assert!(is_runtime_provided_module("std/semaphore"));
        assert!(is_runtime_provided_module("std/ref"));
        assert!(is_runtime_provided_module("std/stream"));
        assert!(is_runtime_provided_module("std/text"));
        assert!(is_runtime_provided_module("std/path"));
        assert!(is_runtime_provided_module("std/process"));
        assert!(is_runtime_provided_module("std/child-process"));
        assert!(is_runtime_provided_module("std/random"));
        assert!(is_runtime_provided_module("std/entropy"));
        assert!(is_runtime_provided_module("std/fs"));
        assert!(is_runtime_provided_module("std/number"));
        assert!(is_runtime_provided_module("std/int"));
        assert!(is_runtime_provided_module("std/float"));
        assert!(is_runtime_provided_module("std/http/server"));
        assert!(is_runtime_provided_module("std/http/multipart"));
        assert!(is_runtime_provided_module("std/sse"));
        assert!(is_runtime_provided_module("std/websocket"));
        assert!(is_runtime_provided_module("std/websocket/server"));
        assert!(is_runtime_provided_module("std/clock"));
        assert!(is_runtime_provided_module("std/time"));
        assert!(is_runtime_provided_module("std/http"));
        assert!(!is_runtime_provided_module("app/domain"));
    }
}
