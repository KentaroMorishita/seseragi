use crate::{CoreCallEvidence, CoreInstanceEvidence};

#[derive(Clone, Copy)]
pub(crate) struct RuntimeCollectionOperation {
    /// Restore the checked source result when the host ABI erases an HKT.
    pub(crate) result_erased: bool,
    pub(crate) runtime_feature: &'static str,
    pub(crate) local_name: &'static str,
    pub(crate) module: &'static str,
    pub(crate) export_name: &'static str,
    /// Source parameters, excluding evidence dictionaries and callable results.
    pub(crate) source_arity: usize,
}

macro_rules! sequence_operation {
    ($module:literal, $name:literal, $feature:literal, $arity:literal) => {
        sequence_operation!($module, $name, $feature, $arity, $name)
    };
    ($module:literal, $name:literal, $feature:literal, $arity:literal, $export:literal) => {
        (
            concat!("std/", $module, "::", $name),
            RuntimeCollectionOperation {
                result_erased: false,
                runtime_feature: concat!("core.", $module, ".", $feature),
                local_name: concat!("_ssrg_", $module, "_", $name),
                module: concat!("@seseragi/runtime/", $module),
                export_name: $export,
                source_arity: $arity,
            },
        )
    };
}

const ARRAY_REDUCE: RuntimeCollectionOperation = RuntimeCollectionOperation {
    result_erased: false,
    runtime_feature: "core.array.reduce",
    local_name: "_ssrg_array_reduce",
    module: "@seseragi/runtime/array",
    export_name: "reduce",
    source_arity: 3,
};

const RANGE_REDUCE: RuntimeCollectionOperation = RuntimeCollectionOperation {
    result_erased: false,
    runtime_feature: "core.range.reduce",
    local_name: "_ssrg_range_reduce",
    module: "@seseragi/runtime/range",
    export_name: "reduce",
    source_arity: 3,
};

const LIST_REDUCE: RuntimeCollectionOperation = RuntimeCollectionOperation {
    result_erased: false,
    runtime_feature: "core.list.reduce",
    local_name: "_ssrg_list_reduce",
    module: "@seseragi/runtime/list",
    export_name: "reduce",
    source_arity: 3,
};

const COLLECTION_JOIN: RuntimeCollectionOperation = RuntimeCollectionOperation {
    result_erased: false,
    runtime_feature: "core.collection.join",
    local_name: "_ssrg_collection_join",
    module: "@seseragi/runtime/collection",
    export_name: "join",
    source_arity: 2,
};

const COLLECTION_SUM: RuntimeCollectionOperation = RuntimeCollectionOperation {
    result_erased: false,
    runtime_feature: "core.collection.sum",
    local_name: "_ssrg_collection_sum",
    module: "@seseragi/runtime/collection",
    export_name: "sum",
    source_arity: 1,
};

const COLLECTION_PRODUCT: RuntimeCollectionOperation = RuntimeCollectionOperation {
    result_erased: false,
    runtime_feature: "core.collection.product",
    local_name: "_ssrg_collection_product",
    module: "@seseragi/runtime/collection",
    export_name: "product",
    source_arity: 1,
};

const COLLECTION_COMBINE: RuntimeCollectionOperation = RuntimeCollectionOperation {
    result_erased: false,
    runtime_feature: "core.collection.combine",
    local_name: "_ssrg_collection_combine",
    module: "@seseragi/runtime/collection",
    export_name: "combine",
    source_arity: 1,
};

const COLLECTION_ANY: RuntimeCollectionOperation = RuntimeCollectionOperation {
    result_erased: false,
    runtime_feature: "core.collection.any",
    local_name: "_ssrg_collection_any",
    module: "@seseragi/runtime/collection",
    export_name: "any",
    source_arity: 2,
};

const COLLECTION_ALL: RuntimeCollectionOperation = RuntimeCollectionOperation {
    result_erased: false,
    runtime_feature: "core.collection.all",
    local_name: "_ssrg_collection_all",
    module: "@seseragi/runtime/collection",
    export_name: "all",
    source_arity: 2,
};

const COLLECTION_FOR_EACH: RuntimeCollectionOperation = RuntimeCollectionOperation {
    result_erased: false,
    runtime_feature: "effect.collection.for-each",
    local_name: "_ssrg_collection_for_each",
    module: "@seseragi/runtime/collection",
    export_name: "forEach",
    source_arity: 2,
};

const ARRAY_COMPREHEND: RuntimeCollectionOperation = RuntimeCollectionOperation {
    result_erased: false,
    runtime_feature: "core.array.comprehend",
    local_name: "_ssrg_array_comprehend",
    module: "@seseragi/runtime/array",
    export_name: "collectMap",
    source_arity: 3,
};

const ARRAY_COMPREHEND_FLAT: RuntimeCollectionOperation = RuntimeCollectionOperation {
    result_erased: false,
    runtime_feature: "core.array.comprehend.flat",
    local_name: "_ssrg_array_comprehend_flat",
    module: "@seseragi/runtime/array",
    export_name: "collectFlatMap",
    source_arity: 3,
};

const RANGE_COMPREHEND: RuntimeCollectionOperation = RuntimeCollectionOperation {
    result_erased: false,
    runtime_feature: "core.range.comprehend",
    local_name: "_ssrg_range_comprehend",
    module: "@seseragi/runtime/range",
    export_name: "collectMap",
    source_arity: 3,
};

const RANGE_COMPREHEND_FLAT: RuntimeCollectionOperation = RuntimeCollectionOperation {
    result_erased: false,
    runtime_feature: "core.range.comprehend.flat",
    local_name: "_ssrg_range_comprehend_flat",
    module: "@seseragi/runtime/range",
    export_name: "collectFlatMap",
    source_arity: 3,
};

const LIST_COMPREHEND: RuntimeCollectionOperation = RuntimeCollectionOperation {
    result_erased: false,
    runtime_feature: "core.list.comprehend",
    local_name: "_ssrg_list_comprehend",
    module: "@seseragi/runtime/list",
    export_name: "collectMap",
    source_arity: 3,
};

const LIST_COMPREHEND_FLAT: RuntimeCollectionOperation = RuntimeCollectionOperation {
    result_erased: false,
    runtime_feature: "core.list.comprehend.flat",
    local_name: "_ssrg_list_comprehend_flat",
    module: "@seseragi/runtime/list",
    export_name: "collectFlatMap",
    source_arity: 3,
};

const STANDARD_COLLECTION_OPERATIONS: &[(&str, RuntimeCollectionOperation)] = &[
    sequence_operation!("collection", "Next", "reduce-step.next", 1),
    sequence_operation!("collection", "Done", "reduce-step.done", 1),
    sequence_operation!("collection", "reduceUntil", "reduce-until", 3),
    sequence_operation!(
        "collection",
        "NonPositiveSize",
        "size-error.non-positive",
        1
    ),
    sequence_operation!("array", "empty", "empty", 1),
    sequence_operation!("array", "singleton", "singleton", 1),
    sequence_operation!("array", "fromIterable", "from-iterable", 1),
    sequence_operation!("array", "reduceRight", "reduce-right", 3),
    sequence_operation!("array", "findIndex", "find-index", 2),
    sequence_operation!("array", "takeWhile", "take-while", 2),
    sequence_operation!("array", "dropWhile", "drop-while", 2),
    sequence_operation!("array", "zip", "zip", 2),
    sequence_operation!("array", "zipWith", "zip-with", 3),
    sequence_operation!("array", "unzip", "unzip", 1),
    sequence_operation!("array", "sort", "sort", 1),
    sequence_operation!("array", "sortBy", "sort-by", 2),
    sequence_operation!("array", "groupBy", "group-by", 2),
    sequence_operation!("array", "last", "last", 1),
    sequence_operation!("array", "init", "init", 1),
    sequence_operation!("array", "chunksOf", "chunks-of", 2),
    sequence_operation!("array", "windows", "windows", 2),
    sequence_operation!("list", "empty", "empty", 1),
    sequence_operation!("list", "singleton", "singleton", 1, "singletonList"),
    sequence_operation!("list", "fromIterable", "from-iterable", 1),
    sequence_operation!("list", "reduceRight", "reduce-right", 3),
    sequence_operation!("list", "findIndex", "find-index", 2),
    sequence_operation!("list", "takeWhile", "take-while", 2),
    sequence_operation!("list", "dropWhile", "drop-while", 2),
    sequence_operation!("list", "zip", "zip", 2),
    sequence_operation!("list", "zipWith", "zip-with", 3),
    sequence_operation!("list", "unzip", "unzip", 1),
    sequence_operation!("list", "sort", "sort", 1),
    sequence_operation!("list", "sortBy", "sort-by", 2),
    sequence_operation!("list", "groupBy", "group-by", 2),
    sequence_operation!("list", "last", "last", 1),
    sequence_operation!("list", "init", "init", 1),
    sequence_operation!("list", "chunksOf", "chunks-of", 2),
    sequence_operation!("list", "windows", "windows", 2),
    (
        "std/map::empty",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.map.empty",
            local_name: "_ssrg_map_empty",
            module: "@seseragi/runtime/map",
            export_name: "empty",
            source_arity: 1,
        },
    ),
    (
        "std/map::singleton",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.map.singleton",
            local_name: "_ssrg_map_singleton",
            module: "@seseragi/runtime/map",
            export_name: "singleton",
            source_arity: 2,
        },
    ),
    (
        "std/map::fromEntries",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.map.from-entries",
            local_name: "_ssrg_map_fromEntries",
            module: "@seseragi/runtime/map",
            export_name: "fromEntries",
            source_arity: 1,
        },
    ),
    (
        "std/map::get",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.map.get",
            local_name: "_ssrg_map_get",
            module: "@seseragi/runtime/map",
            export_name: "get",
            source_arity: 2,
        },
    ),
    (
        "std/map::containsKey",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.map.contains-key",
            local_name: "_ssrg_map_containsKey",
            module: "@seseragi/runtime/map",
            export_name: "containsKey",
            source_arity: 2,
        },
    ),
    (
        "std/map::insert",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.map.insert",
            local_name: "_ssrg_map_insert",
            module: "@seseragi/runtime/map",
            export_name: "insert",
            source_arity: 3,
        },
    ),
    (
        "std/map::upsert",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.map.upsert",
            local_name: "_ssrg_map_upsert",
            module: "@seseragi/runtime/map",
            export_name: "upsert",
            source_arity: 3,
        },
    ),
    (
        "std/map::remove",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.map.remove",
            local_name: "_ssrg_map_remove",
            module: "@seseragi/runtime/map",
            export_name: "remove",
            source_arity: 2,
        },
    ),
    (
        "std/map::filter",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.map.filter",
            local_name: "_ssrg_map_filter",
            module: "@seseragi/runtime/map",
            export_name: "filter",
            source_arity: 2,
        },
    ),
    (
        "std/map::mapValues",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.map.map-values",
            local_name: "_ssrg_map_mapValues",
            module: "@seseragi/runtime/map",
            export_name: "mapValues",
            source_arity: 2,
        },
    ),
    (
        "std/map::mapKeysWith",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.map.map-keys-with",
            local_name: "_ssrg_map_mapKeysWith",
            module: "@seseragi/runtime/map",
            export_name: "mapKeysWith",
            source_arity: 3,
        },
    ),
    (
        "std/map::mergeWith",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.map.merge-with",
            local_name: "_ssrg_map_mergeWith",
            module: "@seseragi/runtime/map",
            export_name: "mergeWith",
            source_arity: 3,
        },
    ),
    (
        "std/map::keys",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.map.keys",
            local_name: "_ssrg_map_keys",
            module: "@seseragi/runtime/map",
            export_name: "keys",
            source_arity: 1,
        },
    ),
    (
        "std/map::values",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.map.values",
            local_name: "_ssrg_map_values",
            module: "@seseragi/runtime/map",
            export_name: "values",
            source_arity: 1,
        },
    ),
    (
        "std/map::entries",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.map.entries",
            local_name: "_ssrg_map_entries",
            module: "@seseragi/runtime/map",
            export_name: "entries",
            source_arity: 1,
        },
    ),
    (
        "std/map::size",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.map.size",
            local_name: "_ssrg_map_size",
            module: "@seseragi/runtime/map",
            export_name: "size",
            source_arity: 1,
        },
    ),
    (
        "std/map::isEmpty",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.map.is-empty",
            local_name: "_ssrg_map_isEmpty",
            module: "@seseragi/runtime/map",
            export_name: "isEmpty",
            source_arity: 1,
        },
    ),
    (
        "std/set::empty",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.set.empty",
            local_name: "_ssrg_set_empty",
            module: "@seseragi/runtime/set",
            export_name: "empty",
            source_arity: 1,
        },
    ),
    (
        "std/set::singleton",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.set.singleton",
            local_name: "_ssrg_set_singleton",
            module: "@seseragi/runtime/set",
            export_name: "singleton",
            source_arity: 1,
        },
    ),
    (
        "std/set::fromIterable",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.set.from-iterable",
            local_name: "_ssrg_set_fromIterable",
            module: "@seseragi/runtime/set",
            export_name: "fromIterable",
            source_arity: 1,
        },
    ),
    (
        "std/set::contains",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.set.contains",
            local_name: "_ssrg_set_contains",
            module: "@seseragi/runtime/set",
            export_name: "contains",
            source_arity: 2,
        },
    ),
    (
        "std/set::insert",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.set.insert",
            local_name: "_ssrg_set_insert",
            module: "@seseragi/runtime/set",
            export_name: "insert",
            source_arity: 2,
        },
    ),
    (
        "std/set::remove",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.set.remove",
            local_name: "_ssrg_set_remove",
            module: "@seseragi/runtime/set",
            export_name: "remove",
            source_arity: 2,
        },
    ),
    (
        "std/set::filter",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.set.filter",
            local_name: "_ssrg_set_filter",
            module: "@seseragi/runtime/set",
            export_name: "filter",
            source_arity: 2,
        },
    ),
    (
        "std/set::map",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.set.map",
            local_name: "_ssrg_set_map",
            module: "@seseragi/runtime/set",
            export_name: "map",
            source_arity: 2,
        },
    ),
    (
        "std/set::union",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.set.union",
            local_name: "_ssrg_set_union",
            module: "@seseragi/runtime/set",
            export_name: "union",
            source_arity: 2,
        },
    ),
    (
        "std/set::intersection",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.set.intersection",
            local_name: "_ssrg_set_intersection",
            module: "@seseragi/runtime/set",
            export_name: "intersection",
            source_arity: 2,
        },
    ),
    (
        "std/set::difference",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.set.difference",
            local_name: "_ssrg_set_difference",
            module: "@seseragi/runtime/set",
            export_name: "difference",
            source_arity: 2,
        },
    ),
    (
        "std/set::isSubsetOf",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.set.is-subset-of",
            local_name: "_ssrg_set_isSubsetOf",
            module: "@seseragi/runtime/set",
            export_name: "isSubsetOf",
            source_arity: 2,
        },
    ),
    (
        "std/set::toArray",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.set.to-array",
            local_name: "_ssrg_set_toArray",
            module: "@seseragi/runtime/set",
            export_name: "toArray",
            source_arity: 1,
        },
    ),
    (
        "std/set::toList",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.set.to-list",
            local_name: "_ssrg_set_toList",
            module: "@seseragi/runtime/set",
            export_name: "toList",
            source_arity: 1,
        },
    ),
    (
        "std/set::size",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.set.size",
            local_name: "_ssrg_set_size",
            module: "@seseragi/runtime/set",
            export_name: "size",
            source_arity: 1,
        },
    ),
    (
        "std/set::isEmpty",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.set.is-empty",
            local_name: "_ssrg_set_isEmpty",
            module: "@seseragi/runtime/set",
            export_name: "isEmpty",
            source_arity: 1,
        },
    ),
    (
        "std/array::toList",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.array.to-list",
            local_name: "_ssrg_array_toList",
            module: "@seseragi/runtime/array",
            export_name: "toList",
            source_arity: 1,
        },
    ),
    (
        "std/array::filter",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.array.filter",
            local_name: "_ssrg_array_filter",
            module: "@seseragi/runtime/array",
            export_name: "filter",
            source_arity: 2,
        },
    ),
    (
        "std/array::filterMap",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.array.filter-map",
            local_name: "_ssrg_array_filterMap",
            module: "@seseragi/runtime/array",
            export_name: "filterMap",
            source_arity: 2,
        },
    ),
    (
        "std/array::flatMap",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.array.flat-map",
            local_name: "_ssrg_array_flatMap",
            module: "@seseragi/runtime/array",
            export_name: "flatMap",
            source_arity: 2,
        },
    ),
    (
        "std/array::find",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.array.find",
            local_name: "_ssrg_array_find",
            module: "@seseragi/runtime/array",
            export_name: "find",
            source_arity: 2,
        },
    ),
    (
        "std/array::take",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.array.take",
            local_name: "_ssrg_array_take",
            module: "@seseragi/runtime/array",
            export_name: "take",
            source_arity: 2,
        },
    ),
    (
        "std/array::drop",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.array.drop",
            local_name: "_ssrg_array_drop",
            module: "@seseragi/runtime/array",
            export_name: "drop",
            source_arity: 2,
        },
    ),
    (
        "std/array::append",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.array.append",
            local_name: "_ssrg_array_append",
            module: "@seseragi/runtime/array",
            export_name: "append",
            source_arity: 2,
        },
    ),
    (
        "std/array::concat",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.array.concat",
            local_name: "_ssrg_array_concat",
            module: "@seseragi/runtime/array",
            export_name: "concat",
            source_arity: 1,
        },
    ),
    (
        "std/array::reverse",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.array.reverse",
            local_name: "_ssrg_array_reverse",
            module: "@seseragi/runtime/array",
            export_name: "reverse",
            source_arity: 1,
        },
    ),
    (
        "std/array::length",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.array.length",
            local_name: "_ssrg_array_length",
            module: "@seseragi/runtime/array",
            export_name: "length",
            source_arity: 1,
        },
    ),
    (
        "std/array::isEmpty",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.array.is-empty",
            local_name: "_ssrg_array_isEmpty",
            module: "@seseragi/runtime/array",
            export_name: "isEmpty",
            source_arity: 1,
        },
    ),
    (
        "std/array::get",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.array.get",
            local_name: "_ssrg_array_get",
            module: "@seseragi/runtime/array",
            export_name: "get",
            source_arity: 2,
        },
    ),
    (
        "std/array::head",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.array.head",
            local_name: "_ssrg_array_head",
            module: "@seseragi/runtime/array",
            export_name: "head",
            source_arity: 1,
        },
    ),
    (
        "std/array::tail",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.array.tail",
            local_name: "_ssrg_array_tail",
            module: "@seseragi/runtime/array",
            export_name: "tail",
            source_arity: 1,
        },
    ),
    (
        "std/list::toArray",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.list.to-array",
            local_name: "_ssrg_list_toArray",
            module: "@seseragi/runtime/list",
            export_name: "toArray",
            source_arity: 1,
        },
    ),
    (
        "std/list::filter",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.list.filter",
            local_name: "_ssrg_list_filter",
            module: "@seseragi/runtime/list",
            export_name: "filter",
            source_arity: 2,
        },
    ),
    (
        "std/list::filterMap",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.list.filter-map",
            local_name: "_ssrg_list_filterMap",
            module: "@seseragi/runtime/list",
            export_name: "filterMap",
            source_arity: 2,
        },
    ),
    (
        "std/list::flatMap",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.list.flat-map",
            local_name: "_ssrg_list_flatMap",
            module: "@seseragi/runtime/list",
            export_name: "flatMap",
            source_arity: 2,
        },
    ),
    (
        "std/list::find",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.list.find",
            local_name: "_ssrg_list_find",
            module: "@seseragi/runtime/list",
            export_name: "find",
            source_arity: 2,
        },
    ),
    (
        "std/list::take",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.list.take",
            local_name: "_ssrg_list_take",
            module: "@seseragi/runtime/list",
            export_name: "take",
            source_arity: 2,
        },
    ),
    (
        "std/list::drop",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.list.drop",
            local_name: "_ssrg_list_drop",
            module: "@seseragi/runtime/list",
            export_name: "drop",
            source_arity: 2,
        },
    ),
    (
        "std/list::append",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.list.append",
            local_name: "_ssrg_list_append",
            module: "@seseragi/runtime/list",
            export_name: "append",
            source_arity: 2,
        },
    ),
    (
        "std/list::concat",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.list.concat",
            local_name: "_ssrg_list_concat",
            module: "@seseragi/runtime/list",
            export_name: "concat",
            source_arity: 1,
        },
    ),
    (
        "std/list::reverse",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.list.reverse",
            local_name: "_ssrg_list_reverse",
            module: "@seseragi/runtime/list",
            export_name: "reverse",
            source_arity: 1,
        },
    ),
    (
        "std/list::length",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.list.length",
            local_name: "_ssrg_list_length",
            module: "@seseragi/runtime/list",
            export_name: "length",
            source_arity: 1,
        },
    ),
    (
        "std/list::isEmpty",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.list.is-empty",
            local_name: "_ssrg_list_isEmpty",
            module: "@seseragi/runtime/list",
            export_name: "isEmpty",
            source_arity: 1,
        },
    ),
    (
        "std/list::get",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.list.get",
            local_name: "_ssrg_list_get",
            module: "@seseragi/runtime/list",
            export_name: "get",
            source_arity: 2,
        },
    ),
    (
        "std/list::head",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.list.head",
            local_name: "_ssrg_list_head",
            module: "@seseragi/runtime/list",
            export_name: "head",
            source_arity: 1,
        },
    ),
    (
        "std/list::tail",
        RuntimeCollectionOperation {
            result_erased: false,
            runtime_feature: "core.list.tail",
            local_name: "_ssrg_list_tail",
            module: "@seseragi/runtime/list",
            export_name: "tail",
            source_arity: 1,
        },
    ),
];

pub(crate) fn runtime_standard_collection_operation(
    callee: &str,
) -> Option<&'static RuntimeCollectionOperation> {
    STANDARD_COLLECTION_OPERATIONS
        .iter()
        .find(|(canonical, _)| *canonical == callee)
        .map(|(_, operation)| operation)
}

pub(crate) fn runtime_collection_operation(
    callee: &str,
    evidence: &[CoreCallEvidence],
) -> Option<&'static RuntimeCollectionOperation> {
    let [selected] = evidence else {
        return None;
    };
    let CoreInstanceEvidence::Standard { identity, .. } = &selected.evidence else {
        return None;
    };
    if callee != "std/prelude::reduce" || selected.constraint.name != "Reducible" {
        return None;
    }
    match identity.as_str() {
        "std/array::Reducible" => Some(&ARRAY_REDUCE),
        "std/range::Reducible" => Some(&RANGE_REDUCE),
        "std/list::Reducible" => Some(&LIST_REDUCE),
        _ => None,
    }
}

pub(crate) fn runtime_collection_join_operation(
    callee: &str,
    evidence: &[CoreCallEvidence],
) -> Option<&'static RuntimeCollectionOperation> {
    matches!(
        evidence,
        [selected] if selected.constraint.name == "Reducible"
    )
    .then_some(())
    .filter(|_| callee == "std/prelude::join")
    .map(|_| &COLLECTION_JOIN)
}

pub(crate) fn runtime_collection_sum_operation(
    callee: &str,
    evidence: &[CoreCallEvidence],
) -> Option<&'static RuntimeCollectionOperation> {
    matches!(
        evidence,
        [reducible, zero, add]
            if reducible.constraint.name == "Reducible"
                && zero.constraint.name == "Zero"
                && add.constraint.name == "Add"
    )
    .then_some(())
    .filter(|_| callee == "std/prelude::sum")
    .map(|_| &COLLECTION_SUM)
}

pub(crate) fn runtime_collection_product_operation(
    callee: &str,
    evidence: &[CoreCallEvidence],
) -> Option<&'static RuntimeCollectionOperation> {
    matches!(
        evidence,
        [reducible, one, mul]
            if reducible.constraint.name == "Reducible"
                && one.constraint.name == "One"
                && mul.constraint.name == "Mul"
    )
    .then_some(())
    .filter(|_| callee == "std/prelude::product")
    .map(|_| &COLLECTION_PRODUCT)
}

pub(crate) fn runtime_collection_combine_operation(
    callee: &str,
    evidence: &[CoreCallEvidence],
) -> Option<&'static RuntimeCollectionOperation> {
    matches!(
        evidence,
        [reducible, monoid]
            if reducible.constraint.name == "Reducible"
                && monoid.constraint.name == "Monoid"
    )
    .then_some(())
    .filter(|_| callee == "std/prelude::combine")
    .map(|_| &COLLECTION_COMBINE)
}

pub(crate) fn runtime_collection_predicate_operation(
    callee: &str,
    evidence: &[CoreCallEvidence],
) -> Option<&'static RuntimeCollectionOperation> {
    matches!(
        evidence,
        [iterable] if iterable.constraint.name == "Iterable"
    )
    .then_some(())?;
    match callee {
        "std/prelude::any" => Some(&COLLECTION_ANY),
        "std/prelude::all" => Some(&COLLECTION_ALL),
        _ => None,
    }
}

pub(crate) fn runtime_collection_for_each_operation(
    callee: &str,
    evidence: &[CoreCallEvidence],
) -> Option<&'static RuntimeCollectionOperation> {
    matches!(
        evidence,
        [selected] if selected.constraint.name == "Iterable"
    )
    .then_some(())
    .filter(|_| callee == "std/prelude::forEach")
    .map(|_| &COLLECTION_FOR_EACH)
}

pub(crate) fn runtime_collection_operation_for_feature(
    feature: &str,
) -> Option<RuntimeCollectionOperation> {
    [
        ARRAY_REDUCE,
        RANGE_REDUCE,
        LIST_REDUCE,
        COLLECTION_JOIN,
        COLLECTION_SUM,
        COLLECTION_PRODUCT,
        COLLECTION_COMBINE,
        COLLECTION_ANY,
        COLLECTION_ALL,
        COLLECTION_FOR_EACH,
        ARRAY_COMPREHEND,
        ARRAY_COMPREHEND_FLAT,
        RANGE_COMPREHEND,
        RANGE_COMPREHEND_FLAT,
        LIST_COMPREHEND,
        LIST_COMPREHEND_FLAT,
    ]
    .into_iter()
    .chain(
        STANDARD_COLLECTION_OPERATIONS
            .iter()
            .map(|(_, operation)| *operation),
    )
    .find(|operation| operation.runtime_feature == feature)
}

pub(crate) fn runtime_iterable_operation(
    evidence: &CoreCallEvidence,
    flatten: bool,
) -> Option<&'static RuntimeCollectionOperation> {
    if evidence.constraint.name != "Iterable" {
        return None;
    }
    let CoreInstanceEvidence::Standard { identity, .. } = &evidence.evidence else {
        return None;
    };
    match (identity.as_str(), flatten) {
        ("std/array::Iterable", false) => Some(&ARRAY_COMPREHEND),
        ("std/array::Iterable", true) => Some(&ARRAY_COMPREHEND_FLAT),
        ("std/range::Iterable", false) => Some(&RANGE_COMPREHEND),
        ("std/range::Iterable", true) => Some(&RANGE_COMPREHEND_FLAT),
        ("std/list::Iterable", false) => Some(&LIST_COMPREHEND),
        ("std/list::Iterable", true) => Some(&LIST_COMPREHEND_FLAT),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CoreInstanceConstraint, CoreType};

    fn parameter_evidence(name: &str, index: usize) -> CoreCallEvidence {
        CoreCallEvidence {
            constraint: CoreInstanceConstraint {
                trait_identity: None,
                name: name.to_owned(),
                arguments: Vec::new(),
            },
            evidence: CoreInstanceEvidence::Parameter { index },
        }
    }

    #[test]
    fn resolves_array_reduce_only_with_selected_standard_evidence() {
        let evidence = [CoreCallEvidence {
            constraint: CoreInstanceConstraint {
                trait_identity: None,
                name: "Reducible".to_owned(),
                arguments: vec![CoreType::Named {
                    name: "Array".to_owned(),
                    arguments: vec![CoreType::Named {
                        name: "Int".to_owned(),
                        arguments: Vec::new(),
                    }],
                }],
            },
            evidence: CoreInstanceEvidence::Standard {
                identity: "std/array::Reducible".to_owned(),
                type_arguments: Vec::new(),
                evidence_arguments: Vec::new(),
            },
        }];

        assert_eq!(
            runtime_collection_operation("std/prelude::reduce", &evidence)
                .map(|operation| operation.runtime_feature),
            Some("core.array.reduce")
        );
        assert!(runtime_collection_operation("user::reduce", &evidence).is_none());
    }

    #[test]
    fn resolves_range_reduce_with_selected_standard_evidence() {
        let evidence = [CoreCallEvidence {
            constraint: CoreInstanceConstraint {
                trait_identity: None,
                name: "Reducible".to_owned(),
                arguments: vec![CoreType::Named {
                    name: "Range".to_owned(),
                    arguments: vec![CoreType::Named {
                        name: "Int".to_owned(),
                        arguments: Vec::new(),
                    }],
                }],
            },
            evidence: CoreInstanceEvidence::Standard {
                identity: "std/range::Reducible".to_owned(),
                type_arguments: Vec::new(),
                evidence_arguments: Vec::new(),
            },
        }];

        assert_eq!(
            runtime_collection_operation("std/prelude::reduce", &evidence)
                .map(|operation| operation.runtime_feature),
            Some("core.range.reduce")
        );
    }

    #[test]
    fn resolves_generic_join_from_reducible_evidence() {
        let evidence = [CoreCallEvidence {
            constraint: CoreInstanceConstraint {
                trait_identity: None,
                name: "Reducible".to_owned(),
                arguments: vec![
                    CoreType::Named {
                        name: "C".to_owned(),
                        arguments: Vec::new(),
                    },
                    CoreType::Named {
                        name: "String".to_owned(),
                        arguments: Vec::new(),
                    },
                ],
            },
            evidence: CoreInstanceEvidence::Parameter { index: 0 },
        }];

        assert_eq!(
            runtime_collection_join_operation("std/prelude::join", &evidence)
                .map(|operation| operation.runtime_feature),
            Some("core.collection.join")
        );
        assert!(runtime_collection_join_operation("user::join", &evidence).is_none());
    }

    #[test]
    fn resolves_generic_sum_from_reducible_zero_and_add_evidence() {
        let evidence = [
            CoreCallEvidence {
                constraint: CoreInstanceConstraint {
                    trait_identity: None,
                    name: "Reducible".to_owned(),
                    arguments: vec![],
                },
                evidence: CoreInstanceEvidence::Parameter { index: 0 },
            },
            CoreCallEvidence {
                constraint: CoreInstanceConstraint {
                    trait_identity: None,
                    name: "Zero".to_owned(),
                    arguments: vec![],
                },
                evidence: CoreInstanceEvidence::Parameter { index: 1 },
            },
            CoreCallEvidence {
                constraint: CoreInstanceConstraint {
                    trait_identity: None,
                    name: "Add".to_owned(),
                    arguments: vec![],
                },
                evidence: CoreInstanceEvidence::Parameter { index: 2 },
            },
        ];

        assert_eq!(
            runtime_collection_sum_operation("std/prelude::sum", &evidence)
                .map(|operation| operation.runtime_feature),
            Some("core.collection.sum")
        );
        assert!(runtime_collection_sum_operation("user::sum", &evidence).is_none());
    }

    #[test]
    fn resolves_generic_product_from_reducible_one_and_mul_evidence() {
        let evidence = [
            parameter_evidence("Reducible", 0),
            parameter_evidence("One", 1),
            parameter_evidence("Mul", 2),
        ];

        assert_eq!(
            runtime_collection_product_operation("std/prelude::product", &evidence)
                .map(|operation| operation.runtime_feature),
            Some("core.collection.product")
        );
        assert!(runtime_collection_product_operation("user::product", &evidence).is_none());
    }

    #[test]
    fn resolves_short_circuit_predicate_aggregates_from_iterable_evidence() {
        let evidence = [parameter_evidence("Iterable", 0)];

        assert_eq!(
            runtime_collection_predicate_operation("std/prelude::any", &evidence)
                .map(|operation| operation.runtime_feature),
            Some("core.collection.any")
        );
        assert_eq!(
            runtime_collection_predicate_operation("std/prelude::all", &evidence)
                .map(|operation| operation.runtime_feature),
            Some("core.collection.all")
        );
    }

    #[test]
    fn resolves_standard_array_and_list_operations_from_one_registry() {
        assert_eq!(
            runtime_standard_collection_operation("std/array::filterMap")
                .map(|operation| operation.runtime_feature),
            Some("core.array.filter-map")
        );
        assert_eq!(
            runtime_standard_collection_operation("std/list::tail")
                .map(|operation| operation.runtime_feature),
            Some("core.list.tail")
        );
        assert!(runtime_standard_collection_operation("user::head").is_none());
    }

    #[test]
    fn resolves_generic_combine_from_reducible_and_monoid_evidence() {
        let evidence = [
            CoreCallEvidence {
                constraint: CoreInstanceConstraint {
                    trait_identity: None,
                    name: "Reducible".to_owned(),
                    arguments: vec![],
                },
                evidence: CoreInstanceEvidence::Parameter { index: 0 },
            },
            CoreCallEvidence {
                constraint: CoreInstanceConstraint {
                    trait_identity: None,
                    name: "Monoid".to_owned(),
                    arguments: vec![],
                },
                evidence: CoreInstanceEvidence::Standard {
                    identity: "std/string::Monoid".to_owned(),
                    type_arguments: Vec::new(),
                    evidence_arguments: Vec::new(),
                },
            },
        ];

        assert_eq!(
            runtime_collection_combine_operation("std/prelude::combine", &evidence)
                .map(|operation| operation.runtime_feature),
            Some("core.collection.combine")
        );
        assert!(runtime_collection_combine_operation("user::combine", &evidence).is_none());
    }

    #[test]
    fn resolves_generic_for_each_from_iterable_evidence() {
        let evidence = [CoreCallEvidence {
            constraint: CoreInstanceConstraint {
                trait_identity: None,
                name: "Iterable".to_owned(),
                arguments: vec![
                    CoreType::Named {
                        name: "C".to_owned(),
                        arguments: Vec::new(),
                    },
                    CoreType::Named {
                        name: "A".to_owned(),
                        arguments: Vec::new(),
                    },
                ],
            },
            evidence: CoreInstanceEvidence::Parameter { index: 0 },
        }];

        assert_eq!(
            runtime_collection_for_each_operation("std/prelude::forEach", &evidence)
                .map(|operation| operation.runtime_feature),
            Some("effect.collection.for-each")
        );
    }

    #[test]
    fn resolves_list_reduce_with_selected_standard_evidence() {
        let evidence = [CoreCallEvidence {
            constraint: CoreInstanceConstraint {
                trait_identity: None,
                name: "Reducible".to_owned(),
                arguments: vec![
                    CoreType::Named {
                        name: "List".to_owned(),
                        arguments: vec![CoreType::Named {
                            name: "Int".to_owned(),
                            arguments: Vec::new(),
                        }],
                    },
                    CoreType::Named {
                        name: "Int".to_owned(),
                        arguments: Vec::new(),
                    },
                ],
            },
            evidence: CoreInstanceEvidence::Standard {
                identity: "std/list::Reducible".to_owned(),
                type_arguments: Vec::new(),
                evidence_arguments: Vec::new(),
            },
        }];

        assert_eq!(
            runtime_collection_operation("std/prelude::reduce", &evidence)
                .map(|operation| operation.runtime_feature),
            Some("core.list.reduce")
        );
    }

    #[test]
    fn resolves_iterable_runtime_by_evidence_and_nesting() {
        let range = CoreCallEvidence {
            constraint: CoreInstanceConstraint {
                trait_identity: None,
                name: "Iterable".to_owned(),
                arguments: vec![
                    CoreType::Named {
                        name: "Range".to_owned(),
                        arguments: vec![CoreType::Named {
                            name: "Int".to_owned(),
                            arguments: Vec::new(),
                        }],
                    },
                    CoreType::Named {
                        name: "Int".to_owned(),
                        arguments: Vec::new(),
                    },
                ],
            },
            evidence: CoreInstanceEvidence::Standard {
                identity: "std/range::Iterable".to_owned(),
                type_arguments: Vec::new(),
                evidence_arguments: Vec::new(),
            },
        };

        assert_eq!(
            runtime_iterable_operation(&range, false).map(|operation| operation.runtime_feature),
            Some("core.range.comprehend")
        );
        assert_eq!(
            runtime_iterable_operation(&range, true).map(|operation| operation.runtime_feature),
            Some("core.range.comprehend.flat")
        );
    }
}
