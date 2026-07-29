import { arrayReducible as _ssrg_array_reducible, collectMap as _ssrg_array_comprehend, toList as _ssrg_array_toList } from "@seseragi/runtime/array"
import { join as _ssrg_collection_join } from "@seseragi/runtime/collection"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { collectMap as _ssrg_list_comprehend, toArray as _ssrg_list_toArray, fromArray as _ssrg_list_from_array, type List as List } from "@seseragi/runtime/list"

const showArray = (values: ReadonlyArray<number>) => _ssrg_collection_join(_ssrg_array_reducible, ",", _ssrg_array_comprehend(values, (value) => true, (value) => _ssrg_show_intShow["show"](value)))
const showList = (values: List<number>) => _ssrg_collection_join(_ssrg_array_reducible, ",", _ssrg_list_comprehend(values, (value) => true, (value) => _ssrg_show_intShow["show"](value)))
export const collectionConvertResults = (unit: undefined) => [showList(_ssrg_array_toList([1, 2, 3])), showArray(_ssrg_list_toArray(_ssrg_list_from_array([4, 5, 6]))), showArray(_ssrg_list_toArray(_ssrg_array_toList([1, 2, 3]))), showList(_ssrg_array_toList(_ssrg_list_toArray(_ssrg_list_from_array([4, 5, 6])))), showList(_ssrg_array_toList([] as ReadonlyArray<number>)), showArray(_ssrg_list_toArray(_ssrg_list_from_array([] as ReadonlyArray<number>)))]
