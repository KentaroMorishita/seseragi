use crate::collection_ops::RuntimeCollectionOperation;

macro_rules! sum_operation {
    ($module:literal, $name:literal, $feature:literal, $export:literal, $arity:literal) => {
        sum_operation!(
            $module,
            $name,
            $feature,
            $export,
            $arity,
            "@seseragi/runtime/sum"
        )
    };
    ($module:literal, $name:literal, $feature:literal, $export:literal, $arity:literal, $runtime:literal) => {
        (
            concat!("std/", $module, "::", $name),
            RuntimeCollectionOperation {
                runtime_feature: concat!("core.", $module, ".", $feature),
                local_name: concat!("_ssrg_", $module, "_", $name),
                module: $runtime,
                export_name: $export,
                source_arity: $arity,
            },
        )
    };
}

const SUM_OPERATIONS: &[(&str, RuntimeCollectionOperation)] = &[
    sum_operation!(
        "validation",
        "Valid",
        "valid-constructor",
        "Valid",
        1,
        "@seseragi/runtime/validation"
    ),
    sum_operation!(
        "validation",
        "Invalid",
        "invalid-constructor",
        "Invalid",
        1,
        "@seseragi/runtime/validation"
    ),
    sum_operation!(
        "validation",
        "valid",
        "valid",
        "valid",
        1,
        "@seseragi/runtime/validation"
    ),
    sum_operation!(
        "validation",
        "invalid",
        "invalid",
        "invalid",
        1,
        "@seseragi/runtime/validation"
    ),
    sum_operation!(
        "validation",
        "invalidMany",
        "invalid-many",
        "invalidMany",
        1,
        "@seseragi/runtime/validation"
    ),
    sum_operation!(
        "validation",
        "fromEither",
        "from-either",
        "fromEither",
        1,
        "@seseragi/runtime/validation"
    ),
    sum_operation!(
        "validation",
        "toEither",
        "to-either",
        "toEither",
        1,
        "@seseragi/runtime/validation"
    ),
    sum_operation!("maybe", "withDefault", "with-default", "withDefault", 2),
    sum_operation!("maybe", "orElse", "or-else", "orElse", 2),
    sum_operation!("maybe", "sequence", "sequence", "maybeSequence", 1),
    sum_operation!("maybe", "traverse", "traverse", "maybeTraverse", 2),
    sum_operation!("either", "mapLeft", "map-left", "mapLeft", 2),
    sum_operation!("either", "mapRight", "map-right", "mapRight", 2),
    sum_operation!("either", "bimap", "bimap", "bimap", 3),
    sum_operation!("either", "fold", "fold", "fold", 3),
    sum_operation!("either", "swap", "swap", "swap", 1),
    sum_operation!("either", "sequence", "sequence", "eitherSequence", 1),
    sum_operation!("either", "traverse", "traverse", "eitherTraverse", 2),
];

pub(crate) fn runtime_sum_operation(name: &str) -> Option<&'static RuntimeCollectionOperation> {
    SUM_OPERATIONS
        .iter()
        .find(|(canonical, _)| *canonical == name)
        .map(|(_, operation)| operation)
}

pub(crate) fn runtime_sum_operation_for_feature(
    feature: &str,
) -> Option<RuntimeCollectionOperation> {
    SUM_OPERATIONS
        .iter()
        .map(|(_, operation)| *operation)
        .find(|operation| operation.runtime_feature == feature)
}

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
        semantic_name: "std/prelude::Product",
        runtime_feature: "core.product.constructor",
        local_name: "_ssrg_product_Product",
        module: "@seseragi/runtime/sum",
        export_name: "Product",
        source_map_name: "Product",
    },
    RuntimeSumConstructor {
        semantic_name: "std/prelude::Sum",
        runtime_feature: "core.sum.constructor",
        local_name: "_ssrg_sum_Sum",
        module: "@seseragi/runtime/sum",
        export_name: "Sum",
        source_map_name: "Sum",
    },
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
    RuntimeSumConstructor {
        semantic_name: "std/prelude::Less",
        runtime_feature: "core.ordering.less",
        local_name: "_ssrg_ordering_Less",
        module: "@seseragi/runtime/sum",
        export_name: "Less",
        source_map_name: "Less",
    },
    RuntimeSumConstructor {
        semantic_name: "std/prelude::Equal",
        runtime_feature: "core.ordering.equal",
        local_name: "_ssrg_ordering_Equal",
        module: "@seseragi/runtime/sum",
        export_name: "Equal",
        source_map_name: "Equal",
    },
    RuntimeSumConstructor {
        semantic_name: "std/prelude::Greater",
        runtime_feature: "core.ordering.greater",
        local_name: "_ssrg_ordering_Greater",
        module: "@seseragi/runtime/sum",
        export_name: "Greater",
        source_map_name: "Greater",
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
            ("std/prelude::Less", "core.ordering.less"),
            ("std/prelude::Equal", "core.ordering.equal"),
            ("std/prelude::Greater", "core.ordering.greater"),
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
