#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RuntimeDisplayDictionary {
    pub(crate) semantic_identity: &'static str,
    pub(crate) runtime_feature: &'static str,
    pub(crate) local_name: &'static str,
    pub(crate) module: &'static str,
    pub(crate) export_name: &'static str,
    pub(crate) source_map_name: &'static str,
}

const RUNTIME_DISPLAY_DICTIONARIES: &[RuntimeDisplayDictionary] = &[
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/prelude::Int>",
        runtime_feature: "core.int64.show",
        local_name: "_ssrg_show_intShow",
        module: "@seseragi/runtime/show",
        export_name: "intShow",
        source_map_name: "intShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/prelude::Int>",
        runtime_feature: "core.int64.debug",
        local_name: "_ssrg_debug_intDebug",
        module: "@seseragi/runtime/show",
        export_name: "intDebug",
        source_map_name: "intDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/prelude::Float>",
        runtime_feature: "core.float64.show",
        local_name: "_ssrg_show_floatShow",
        module: "@seseragi/runtime/show",
        export_name: "floatShow",
        source_map_name: "floatShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/prelude::Float>",
        runtime_feature: "core.float64.debug",
        local_name: "_ssrg_debug_floatDebug",
        module: "@seseragi/runtime/show",
        export_name: "floatDebug",
        source_map_name: "floatDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/prelude::Never>",
        runtime_feature: "core.never.show",
        local_name: "_ssrg_show_neverShow",
        module: "@seseragi/runtime/show",
        export_name: "neverShow",
        source_map_name: "neverShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/prelude::Never>",
        runtime_feature: "core.never.debug",
        local_name: "_ssrg_debug_neverDebug",
        module: "@seseragi/runtime/show",
        export_name: "neverDebug",
        source_map_name: "neverDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/prelude::String>",
        runtime_feature: "core.string.show",
        local_name: "_ssrg_show_stringShow",
        module: "@seseragi/runtime/show",
        export_name: "stringShow",
        source_map_name: "stringShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/prelude::ConsoleError>",
        runtime_feature: "effect.console.error.show",
        local_name: "_ssrg_show_consoleErrorShow",
        module: "@seseragi/runtime/show",
        export_name: "consoleErrorShow",
        source_map_name: "consoleErrorShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/prelude::StdinError>",
        runtime_feature: "effect.stdin.error.show",
        local_name: "_ssrg_show_stdinErrorShow",
        module: "@seseragi/runtime/show",
        export_name: "stdinErrorShow",
        source_map_name: "stdinErrorShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/prelude::Bool>",
        runtime_feature: "core.bool.show",
        local_name: "_ssrg_show_boolShow",
        module: "@seseragi/runtime/show",
        export_name: "boolShow",
        source_map_name: "boolShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/prelude::Unit>",
        runtime_feature: "core.unit.show",
        local_name: "_ssrg_show_unitShow",
        module: "@seseragi/runtime/show",
        export_name: "unitShow",
        source_map_name: "unitShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/prelude::Char>",
        runtime_feature: "core.char.show",
        local_name: "_ssrg_show_charShow",
        module: "@seseragi/runtime/show",
        export_name: "charShow",
        source_map_name: "charShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/prelude::String>",
        runtime_feature: "core.string.debug",
        local_name: "_ssrg_debug_stringDebug",
        module: "@seseragi/runtime/show",
        export_name: "stringDebug",
        source_map_name: "stringDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/prelude::Bool>",
        runtime_feature: "core.bool.debug",
        local_name: "_ssrg_debug_boolDebug",
        module: "@seseragi/runtime/show",
        export_name: "boolDebug",
        source_map_name: "boolDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/prelude::Unit>",
        runtime_feature: "core.unit.debug",
        local_name: "_ssrg_debug_unitDebug",
        module: "@seseragi/runtime/show",
        export_name: "unitDebug",
        source_map_name: "unitDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/prelude::Char>",
        runtime_feature: "core.char.debug",
        local_name: "_ssrg_debug_charDebug",
        module: "@seseragi/runtime/show",
        export_name: "charDebug",
        source_map_name: "charDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "std/array::Show",
        runtime_feature: "core.array.show",
        local_name: "_ssrg_show_arrayShow",
        module: "@seseragi/runtime/show",
        export_name: "arrayShow",
        source_map_name: "arrayShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "std/array::Debug",
        runtime_feature: "core.array.debug",
        local_name: "_ssrg_debug_arrayDebug",
        module: "@seseragi/runtime/show",
        export_name: "arrayDebug",
        source_map_name: "arrayDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "std/list::Show",
        runtime_feature: "core.list.show",
        local_name: "_ssrg_show_listShow",
        module: "@seseragi/runtime/show",
        export_name: "listShow",
        source_map_name: "listShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "std/list::Debug",
        runtime_feature: "core.list.debug",
        local_name: "_ssrg_debug_listDebug",
        module: "@seseragi/runtime/show",
        export_name: "listDebug",
        source_map_name: "listDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "std/maybe::Show",
        runtime_feature: "core.maybe.show",
        local_name: "_ssrg_show_maybeShow",
        module: "@seseragi/runtime/show",
        export_name: "maybeShow",
        source_map_name: "maybeShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "std/maybe::Debug",
        runtime_feature: "core.maybe.debug",
        local_name: "_ssrg_debug_maybeDebug",
        module: "@seseragi/runtime/show",
        export_name: "maybeDebug",
        source_map_name: "maybeDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "std/either::Show",
        runtime_feature: "core.either.show",
        local_name: "_ssrg_show_eitherShow",
        module: "@seseragi/runtime/show",
        export_name: "eitherShow",
        source_map_name: "eitherShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "std/either::Debug",
        runtime_feature: "core.either.debug",
        local_name: "_ssrg_debug_eitherDebug",
        module: "@seseragi/runtime/show",
        export_name: "eitherDebug",
        source_map_name: "eitherDebug",
    },
];

pub(crate) fn runtime_display_dictionary_for_feature(
    feature: &str,
) -> Option<RuntimeDisplayDictionary> {
    RUNTIME_DISPLAY_DICTIONARIES
        .iter()
        .copied()
        .find(|dictionary| dictionary.runtime_feature == feature)
}

pub(crate) fn runtime_display_dictionary_for_identity(
    identity: &str,
) -> Option<RuntimeDisplayDictionary> {
    RUNTIME_DISPLAY_DICTIONARIES
        .iter()
        .copied()
        .find(|dictionary| dictionary.semantic_identity == identity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_the_complete_standard_display_dictionary_family() {
        for (identity, feature, local_name, export_name) in [
            (
                "Show<std/prelude::Int>",
                "core.int64.show",
                "_ssrg_show_intShow",
                "intShow",
            ),
            (
                "Show<std/prelude::String>",
                "core.string.show",
                "_ssrg_show_stringShow",
                "stringShow",
            ),
            (
                "Debug<std/prelude::Int>",
                "core.int64.debug",
                "_ssrg_debug_intDebug",
                "intDebug",
            ),
            (
                "Show<std/prelude::Float>",
                "core.float64.show",
                "_ssrg_show_floatShow",
                "floatShow",
            ),
            (
                "Debug<std/prelude::Float>",
                "core.float64.debug",
                "_ssrg_debug_floatDebug",
                "floatDebug",
            ),
            (
                "Show<std/prelude::Never>",
                "core.never.show",
                "_ssrg_show_neverShow",
                "neverShow",
            ),
            (
                "Debug<std/prelude::Never>",
                "core.never.debug",
                "_ssrg_debug_neverDebug",
                "neverDebug",
            ),
            (
                "Show<std/prelude::ConsoleError>",
                "effect.console.error.show",
                "_ssrg_show_consoleErrorShow",
                "consoleErrorShow",
            ),
            (
                "Show<std/prelude::StdinError>",
                "effect.stdin.error.show",
                "_ssrg_show_stdinErrorShow",
                "stdinErrorShow",
            ),
            (
                "Show<std/prelude::Bool>",
                "core.bool.show",
                "_ssrg_show_boolShow",
                "boolShow",
            ),
            (
                "Show<std/prelude::Unit>",
                "core.unit.show",
                "_ssrg_show_unitShow",
                "unitShow",
            ),
            (
                "Show<std/prelude::Char>",
                "core.char.show",
                "_ssrg_show_charShow",
                "charShow",
            ),
            (
                "Debug<std/prelude::String>",
                "core.string.debug",
                "_ssrg_debug_stringDebug",
                "stringDebug",
            ),
            (
                "Debug<std/prelude::Bool>",
                "core.bool.debug",
                "_ssrg_debug_boolDebug",
                "boolDebug",
            ),
            (
                "Debug<std/prelude::Unit>",
                "core.unit.debug",
                "_ssrg_debug_unitDebug",
                "unitDebug",
            ),
            (
                "Debug<std/prelude::Char>",
                "core.char.debug",
                "_ssrg_debug_charDebug",
                "charDebug",
            ),
            (
                "std/array::Show",
                "core.array.show",
                "_ssrg_show_arrayShow",
                "arrayShow",
            ),
            (
                "std/array::Debug",
                "core.array.debug",
                "_ssrg_debug_arrayDebug",
                "arrayDebug",
            ),
            (
                "std/list::Show",
                "core.list.show",
                "_ssrg_show_listShow",
                "listShow",
            ),
            (
                "std/list::Debug",
                "core.list.debug",
                "_ssrg_debug_listDebug",
                "listDebug",
            ),
            (
                "std/maybe::Show",
                "core.maybe.show",
                "_ssrg_show_maybeShow",
                "maybeShow",
            ),
            (
                "std/maybe::Debug",
                "core.maybe.debug",
                "_ssrg_debug_maybeDebug",
                "maybeDebug",
            ),
            (
                "std/either::Show",
                "core.either.show",
                "_ssrg_show_eitherShow",
                "eitherShow",
            ),
            (
                "std/either::Debug",
                "core.either.debug",
                "_ssrg_debug_eitherDebug",
                "eitherDebug",
            ),
        ] {
            let dictionary = runtime_display_dictionary_for_feature(feature).unwrap();
            assert_eq!(dictionary.local_name, local_name);
            assert_eq!(dictionary.module, "@seseragi/runtime/show");
            assert_eq!(dictionary.export_name, export_name);
            assert_eq!(dictionary.source_map_name, export_name);
            assert_eq!(
                runtime_display_dictionary_for_identity(identity),
                Some(dictionary)
            );
        }
    }

    #[test]
    fn rejects_unknown_display_dictionary_features() {
        assert!(runtime_display_dictionary_for_feature("core.decimal.show").is_none());
        assert!(runtime_display_dictionary_for_identity("Show<fixture/local::Detail>").is_none());
    }
}
