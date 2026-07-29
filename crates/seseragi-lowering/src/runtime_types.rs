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
        canonical: "std/prelude::Debug",
        runtime_feature: "core.debug.dictionary",
        module: "@seseragi/runtime/show",
        export_name: "Debug",
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
        canonical: "std/number::RoundingMode",
        runtime_feature: "core.number.rounding-mode",
        module: "@seseragi/runtime/number",
        export_name: "RoundingMode",
    },
    RuntimeTypeImport {
        canonical: "std/int::IntParseError",
        runtime_feature: "core.int.parse-error",
        module: "@seseragi/runtime/int",
        export_name: "IntParseError",
    },
    RuntimeTypeImport {
        canonical: "std/int::IntDivisionError",
        runtime_feature: "core.int.division-error",
        module: "@seseragi/runtime/int",
        export_name: "IntDivisionError",
    },
    RuntimeTypeImport {
        canonical: "std/int::IntPowerError",
        runtime_feature: "core.int.power-error",
        module: "@seseragi/runtime/int",
        export_name: "IntPowerError",
    },
    RuntimeTypeImport {
        canonical: "std/float::FloatParseError",
        runtime_feature: "core.float64.parse-error",
        module: "@seseragi/runtime/float",
        export_name: "FloatParseError",
    },
    RuntimeTypeImport {
        canonical: "std/float::FloatConversionError",
        runtime_feature: "core.float64.conversion-error",
        module: "@seseragi/runtime/float",
        export_name: "FloatConversionError",
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
        canonical: "std/web/html::Tag",
        runtime_feature: "web.html.tag-type",
        module: "@seseragi/runtime/html",
        export_name: "Tag",
    },
    RuntimeTypeImport {
        canonical: "std/web/html::Attribute",
        runtime_feature: "web.html.attribute-type",
        module: "@seseragi/runtime/html",
        export_name: "Attribute",
    },
    RuntimeTypeImport {
        canonical: "std/web/html::HtmlBuildError",
        runtime_feature: "web.html.build-error-type",
        module: "@seseragi/runtime/html",
        export_name: "HtmlBuildError",
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
        canonical: "std/web/html::KeyboardEvent",
        runtime_feature: "web.html.keyboard-event-type",
        module: "@seseragi/runtime/html",
        export_name: "KeyboardEvent",
    },
    RuntimeTypeImport {
        canonical: "std/web/html::MouseEvent",
        runtime_feature: "web.html.mouse-event-type",
        module: "@seseragi/runtime/html",
        export_name: "MouseEvent",
    },
    RuntimeTypeImport {
        canonical: "std/web/html::PointerEvent",
        runtime_feature: "web.html.pointer-event-type",
        module: "@seseragi/runtime/html",
        export_name: "PointerEvent",
    },
    RuntimeTypeImport {
        canonical: "std/web/html::ScrollEvent",
        runtime_feature: "web.html.scroll-event-type",
        module: "@seseragi/runtime/html",
        export_name: "ScrollEvent",
    },
    RuntimeTypeImport {
        canonical: "std/web/html::EventAction",
        runtime_feature: "web.html.event-action-type",
        module: "@seseragi/runtime/html",
        export_name: "EventAction",
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

        let debug = RuntimeTypeImport {
            canonical: "std/prelude::Debug",
            runtime_feature: "core.debug.dictionary",
            module: "@seseragi/runtime/show",
            export_name: "Debug",
        };
        assert_eq!(runtime_type_import("std/prelude::Debug"), Some(debug));
        assert_eq!(
            runtime_type_import_for_feature("core.debug.dictionary"),
            Some(debug)
        );
    }

    #[test]
    fn resolves_numeric_error_and_rounding_types() {
        for (canonical, feature, module, export_name) in [
            (
                "std/number::RoundingMode",
                "core.number.rounding-mode",
                "@seseragi/runtime/number",
                "RoundingMode",
            ),
            (
                "std/int::IntParseError",
                "core.int.parse-error",
                "@seseragi/runtime/int",
                "IntParseError",
            ),
            (
                "std/int::IntDivisionError",
                "core.int.division-error",
                "@seseragi/runtime/int",
                "IntDivisionError",
            ),
            (
                "std/int::IntPowerError",
                "core.int.power-error",
                "@seseragi/runtime/int",
                "IntPowerError",
            ),
            (
                "std/float::FloatParseError",
                "core.float64.parse-error",
                "@seseragi/runtime/float",
                "FloatParseError",
            ),
            (
                "std/float::FloatConversionError",
                "core.float64.conversion-error",
                "@seseragi/runtime/float",
                "FloatConversionError",
            ),
        ] {
            let expected = RuntimeTypeImport {
                canonical,
                runtime_feature: feature,
                module,
                export_name,
            };
            assert_eq!(runtime_type_import(canonical), Some(expected));
            assert_eq!(runtime_type_import_for_feature(feature), Some(expected));
        }
    }
}
