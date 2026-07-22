#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RuntimeTypeImport {
    pub(crate) canonical: &'static str,
    pub(crate) runtime_feature: &'static str,
    pub(crate) module: &'static str,
    pub(crate) export_name: &'static str,
}

const RUNTIME_TYPE_IMPORTS: &[RuntimeTypeImport] = &[
    RuntimeTypeImport {
        canonical: "std/prelude::Show",
        runtime_feature: "core.show.dictionary",
        module: "@seseragi/runtime/show",
        export_name: "Show",
    },
    RuntimeTypeImport {
        canonical: "std/prelude::Console",
        runtime_feature: "effect.console.service",
        module: "@seseragi/runtime/console",
        export_name: "Console",
    },
    RuntimeTypeImport {
        canonical: "std/prelude::ConsoleError",
        runtime_feature: "effect.console.error",
        module: "@seseragi/runtime/console",
        export_name: "ConsoleError",
    },
    RuntimeTypeImport {
        canonical: "std/prelude::Stdin",
        runtime_feature: "effect.stdin.service",
        module: "@seseragi/runtime/stdin",
        export_name: "Stdin",
    },
    RuntimeTypeImport {
        canonical: "std/prelude::StdinError",
        runtime_feature: "effect.stdin.error",
        module: "@seseragi/runtime/stdin",
        export_name: "StdinError",
    },
    RuntimeTypeImport {
        canonical: "std/prelude::Effect",
        runtime_feature: "effect.core.type",
        module: "@seseragi/runtime/effect",
        export_name: "Effect",
    },
    RuntimeTypeImport {
        canonical: "std/prelude::Iterator",
        runtime_feature: "core.iterator",
        module: "@seseragi/runtime/iterator",
        export_name: "Iterator",
    },
    RuntimeTypeImport {
        canonical: "std/prelude::List",
        runtime_feature: "core.list",
        module: "@seseragi/runtime/list",
        export_name: "List",
    },
    RuntimeTypeImport {
        canonical: "std/web/html::Html",
        runtime_feature: "web.html.type",
        module: "@seseragi/runtime/html",
        export_name: "Html",
    },
    RuntimeTypeImport {
        canonical: "std/web/html::Style",
        runtime_feature: "web.html.style-type",
        module: "@seseragi/runtime/html",
        export_name: "Style",
    },
    RuntimeTypeImport {
        canonical: "std/web/html::InputEvent",
        runtime_feature: "web.html.input-event-type",
        module: "@seseragi/runtime/html",
        export_name: "InputEvent",
    },
    RuntimeTypeImport {
        canonical: "std/web/html::ChangeEvent",
        runtime_feature: "web.html.change-event-type",
        module: "@seseragi/runtime/html",
        export_name: "ChangeEvent",
    },
    RuntimeTypeImport {
        canonical: "std/web/dom::Dom",
        runtime_feature: "web.dom.service",
        module: "@seseragi/runtime/dom",
        export_name: "Dom",
    },
    RuntimeTypeImport {
        canonical: "std/web/dom::DomOptions",
        runtime_feature: "web.dom.options",
        module: "@seseragi/runtime/dom",
        export_name: "DomOptions",
    },
    RuntimeTypeImport {
        canonical: "std/web/dom::DomTarget",
        runtime_feature: "web.dom.target",
        module: "@seseragi/runtime/dom",
        export_name: "DomTarget",
    },
    RuntimeTypeImport {
        canonical: "std/web/dom::DomError",
        runtime_feature: "web.dom.error",
        module: "@seseragi/runtime/dom",
        export_name: "DomError",
    },
    RuntimeTypeImport {
        canonical: "std/web/dom::DomRuntimeError",
        runtime_feature: "web.dom.runtime-error",
        module: "@seseragi/runtime/dom",
        export_name: "DomRuntimeError",
    },
    RuntimeTypeImport {
        canonical: "std/signal::Signal",
        runtime_feature: "signal.type",
        module: "@seseragi/runtime/signal",
        export_name: "Signal",
    },
    RuntimeTypeImport {
        canonical: "std/signal::MutableSignal",
        runtime_feature: "signal.mutable-type",
        module: "@seseragi/runtime/signal",
        export_name: "MutableSignal",
    },
    RuntimeTypeImport {
        canonical: "std/signal::SignalChange",
        runtime_feature: "signal.change-type",
        module: "@seseragi/runtime/signal",
        export_name: "SignalChange",
    },
    RuntimeTypeImport {
        canonical: "std/signal::Subscription",
        runtime_feature: "signal.subscription-type",
        module: "@seseragi/runtime/signal",
        export_name: "Subscription",
    },
];

pub(crate) fn runtime_type_import(canonical: &str) -> Option<RuntimeTypeImport> {
    RUNTIME_TYPE_IMPORTS
        .iter()
        .copied()
        .find(|type_import| type_import.canonical == canonical)
}

pub(crate) fn runtime_type_imports() -> impl Iterator<Item = RuntimeTypeImport> {
    RUNTIME_TYPE_IMPORTS.iter().copied()
}

pub(crate) fn runtime_type_import_for_feature(feature: &str) -> Option<RuntimeTypeImport> {
    RUNTIME_TYPE_IMPORTS
        .iter()
        .copied()
        .find(|type_import| type_import.runtime_feature == feature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_runtime_types_by_canonical_language_identity() {
        let type_import = runtime_type_import("std/prelude::StdinError").unwrap();

        assert_eq!(type_import.runtime_feature, "effect.stdin.error");
        assert_eq!(type_import.module, "@seseragi/runtime/stdin");
        assert_eq!(type_import.export_name, "StdinError");

        let effect = runtime_type_import("std/prelude::Effect").unwrap();
        assert_eq!(effect.runtime_feature, "effect.core.type");
        assert_eq!(effect.module, "@seseragi/runtime/effect");
        assert_eq!(effect.export_name, "Effect");
    }

    #[test]
    fn does_not_treat_local_spelling_as_a_runtime_type_identity() {
        assert!(runtime_type_import("StdinError").is_none());
        assert!(runtime_type_import("artifact/domain::StdinError").is_none());
    }

    #[test]
    fn resolves_show_dictionary_type_by_identity_and_feature() {
        let expected = RuntimeTypeImport {
            canonical: "std/prelude::Show",
            runtime_feature: "core.show.dictionary",
            module: "@seseragi/runtime/show",
            export_name: "Show",
        };

        assert_eq!(runtime_type_import("std/prelude::Show"), Some(expected));
        assert_eq!(
            runtime_type_import_for_feature("core.show.dictionary"),
            Some(expected)
        );
    }
}
