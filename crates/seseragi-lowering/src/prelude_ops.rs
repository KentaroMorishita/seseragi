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
        semantic_identity: "std/int::Zero",
        runtime_feature: "core.int64.zero-dictionary",
        local_name: "_ssrg_int_zero",
        module: "@seseragi/runtime/int64",
        export_name: "intZero",
        source_map_name: "intZero",
    },
    RuntimePreludeDictionary {
        semantic_identity: "std/int::Add",
        runtime_feature: "core.int64.add-dictionary",
        local_name: "_ssrg_int_add",
        module: "@seseragi/runtime/int64",
        export_name: "intAdd",
        source_map_name: "intAdd",
    },
    RuntimePreludeDictionary {
        semantic_identity: "std/string::Semigroup",
        runtime_feature: "core.string.semigroup",
        local_name: "_ssrg_string_semigroup",
        module: "@seseragi/runtime/string",
        export_name: "stringSemigroup",
        source_map_name: "stringSemigroup",
    },
    RuntimePreludeDictionary {
        semantic_identity: "std/string::Monoid",
        runtime_feature: "core.string.monoid",
        local_name: "_ssrg_string_monoid",
        module: "@seseragi/runtime/string",
        export_name: "stringMonoid",
        source_map_name: "stringMonoid",
    },
    RuntimePreludeDictionary {
        semantic_identity: "std/array::Semigroup",
        runtime_feature: "core.array.semigroup",
        local_name: "_ssrg_array_semigroup",
        module: "@seseragi/runtime/array",
        export_name: "arraySemigroup",
        source_map_name: "arraySemigroup",
    },
    RuntimePreludeDictionary {
        semantic_identity: "std/array::Monoid",
        runtime_feature: "core.array.monoid",
        local_name: "_ssrg_array_monoid",
        module: "@seseragi/runtime/array",
        export_name: "arrayMonoid",
        source_map_name: "arrayMonoid",
    },
    RuntimePreludeDictionary {
        semantic_identity: "std/list::Semigroup",
        runtime_feature: "core.list.semigroup",
        local_name: "_ssrg_list_semigroup",
        module: "@seseragi/runtime/list",
        export_name: "listSemigroup",
        source_map_name: "listSemigroup",
    },
    RuntimePreludeDictionary {
        semantic_identity: "std/list::Monoid",
        runtime_feature: "core.list.monoid",
        local_name: "_ssrg_list_monoid",
        module: "@seseragi/runtime/list",
        export_name: "listMonoid",
        source_map_name: "listMonoid",
    },
    RuntimePreludeDictionary {
        semantic_identity: "std/array::Iterable",
        runtime_feature: "core.array.iterable",
        local_name: "_ssrg_array_iterable",
        module: "@seseragi/runtime/array",
        export_name: "arrayIterable",
        source_map_name: "arrayIterable",
    },
    RuntimePreludeDictionary {
        semantic_identity: "std/list::Iterable",
        runtime_feature: "core.list.iterable",
        local_name: "_ssrg_list_iterable",
        module: "@seseragi/runtime/list",
        export_name: "listIterable",
        source_map_name: "listIterable",
    },
    RuntimePreludeDictionary {
        semantic_identity: "std/range::Iterable",
        runtime_feature: "core.range.iterable",
        local_name: "_ssrg_range_iterable",
        module: "@seseragi/runtime/range",
        export_name: "rangeIterable",
        source_map_name: "rangeIterable",
    },
    RuntimePreludeDictionary {
        semantic_identity: "std/array::Reducible",
        runtime_feature: "core.array.reducible",
        local_name: "_ssrg_array_reducible",
        module: "@seseragi/runtime/array",
        export_name: "arrayReducible",
        source_map_name: "arrayReducible",
    },
    RuntimePreludeDictionary {
        semantic_identity: "std/list::Reducible",
        runtime_feature: "core.list.reducible",
        local_name: "_ssrg_list_reducible",
        module: "@seseragi/runtime/list",
        export_name: "listReducible",
        source_map_name: "listReducible",
    },
    RuntimePreludeDictionary {
        semantic_identity: "std/range::Reducible",
        runtime_feature: "core.range.reducible",
        local_name: "_ssrg_range_reducible",
        module: "@seseragi/runtime/range",
        export_name: "rangeReducible",
        source_map_name: "rangeReducible",
    },
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
    RuntimePreludeDictionary {
        semantic_identity: "std/array::Functor",
        runtime_feature: "core.array.functor",
        local_name: "_ssrg_array_functor",
        module: "@seseragi/runtime/array",
        export_name: "arrayFunctor",
        source_map_name: "arrayFunctor",
    },
    RuntimePreludeDictionary {
        semantic_identity: "std/array::Applicative",
        runtime_feature: "core.array.applicative",
        local_name: "_ssrg_array_applicative",
        module: "@seseragi/runtime/array",
        export_name: "arrayApplicative",
        source_map_name: "arrayApplicative",
    },
    RuntimePreludeDictionary {
        semantic_identity: "std/array::Monad",
        runtime_feature: "core.array.monad",
        local_name: "_ssrg_array_monad",
        module: "@seseragi/runtime/array",
        export_name: "arrayMonad",
        source_map_name: "arrayMonad",
    },
    RuntimePreludeDictionary {
        semantic_identity: "std/list::Functor",
        runtime_feature: "core.list.functor",
        local_name: "_ssrg_list_functor",
        module: "@seseragi/runtime/list",
        export_name: "listFunctor",
        source_map_name: "listFunctor",
    },
    RuntimePreludeDictionary {
        semantic_identity: "std/list::Applicative",
        runtime_feature: "core.list.applicative",
        local_name: "_ssrg_list_applicative",
        module: "@seseragi/runtime/list",
        export_name: "listApplicative",
        source_map_name: "listApplicative",
    },
    RuntimePreludeDictionary {
        semantic_identity: "std/list::Monad",
        runtime_feature: "core.list.monad",
        local_name: "_ssrg_list_monad",
        module: "@seseragi/runtime/list",
        export_name: "listMonad",
        source_map_name: "listMonad",
    },
    RuntimePreludeDictionary {
        semantic_identity: "std/effect::Functor",
        runtime_feature: "effect.core.functor",
        local_name: "_ssrg_effect_functor",
        module: "@seseragi/runtime/effect",
        export_name: "effectFunctor",
        source_map_name: "effectFunctor",
    },
    RuntimePreludeDictionary {
        semantic_identity: "std/effect::Applicative",
        runtime_feature: "effect.core.applicative",
        local_name: "_ssrg_effect_applicative",
        module: "@seseragi/runtime/effect",
        export_name: "effectApplicative",
        source_map_name: "effectApplicative",
    },
    RuntimePreludeDictionary {
        semantic_identity: "std/effect::Monad",
        runtime_feature: "effect.core.monad",
        local_name: "_ssrg_effect_monad",
        module: "@seseragi/runtime/effect",
        export_name: "effectMonad",
        source_map_name: "effectMonad",
    },
    RuntimePreludeDictionary {
        semantic_identity: "std/signal::Functor",
        runtime_feature: "signal.functor",
        local_name: "_ssrg_signal_functor",
        module: "@seseragi/runtime/signal",
        export_name: "signalFunctor",
        source_map_name: "signalFunctor",
    },
    RuntimePreludeDictionary {
        semantic_identity: "std/signal::Applicative",
        runtime_feature: "signal.applicative",
        local_name: "_ssrg_signal_applicative",
        module: "@seseragi/runtime/signal",
        export_name: "signalApplicative",
        source_map_name: "signalApplicative",
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
    fn maps_the_standard_prelude_dictionary_families() {
        for (identity, feature, module, export_name) in [
            (
                "std/int::Zero",
                "core.int64.zero-dictionary",
                "@seseragi/runtime/int64",
                "intZero",
            ),
            (
                "std/int::Add",
                "core.int64.add-dictionary",
                "@seseragi/runtime/int64",
                "intAdd",
            ),
            (
                "std/string::Semigroup",
                "core.string.semigroup",
                "@seseragi/runtime/string",
                "stringSemigroup",
            ),
            (
                "std/string::Monoid",
                "core.string.monoid",
                "@seseragi/runtime/string",
                "stringMonoid",
            ),
            (
                "std/array::Semigroup",
                "core.array.semigroup",
                "@seseragi/runtime/array",
                "arraySemigroup",
            ),
            (
                "std/array::Monoid",
                "core.array.monoid",
                "@seseragi/runtime/array",
                "arrayMonoid",
            ),
            (
                "std/list::Semigroup",
                "core.list.semigroup",
                "@seseragi/runtime/list",
                "listSemigroup",
            ),
            (
                "std/list::Monoid",
                "core.list.monoid",
                "@seseragi/runtime/list",
                "listMonoid",
            ),
            (
                "std/array::Iterable",
                "core.array.iterable",
                "@seseragi/runtime/array",
                "arrayIterable",
            ),
            (
                "std/list::Iterable",
                "core.list.iterable",
                "@seseragi/runtime/list",
                "listIterable",
            ),
            (
                "std/range::Iterable",
                "core.range.iterable",
                "@seseragi/runtime/range",
                "rangeIterable",
            ),
            (
                "std/array::Reducible",
                "core.array.reducible",
                "@seseragi/runtime/array",
                "arrayReducible",
            ),
            (
                "std/list::Reducible",
                "core.list.reducible",
                "@seseragi/runtime/list",
                "listReducible",
            ),
            (
                "std/range::Reducible",
                "core.range.reducible",
                "@seseragi/runtime/range",
                "rangeReducible",
            ),
            (
                "std/maybe::Functor",
                "core.maybe.functor",
                "@seseragi/runtime/sum",
                "maybeFunctor",
            ),
            (
                "std/maybe::Applicative",
                "core.maybe.applicative",
                "@seseragi/runtime/sum",
                "maybeApplicative",
            ),
            (
                "std/maybe::Monad",
                "core.maybe.monad",
                "@seseragi/runtime/sum",
                "maybeMonad",
            ),
            (
                "std/either::Functor",
                "core.either.functor",
                "@seseragi/runtime/sum",
                "eitherFunctor",
            ),
            (
                "std/either::Applicative",
                "core.either.applicative",
                "@seseragi/runtime/sum",
                "eitherApplicative",
            ),
            (
                "std/either::Monad",
                "core.either.monad",
                "@seseragi/runtime/sum",
                "eitherMonad",
            ),
            (
                "std/array::Functor",
                "core.array.functor",
                "@seseragi/runtime/array",
                "arrayFunctor",
            ),
            (
                "std/array::Applicative",
                "core.array.applicative",
                "@seseragi/runtime/array",
                "arrayApplicative",
            ),
            (
                "std/array::Monad",
                "core.array.monad",
                "@seseragi/runtime/array",
                "arrayMonad",
            ),
            (
                "std/list::Functor",
                "core.list.functor",
                "@seseragi/runtime/list",
                "listFunctor",
            ),
            (
                "std/list::Applicative",
                "core.list.applicative",
                "@seseragi/runtime/list",
                "listApplicative",
            ),
            (
                "std/list::Monad",
                "core.list.monad",
                "@seseragi/runtime/list",
                "listMonad",
            ),
            (
                "std/effect::Functor",
                "effect.core.functor",
                "@seseragi/runtime/effect",
                "effectFunctor",
            ),
            (
                "std/effect::Applicative",
                "effect.core.applicative",
                "@seseragi/runtime/effect",
                "effectApplicative",
            ),
            (
                "std/effect::Monad",
                "effect.core.monad",
                "@seseragi/runtime/effect",
                "effectMonad",
            ),
            (
                "std/signal::Functor",
                "signal.functor",
                "@seseragi/runtime/signal",
                "signalFunctor",
            ),
            (
                "std/signal::Applicative",
                "signal.applicative",
                "@seseragi/runtime/signal",
                "signalApplicative",
            ),
        ] {
            let dictionary = runtime_prelude_dictionary_for_feature(feature).unwrap();
            assert_eq!(dictionary.module, module);
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
