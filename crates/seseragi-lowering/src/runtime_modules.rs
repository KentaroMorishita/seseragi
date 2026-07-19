const RUNTIME_PROVIDED_MODULES: &[&str] = &["std/web/html", "std/web/dom", "std/signal"];

pub(crate) fn is_runtime_provided_module(module: &str) -> bool {
    RUNTIME_PROVIDED_MODULES.contains(&module)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_compiler_owned_runtime_modules() {
        assert!(is_runtime_provided_module("std/web/html"));
        assert!(is_runtime_provided_module("std/web/dom"));
        assert!(is_runtime_provided_module("std/signal"));
        assert!(!is_runtime_provided_module("app/domain"));
    }
}
