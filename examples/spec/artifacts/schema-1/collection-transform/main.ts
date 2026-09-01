import { intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
import { remainder as _ssrg_int_remainder, subtract as _ssrg_int_subtract } from "@seseragi/runtime/int"
import { Just as _ssrg_maybe_Just, Nothing as _ssrg_maybe_Nothing } from "@seseragi/runtime/sum"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { fromArray as _ssrg_list_from_array, collectMap as _ssrg_list_comprehend, filter as _ssrg_list_filter, listReducible as _ssrg_list_reducible, filterMap as _ssrg_list_filterMap, flatMap as _ssrg_list_flatMap, type List as List } from "@seseragi/runtime/list"
import { arrayReducible as _ssrg_array_reducible, collectMap as _ssrg_array_comprehend, filter as _ssrg_array_filter, filterMap as _ssrg_array_filterMap, flatMap as _ssrg_array_flatMap } from "@seseragi/runtime/array"
import { join as _ssrg_collection_join } from "@seseragi/runtime/collection"

const even = (value: number) => _ssrg_int_eq_dictionary["eq"](_ssrg_int_remainder(value, 2))(0)
const labelEven = (value: number) => even(value) ? _ssrg_maybe_Just("#" + _ssrg_show_intShow["show"](value)) : _ssrg_maybe_Nothing
const signedArray = (value: number) => [value, _ssrg_int_subtract(0, value)]
const signedList = (value: number) => _ssrg_list_from_array([value, _ssrg_int_subtract(0, value)])
const showArray = (values: ReadonlyArray<number>) => _ssrg_collection_join(_ssrg_array_reducible, ",", _ssrg_array_comprehend(values, (value) => true, (value) => _ssrg_show_intShow["show"](value)))
const showList = (values: List<number>) => _ssrg_collection_join(_ssrg_array_reducible, ",", _ssrg_list_comprehend(values, (value) => true, (value) => _ssrg_show_intShow["show"](value)))
export const collectionTransformResults = (unit: undefined) => [showArray(_ssrg_array_filter(even, [1, 2, 3, 4])), showList(_ssrg_list_filter(even, _ssrg_list_from_array([1, 2, 3, 4]))), _ssrg_collection_join(_ssrg_array_reducible, ",", _ssrg_array_filterMap(labelEven, [1, 2, 3, 4])), _ssrg_collection_join(_ssrg_list_reducible, ",", _ssrg_list_filterMap(labelEven, _ssrg_list_from_array([1, 2, 3, 4]))), showArray(_ssrg_array_flatMap(signedArray, [1, 2])), showList(_ssrg_list_flatMap(signedList, _ssrg_list_from_array([1, 2])))]
