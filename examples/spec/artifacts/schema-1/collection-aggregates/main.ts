import { intOne as _ssrg_int_one, intMul as _ssrg_int_mul_dictionary } from "@seseragi/runtime/int"
import { product as _ssrg_collection_product, any as _ssrg_collection_any, all as _ssrg_collection_all } from "@seseragi/runtime/collection"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { arrayReducible as _ssrg_array_reducible, arrayIterable as _ssrg_array_iterable } from "@seseragi/runtime/array"
import { listReducible as _ssrg_list_reducible, fromArray as _ssrg_list_from_array, listIterable as _ssrg_list_iterable } from "@seseragi/runtime/list"
import { rangeReducible as _ssrg_range_reducible, inclusive as _ssrg_range_inclusive, rangeIterable as _ssrg_range_iterable } from "@seseragi/runtime/range"

const productValues = <C,>(values: C) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => _ssrg_collection_product(__ssrg$evidence$0, _ssrg_int_one, _ssrg_int_mul_dictionary, values)
const anyPositive = <C,>(values: C) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => _ssrg_collection_any(__ssrg$evidence$0, (value: number) => value > 0, values)
const allPositive = <C,>(values: C) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => _ssrg_collection_all(__ssrg$evidence$0, (value: number) => value > 0, values)
const emptyValues = (unit: undefined) => [] as ReadonlyArray<number>
const yesNo = (value: boolean) => value ? "true" : "false"
const number = (value: number) => _ssrg_show_intShow["show"](value)
const productArray = (unit: undefined) => number(((productValues([2, 3, 4])(_ssrg_array_reducible)) as number))
const productList = (unit: undefined) => number(((productValues(_ssrg_list_from_array([2, 3, 4]))(_ssrg_list_reducible)) as number))
const productRange = (unit: undefined) => number(((productValues(_ssrg_range_inclusive(1, 4))(_ssrg_range_reducible)) as number))
const productEmpty = (unit: undefined) => number(((productValues(emptyValues(undefined))(_ssrg_array_reducible)) as number))
const anyArray = (unit: undefined) => yesNo(((anyPositive([0, 0, 2])(_ssrg_array_iterable)) as boolean))
const anyList = (unit: undefined) => yesNo(((anyPositive(_ssrg_list_from_array([0, 0, 0]))(_ssrg_list_iterable)) as boolean))
const anyRange = (unit: undefined) => yesNo(((anyPositive(_ssrg_range_inclusive(0, 2))(_ssrg_range_iterable)) as boolean))
const anyEmpty = (unit: undefined) => yesNo(((anyPositive(emptyValues(undefined))(_ssrg_array_iterable)) as boolean))
const allArray = (unit: undefined) => yesNo(((allPositive([1, 2, 3])(_ssrg_array_iterable)) as boolean))
const allList = (unit: undefined) => yesNo(((allPositive(_ssrg_list_from_array([1, 2, 3]))(_ssrg_list_iterable)) as boolean))
const allRange = (unit: undefined) => yesNo(((allPositive(_ssrg_range_inclusive(1, 3))(_ssrg_range_iterable)) as boolean))
const allEmpty = (unit: undefined) => yesNo(((allPositive(emptyValues(undefined))(_ssrg_array_iterable)) as boolean))
export const collectionAggregateResults = (unit: undefined) => [productArray(undefined), productList(undefined), productRange(undefined), productEmpty(undefined), anyArray(undefined), anyList(undefined), anyRange(undefined), anyEmpty(undefined), allArray(undefined), allList(undefined), allRange(undefined), allEmpty(undefined)]
