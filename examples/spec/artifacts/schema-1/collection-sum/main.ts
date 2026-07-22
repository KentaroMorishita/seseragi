import { intZero as _ssrg_int_zero, intAdd as _ssrg_int_add } from "@seseragi/runtime/int64"
import { sum as _ssrg_collection_sum } from "@seseragi/runtime/collection"
import { arrayReducible as _ssrg_array_reducible } from "@seseragi/runtime/array"
import { listReducible as _ssrg_list_reducible, fromArray as _ssrg_list_from_array } from "@seseragi/runtime/list"
import { rangeReducible as _ssrg_range_reducible, inclusive as _ssrg_range_inclusive } from "@seseragi/runtime/range"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"

const sumValues = <C,>(values: C) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => _ssrg_collection_sum(__ssrg$evidence$0, _ssrg_int_zero, _ssrg_int_add, values)
const emptyValues = (unit: undefined) => [] as ReadonlyArray<bigint>
const arrayTotal = (unit: undefined) => sumValues([20n, 22n])(_ssrg_array_reducible)
const listTotal = (unit: undefined) => sumValues(_ssrg_list_from_array([10n, 20n, 12n]))(_ssrg_list_reducible)
const rangeTotal = (unit: undefined) => sumValues(_ssrg_range_inclusive(1n, 4n))(_ssrg_range_reducible)
const emptyTotal = (unit: undefined) => sumValues(emptyValues(undefined))(_ssrg_array_reducible)
export const collectionSumResults = (unit: undefined) => [_ssrg_show_intShow["show"](arrayTotal(undefined)), _ssrg_show_intShow["show"](listTotal(undefined)), _ssrg_show_intShow["show"](rangeTotal(undefined)), _ssrg_show_intShow["show"](emptyTotal(undefined))]
