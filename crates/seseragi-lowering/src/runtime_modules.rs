const RUNTIME_PROVIDED_MODULES: &[&str] = &[
    "std/number",
    "std/int",
    "std/float",
    "std/array",
    "std/list",
    "std/bytes",
    "std/text",
    "std/web/html",
    "std/web/dom",
    "std/signal",
    "std/clock",
    "std/time",
    "std/http",
    "std/http/server",
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
        assert!(is_runtime_provided_module("std/web/dom"));
        assert!(is_runtime_provided_module("std/signal"));
        assert!(is_runtime_provided_module("std/array"));
        assert!(is_runtime_provided_module("std/list"));
        assert!(is_runtime_provided_module("std/bytes"));
        assert!(is_runtime_provided_module("std/text"));
        assert!(is_runtime_provided_module("std/number"));
        assert!(is_runtime_provided_module("std/int"));
        assert!(is_runtime_provided_module("std/float"));
        assert!(is_runtime_provided_module("std/http/server"));
        assert!(is_runtime_provided_module("std/clock"));
        assert!(is_runtime_provided_module("std/time"));
        assert!(is_runtime_provided_module("std/http"));
        assert!(!is_runtime_provided_module("app/domain"));
    }
}
