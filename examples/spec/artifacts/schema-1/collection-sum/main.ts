import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { intZero as _ssrg_int_zero, intAdd as _ssrg_int_add_dictionary } from "@seseragi/runtime/int"
import { sum as _ssrg_collection_sum } from "@seseragi/runtime/collection"
import { arrayReducible as _ssrg_array_reducible } from "@seseragi/runtime/array"
import { listReducible as _ssrg_list_reducible, fromArray as _ssrg_list_from_array } from "@seseragi/runtime/list"
import { rangeReducible as _ssrg_range_reducible, inclusive as _ssrg_range_inclusive } from "@seseragi/runtime/range"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
$ssrg$assertUnicodeVersion("17.0.0")

const sumValues = <C,>(values: C) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => _ssrg_collection_sum(__ssrg$evidence$0, _ssrg_int_zero, _ssrg_int_add_dictionary, values)
const emptyValues = (unit: undefined) => [] as ReadonlyArray<number>
const arrayTotal = (unit: undefined) => ((sumValues([20, 22])(_ssrg_array_reducible)) as number)
const listTotal = (unit: undefined) => ((sumValues(_ssrg_list_from_array([10, 20, 12]))(_ssrg_list_reducible)) as number)
const rangeTotal = (unit: undefined) => ((sumValues(_ssrg_range_inclusive(1, 4))(_ssrg_range_reducible)) as number)
const emptyTotal = (unit: undefined) => ((sumValues(emptyValues(undefined))(_ssrg_array_reducible)) as number)
export const collectionSumResults = (unit: undefined) => [_ssrg_show_intShow["show"](arrayTotal(undefined)), _ssrg_show_intShow["show"](listTotal(undefined)), _ssrg_show_intShow["show"](rangeTotal(undefined)), _ssrg_show_intShow["show"](emptyTotal(undefined))]
