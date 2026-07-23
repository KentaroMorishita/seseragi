import { remainder as _ssrg_int64_remainder, subtract as _ssrg_int64_subtract } from "@seseragi/runtime/int64"
import { Just as _ssrg_maybe_Just, Nothing as _ssrg_maybe_Nothing } from "@seseragi/runtime/sum"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { fromArray as _ssrg_list_from_array, collectMap as _ssrg_list_comprehend, filter as _ssrg_list_filter, listReducible as _ssrg_list_reducible, filterMap as _ssrg_list_filterMap, flatMap as _ssrg_list_flatMap, type List as List } from "@seseragi/runtime/list"
import { arrayReducible as _ssrg_array_reducible, collectMap as _ssrg_array_comprehend, filter as _ssrg_array_filter, filterMap as _ssrg_array_filterMap, flatMap as _ssrg_array_flatMap } from "@seseragi/runtime/array"
import { join as _ssrg_collection_join } from "@seseragi/runtime/collection"

const even = (value: bigint) => _ssrg_int64_remainder(value, 2n) === 0n
const labelEven = (value: bigint) => even(value) ? _ssrg_maybe_Just("#" + _ssrg_show_intShow["show"](value)) : _ssrg_maybe_Nothing
const signedArray = (value: bigint) => [value, _ssrg_int64_subtract(0n, value)]
const signedList = (value: bigint) => _ssrg_list_from_array([value, _ssrg_int64_subtract(0n, value)])
const showArray = (values: ReadonlyArray<bigint>) => _ssrg_collection_join(_ssrg_array_reducible, ",", _ssrg_array_comprehend(values, (value) => true, (value) => _ssrg_show_intShow["show"](value)))
const showList = (values: List<bigint>) => _ssrg_collection_join(_ssrg_array_reducible, ",", _ssrg_list_comprehend(values, (value) => true, (value) => _ssrg_show_intShow["show"](value)))
export const collectionTransformResults = (unit: undefined) => [showArray(_ssrg_array_filter(even, [1n, 2n, 3n, 4n])), showList(_ssrg_list_filter(even, _ssrg_list_from_array([1n, 2n, 3n, 4n]))), _ssrg_collection_join(_ssrg_array_reducible, ",", _ssrg_array_filterMap(labelEven, [1n, 2n, 3n, 4n])), _ssrg_collection_join(_ssrg_list_reducible, ",", _ssrg_list_filterMap(labelEven, _ssrg_list_from_array([1n, 2n, 3n, 4n]))), showArray(_ssrg_array_flatMap(signedArray, [1n, 2n])), showList(_ssrg_list_flatMap(signedList, _ssrg_list_from_array([1n, 2n])))]
