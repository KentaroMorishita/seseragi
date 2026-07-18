#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RuntimePreludeDictionary {
    pub(crate) semantic_identity: &'static str,
    pub(crate) runtime_feature: &'static str,
    pub(crate) local_name: &'static str,
    pub(crate) module: &'static str,
    pub(crate) export_name: &'static str,
    pub(crate) source_map_name: &'static str,
}

const RUNTIME_PRELUDE_DICTIONARIES: &[RuntimePreludeDictionary] = &[
    RuntimePreludeDictionary {
        semantic_identity: "std/maybe::Functor",
        runtime_feature: "core.maybe.functor",
        local_name: "_ssrg_maybe_functor",
        module: "@seseragi/runtime/sum",
        export_name: "maybeFunctor",
        source_map_name: "maybeFunctor",
    },
    RuntimePreludeDictionary {
        semantic_identity: "std/maybe::Applicative",
        runtime_feature: "core.maybe.applicative",
        local_name: "_ssrg_maybe_applicative",
        module: "@seseragi/runtime/sum",
        export_name: "maybeApplicative",
        source_map_name: "maybeApplicative",
    },
    RuntimePreludeDictionary {
        semantic_identity: "std/maybe::Monad",
        runtime_feature: "core.maybe.monad",
        local_name: "_ssrg_maybe_monad",
        module: "@seseragi/runtime/sum",
        export_name: "maybeMonad",
        source_map_name: "maybeMonad",
    },
    RuntimePreludeDictionary {
        semantic_identity: "std/either::Functor",
        runtime_feature: "core.either.functor",
        local_name: "_ssrg_either_functor",
        module: "@seseragi/runtime/sum",
        export_name: "eitherFunctor",
        source_map_name: "eitherFunctor",
    },
    RuntimePreludeDictionary {
        semantic_identity: "std/either::Applicative",
        runtime_feature: "core.either.applicative",
        local_name: "_ssrg_either_applicative",
        module: "@seseragi/runtime/sum",
        export_name: "eitherApplicative",
        source_map_name: "eitherApplicative",
    },
    RuntimePreludeDictionary {
        semantic_identity: "std/either::Monad",
        runtime_feature: "core.either.monad",
        local_name: "_ssrg_either_monad",
        module: "@seseragi/runtime/sum",
        export_name: "eitherMonad",
        source_map_name: "eitherMonad",
    },
];

pub(crate) fn runtime_prelude_dictionary_for_feature(
    feature: &str,
) -> Option<RuntimePreludeDictionary> {
    RUNTIME_PRELUDE_DICTIONARIES
        .iter()
        .copied()
        .find(|dictionary| dictionary.runtime_feature == feature)
}

pub(crate) fn runtime_prelude_dictionary_for_identity(
    identity: &str,
) -> Option<RuntimePreludeDictionary> {
    RUNTIME_PRELUDE_DICTIONARIES
        .iter()
        .copied()
        .find(|dictionary| dictionary.semantic_identity == identity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_the_standard_maybe_and_either_dictionary_families() {
        for (identity, feature, export_name) in [
            ("std/maybe::Functor", "core.maybe.functor", "maybeFunctor"),
            (
                "std/maybe::Applicative",
                "core.maybe.applicative",
                "maybeApplicative",
            ),
            ("std/maybe::Monad", "core.maybe.monad", "maybeMonad"),
            (
                "std/either::Functor",
                "core.either.functor",
                "eitherFunctor",
            ),
            (
                "std/either::Applicative",
                "core.either.applicative",
                "eitherApplicative",
            ),
            ("std/either::Monad", "core.either.monad", "eitherMonad"),
        ] {
            let dictionary = runtime_prelude_dictionary_for_feature(feature).unwrap();
            assert_eq!(dictionary.module, "@seseragi/runtime/sum");
            assert_eq!(dictionary.export_name, export_name);
            assert_eq!(dictionary.source_map_name, export_name);
            assert_eq!(
                runtime_prelude_dictionary_for_identity(identity),
                Some(dictionary)
            );
        }
    }

    #[test]
    fn rejects_unknown_prelude_dictionaries() {
        assert!(runtime_prelude_dictionary_for_feature("core.result.monad").is_none());
        assert!(runtime_prelude_dictionary_for_identity("fixture/local::Monad").is_none());
    }
}
