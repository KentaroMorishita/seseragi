import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { intZero as _ssrg_int_zero, intAdd as _ssrg_int_add_dictionary } from "@seseragi/runtime/int"
import { sum as _ssrg_collection_sum } from "@seseragi/runtime/collection"
import { arrayReducible as _ssrg_array_reducible } from "@seseragi/runtime/array"
import { listReducible as _ssrg_list_reducible, fromArray as _ssrg_list_from_array, type List as List } from "@seseragi/runtime/list"
import { rangeReducible as _ssrg_range_reducible, inclusive as _ssrg_range_inclusive } from "@seseragi/runtime/range"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
const $ssrg$source61 = { file: "main.ssrg", line: 3, column: 3, uri: "seseragi://artifact/collection-sum", start: 61, end: 71 };
const $ssrg$source157 = { file: "main.ssrg", line: 9, column: 3, uri: "seseragi://artifact/collection-sum", start: 157, end: 175 };
const $ssrg$source212 = { file: "main.ssrg", line: 12, column: 3, uri: "seseragi://artifact/collection-sum", start: 212, end: 235 };
const $ssrg$source273 = { file: "main.ssrg", line: 15, column: 3, uri: "seseragi://artifact/collection-sum", start: 273, end: 290 };
const $ssrg$source284 = { file: "main.ssrg", line: 15, column: 14, uri: "seseragi://artifact/collection-sum", start: 284, end: 289 };
const $ssrg$source328 = { file: "main.ssrg", line: 18, column: 3, uri: "seseragi://artifact/collection-sum", start: 328, end: 354 };
const $ssrg$source339 = { file: "main.ssrg", line: 18, column: 14, uri: "seseragi://artifact/collection-sum", start: 339, end: 353 };
const $ssrg$source425 = { file: "main.ssrg", line: 22, column: 8, uri: "seseragi://artifact/collection-sum", start: 425, end: 438 };
const $ssrg$source449 = { file: "main.ssrg", line: 23, column: 8, uri: "seseragi://artifact/collection-sum", start: 449, end: 461 };
const $ssrg$source472 = { file: "main.ssrg", line: 24, column: 8, uri: "seseragi://artifact/collection-sum", start: 472, end: 485 };
const $ssrg$source496 = { file: "main.ssrg", line: 25, column: 8, uri: "seseragi://artifact/collection-sum", start: 496, end: 509 };
$ssrg$assertUnicodeVersion("17.0.0")

const sumValues = <C,>(values: C) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => $ssrg$atSource($ssrg$source61, () => _ssrg_collection_sum(__ssrg$evidence$0, _ssrg_int_zero, _ssrg_int_add_dictionary, values), false)
const emptyValues = (unit: undefined) => [] as ReadonlyArray<number>
const arrayTotal = (unit: undefined) => $ssrg$atSource($ssrg$source157, () => ((sumValues<ReadonlyArray<number>>([20, 22])(_ssrg_array_reducible)) as number), false)
const listTotal = (unit: undefined) => $ssrg$atSource($ssrg$source212, () => ((sumValues<List<number>>(_ssrg_list_from_array([10, 20, 12]))(_ssrg_list_reducible)) as number), false)
const rangeTotal = (unit: undefined) => $ssrg$atSource($ssrg$source273, () => ((sumValues<Readonly<{ start: number; end: number; inclusive: boolean }>>($ssrg$atSource($ssrg$source284, () => _ssrg_range_inclusive(1, 4), false))(_ssrg_range_reducible)) as number), false)
const emptyTotal = (unit: undefined) => $ssrg$atSource($ssrg$source328, () => ((sumValues<ReadonlyArray<number>>($ssrg$atSource($ssrg$source339, () => emptyValues(undefined), false))(_ssrg_array_reducible)) as number), false)
export const collectionSumResults = (unit: undefined) => [_ssrg_show_intShow["show"]($ssrg$atSource($ssrg$source425, () => arrayTotal(undefined), false)), _ssrg_show_intShow["show"]($ssrg$atSource($ssrg$source449, () => listTotal(undefined), false)), _ssrg_show_intShow["show"]($ssrg$atSource($ssrg$source472, () => rangeTotal(undefined), false)), _ssrg_show_intShow["show"]($ssrg$atSource($ssrg$source496, () => emptyTotal(undefined), false))]
