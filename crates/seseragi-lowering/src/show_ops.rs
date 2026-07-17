#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RuntimeShowDictionary {
    pub(crate) semantic_identity: &'static str,
    pub(crate) runtime_feature: &'static str,
    pub(crate) local_name: &'static str,
    pub(crate) module: &'static str,
    pub(crate) export_name: &'static str,
    pub(crate) source_map_name: &'static str,
}

const RUNTIME_SHOW_DICTIONARIES: &[RuntimeShowDictionary] = &[
    RuntimeShowDictionary {
        semantic_identity: "Show<std/prelude::Int>",
        runtime_feature: "core.int64.show",
        local_name: "_ssrg_show_intShow",
        module: "@seseragi/runtime/show",
        export_name: "intShow",
        source_map_name: "intShow",
    },
    RuntimeShowDictionary {
        semantic_identity: "Show<std/prelude::String>",
        runtime_feature: "core.string.show",
        local_name: "_ssrg_show_stringShow",
        module: "@seseragi/runtime/show",
        export_name: "stringShow",
        source_map_name: "stringShow",
    },
    RuntimeShowDictionary {
        semantic_identity: "Show<std/prelude::ConsoleError>",
        runtime_feature: "effect.console.error.show",
        local_name: "_ssrg_show_consoleErrorShow",
        module: "@seseragi/runtime/show",
        export_name: "consoleErrorShow",
        source_map_name: "consoleErrorShow",
    },
    RuntimeShowDictionary {
        semantic_identity: "Show<std/prelude::StdinError>",
        runtime_feature: "effect.stdin.error.show",
        local_name: "_ssrg_show_stdinErrorShow",
        module: "@seseragi/runtime/show",
        export_name: "stdinErrorShow",
        source_map_name: "stdinErrorShow",
    },
];

pub(crate) fn runtime_show_dictionary_for_feature(feature: &str) -> Option<RuntimeShowDictionary> {
    RUNTIME_SHOW_DICTIONARIES
        .iter()
        .copied()
        .find(|dictionary| dictionary.runtime_feature == feature)
}

pub(crate) fn runtime_show_dictionary_for_identity(
    identity: &str,
) -> Option<RuntimeShowDictionary> {
    RUNTIME_SHOW_DICTIONARIES
        .iter()
        .copied()
        .find(|dictionary| dictionary.semantic_identity == identity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_the_complete_standard_show_dictionary_family() {
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
        ] {
            let dictionary = runtime_show_dictionary_for_feature(feature).unwrap();
            assert_eq!(dictionary.local_name, local_name);
            assert_eq!(dictionary.module, "@seseragi/runtime/show");
            assert_eq!(dictionary.export_name, export_name);
            assert_eq!(dictionary.source_map_name, export_name);
            assert_eq!(
                runtime_show_dictionary_for_identity(identity),
                Some(dictionary)
            );
        }
    }

    #[test]
    fn rejects_unknown_show_dictionary_features() {
        assert!(runtime_show_dictionary_for_feature("core.float64.show").is_none());
        assert!(runtime_show_dictionary_for_identity("Show<fixture/local::Detail>").is_none());
    }
}
