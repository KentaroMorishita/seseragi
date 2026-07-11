#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RuntimeSumConstructor {
    pub(crate) semantic_name: &'static str,
    pub(crate) runtime_feature: &'static str,
    pub(crate) local_name: &'static str,
    pub(crate) module: &'static str,
    pub(crate) export_name: &'static str,
    pub(crate) source_map_name: &'static str,
}

const RUNTIME_SUM_CONSTRUCTORS: &[RuntimeSumConstructor] = &[
    RuntimeSumConstructor {
        semantic_name: "std/prelude::Nothing",
        runtime_feature: "core.maybe.nothing",
        local_name: "_ssrg_maybe_Nothing",
        module: "@seseragi/runtime/sum",
        export_name: "Nothing",
        source_map_name: "Nothing",
    },
    RuntimeSumConstructor {
        semantic_name: "std/prelude::Just",
        runtime_feature: "core.maybe.just",
        local_name: "_ssrg_maybe_Just",
        module: "@seseragi/runtime/sum",
        export_name: "Just",
        source_map_name: "Just",
    },
    RuntimeSumConstructor {
        semantic_name: "std/prelude::Left",
        runtime_feature: "core.either.left",
        local_name: "_ssrg_either_Left",
        module: "@seseragi/runtime/sum",
        export_name: "Left",
        source_map_name: "Left",
    },
    RuntimeSumConstructor {
        semantic_name: "std/prelude::Right",
        runtime_feature: "core.either.right",
        local_name: "_ssrg_either_Right",
        module: "@seseragi/runtime/sum",
        export_name: "Right",
        source_map_name: "Right",
    },
];

pub(crate) fn runtime_sum_constructor(semantic_name: &str) -> Option<RuntimeSumConstructor> {
    RUNTIME_SUM_CONSTRUCTORS
        .iter()
        .copied()
        .find(|constructor| constructor.semantic_name == semantic_name)
}

pub(crate) fn runtime_sum_constructor_for_feature(feature: &str) -> Option<RuntimeSumConstructor> {
    RUNTIME_SUM_CONSTRUCTORS
        .iter()
        .copied()
        .find(|constructor| constructor.runtime_feature == feature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_the_complete_standard_sum_constructor_family() {
        for (semantic_name, feature) in [
            ("std/prelude::Nothing", "core.maybe.nothing"),
            ("std/prelude::Just", "core.maybe.just"),
            ("std/prelude::Left", "core.either.left"),
            ("std/prelude::Right", "core.either.right"),
        ] {
            let constructor = runtime_sum_constructor(semantic_name).unwrap();
            assert_eq!(constructor.runtime_feature, feature);
            assert_eq!(
                runtime_sum_constructor_for_feature(feature),
                Some(constructor)
            );
        }
    }

    #[test]
    fn does_not_map_local_constructor_names() {
        assert!(runtime_sum_constructor("artifact/local::Just").is_none());
    }
}
