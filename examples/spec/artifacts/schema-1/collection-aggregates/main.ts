import { intOne as _ssrg_int_one, intMul as _ssrg_int_mul } from "@seseragi/runtime/int64"
import { product as _ssrg_collection_product, any as _ssrg_collection_any, all as _ssrg_collection_all } from "@seseragi/runtime/collection"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { arrayReducible as _ssrg_array_reducible, arrayIterable as _ssrg_array_iterable } from "@seseragi/runtime/array"
import { listReducible as _ssrg_list_reducible, fromArray as _ssrg_list_from_array, listIterable as _ssrg_list_iterable } from "@seseragi/runtime/list"
import { rangeReducible as _ssrg_range_reducible, inclusive as _ssrg_range_inclusive, rangeIterable as _ssrg_range_iterable } from "@seseragi/runtime/range"

const productValues = <C,>(values: C) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => _ssrg_collection_product(__ssrg$evidence$0, _ssrg_int_one, _ssrg_int_mul, values)
const anyPositive = <C,>(values: C) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => _ssrg_collection_any(__ssrg$evidence$0, (value: bigint) => value > 0n, values)
const allPositive = <C,>(values: C) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => _ssrg_collection_all(__ssrg$evidence$0, (value: bigint) => value > 0n, values)
const emptyValues = (unit: undefined) => [] as ReadonlyArray<bigint>
const yesNo = (value: boolean) => value ? "true" : "false"
const number = (value: bigint) => _ssrg_show_intShow["show"](value)
const productArray = (unit: undefined) => number(productValues([2n, 3n, 4n])(_ssrg_array_reducible))
const productList = (unit: undefined) => number(productValues(_ssrg_list_from_array([2n, 3n, 4n]))(_ssrg_list_reducible))
const productRange = (unit: undefined) => number(productValues(_ssrg_range_inclusive(1n, 4n))(_ssrg_range_reducible))
const productEmpty = (unit: undefined) => number(productValues(emptyValues(undefined))(_ssrg_array_reducible))
const anyArray = (unit: undefined) => yesNo(anyPositive([0n, 0n, 2n])(_ssrg_array_iterable))
const anyList = (unit: undefined) => yesNo(anyPositive(_ssrg_list_from_array([0n, 0n, 0n]))(_ssrg_list_iterable))
const anyRange = (unit: undefined) => yesNo(anyPositive(_ssrg_range_inclusive(0n, 2n))(_ssrg_range_iterable))
const anyEmpty = (unit: undefined) => yesNo(anyPositive(emptyValues(undefined))(_ssrg_array_iterable))
const allArray = (unit: undefined) => yesNo(allPositive([1n, 2n, 3n])(_ssrg_array_iterable))
const allList = (unit: undefined) => yesNo(allPositive(_ssrg_list_from_array([1n, 2n, 3n]))(_ssrg_list_iterable))
const allRange = (unit: undefined) => yesNo(allPositive(_ssrg_range_inclusive(1n, 3n))(_ssrg_range_iterable))
const allEmpty = (unit: undefined) => yesNo(allPositive(emptyValues(undefined))(_ssrg_array_iterable))
export const collectionAggregateResults = (unit: undefined) => [productArray(undefined), productList(undefined), productRange(undefined), productEmpty(undefined), anyArray(undefined), anyList(undefined), anyRange(undefined), anyEmpty(undefined), allArray(undefined), allList(undefined), allRange(undefined), allEmpty(undefined)]
