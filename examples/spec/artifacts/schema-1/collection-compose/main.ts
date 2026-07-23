import { arrayReducible as _ssrg_array_reducible, collectMap as _ssrg_array_comprehend, append as _ssrg_array_append, concat as _ssrg_array_concat, reverse as _ssrg_array_reverse } from "@seseragi/runtime/array"
import { join as _ssrg_collection_join } from "@seseragi/runtime/collection"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { collectMap as _ssrg_list_comprehend, fromArray as _ssrg_list_from_array, append as _ssrg_list_append, concat as _ssrg_list_concat, reverse as _ssrg_list_reverse, type List as List } from "@seseragi/runtime/list"

const showArray = (values: ReadonlyArray<bigint>) => _ssrg_collection_join(_ssrg_array_reducible, ",", _ssrg_array_comprehend(values, (value) => true, (value) => _ssrg_show_intShow["show"](value)))
const showList = (values: List<bigint>) => _ssrg_collection_join(_ssrg_array_reducible, ",", _ssrg_list_comprehend(values, (value) => true, (value) => _ssrg_show_intShow["show"](value)))
const nestedLists = (unit: undefined) => _ssrg_list_from_array([_ssrg_list_from_array([1n, 2n]), _ssrg_list_from_array([] as ReadonlyArray<bigint>), _ssrg_list_from_array([3n, 4n])])
export const collectionComposeResults = (unit: undefined) => [showArray(_ssrg_array_append([3n, 4n], [1n, 2n])), showList(_ssrg_list_append(_ssrg_list_from_array([3n, 4n]), _ssrg_list_from_array([1n, 2n]))), showArray(_ssrg_array_concat([[1n, 2n], [] as ReadonlyArray<bigint>, [3n, 4n]])), showList(_ssrg_list_concat(nestedLists(undefined))), showArray(_ssrg_array_reverse([1n, 2n, 3n])), showList(_ssrg_list_reverse(_ssrg_list_from_array([1n, 2n, 3n]))), showArray(_ssrg_array_concat([] as ReadonlyArray<ReadonlyArray<bigint>>)), showList(_ssrg_list_concat(_ssrg_list_from_array([] as ReadonlyArray<List<bigint>>)))]
