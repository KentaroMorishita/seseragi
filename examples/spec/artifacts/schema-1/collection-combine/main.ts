import { combine as _ssrg_collection_combine, join as _ssrg_collection_join } from "@seseragi/runtime/collection"
import { arrayReducible as _ssrg_array_reducible, arrayMonoid as _ssrg_array_monoid, arraySemigroup as _ssrg_array_semigroup } from "@seseragi/runtime/array"
import { stringMonoid as _ssrg_string_monoid, stringSemigroup as _ssrg_string_semigroup } from "@seseragi/runtime/string"
import { listMonoid as _ssrg_list_monoid, fromArray as _ssrg_list_from_array, listReducible as _ssrg_list_reducible, listSemigroup as _ssrg_list_semigroup, type List as List } from "@seseragi/runtime/list"

const combineValues = <C, A,>(values: C) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => (__ssrg$evidence$1: Readonly<Record<string, (...args: any[]) => any>>) => _ssrg_collection_combine(__ssrg$evidence$0, __ssrg$evidence$1, values)
const combinedString = (unit: undefined) => combineValues(["Sese", "ragi"])(_ssrg_array_reducible)(_ssrg_string_monoid)
const combinedArrayValues = (unit: undefined) => combineValues([["a", "b"], ["c"]])(_ssrg_array_reducible)(_ssrg_array_monoid)
const combinedArray = (unit: undefined) => _ssrg_collection_join(_ssrg_array_reducible, "|", combinedArrayValues(undefined))
const combinedListValues = (unit: undefined) => combineValues([_ssrg_list_from_array(["l1", "l2"]), _ssrg_list_from_array(["l3"])])(_ssrg_array_reducible)(_ssrg_list_monoid)
const combinedList = (unit: undefined) => _ssrg_collection_join(_ssrg_list_reducible, "|", combinedListValues(undefined))
const emptyStrings = (unit: undefined) => [] as ReadonlyArray<string>
const combinedEmpty = (unit: undefined) => combineValues(emptyStrings(undefined))(_ssrg_array_reducible)(_ssrg_string_monoid)
const appendWithMonoid = <A,>(left: A) => (right: A) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => __ssrg$evidence$0["append"](left)(right)
const combinedViaSupertrait = (unit: undefined) => appendWithMonoid("super")("trait")(_ssrg_string_monoid)
const appendedString = (unit: undefined) => _ssrg_string_semigroup["append"]("semi")("group")
const appendedArrayValues = (unit: undefined) => _ssrg_array_semigroup["append"](["array"])(["append"])
const appendedArray = (unit: undefined) => _ssrg_collection_join(_ssrg_array_reducible, "|", appendedArrayValues(undefined))
const appendedListValues = (unit: undefined) => _ssrg_list_semigroup["append"](_ssrg_list_from_array(["list"]))(_ssrg_list_from_array(["append"]))
const appendedList = (unit: undefined) => _ssrg_collection_join(_ssrg_list_reducible, "|", appendedListValues(undefined))
export const collectionCombineResults = (unit: undefined) => [combinedString(undefined), combinedArray(undefined), combinedList(undefined), combinedEmpty(undefined), combinedViaSupertrait(undefined), appendedString(undefined), appendedArray(undefined), appendedList(undefined)]
