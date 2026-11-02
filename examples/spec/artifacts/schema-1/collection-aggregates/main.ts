import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { intOne as _ssrg_int_one, intMul as _ssrg_int_mul_dictionary } from "@seseragi/runtime/int"
import { product as _ssrg_collection_product, any as _ssrg_collection_any, all as _ssrg_collection_all } from "@seseragi/runtime/collection"
import { intOrd as _ssrg_int_ord_dictionary } from "@seseragi/runtime/equality"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { arrayReducible as _ssrg_array_reducible, arrayIterable as _ssrg_array_iterable } from "@seseragi/runtime/array"
import { listReducible as _ssrg_list_reducible, fromArray as _ssrg_list_from_array, listIterable as _ssrg_list_iterable, type List as List } from "@seseragi/runtime/list"
import { rangeReducible as _ssrg_range_reducible, inclusive as _ssrg_range_inclusive, rangeIterable as _ssrg_range_iterable } from "@seseragi/runtime/range"
const $ssrg$source65 = { file: "main.ssrg", line: 3, column: 3, uri: "seseragi://artifact/collection-aggregates", start: 65, end: 79 };
const $ssrg$source144 = { file: "main.ssrg", line: 7, column: 3, uri: "seseragi://artifact/collection-aggregates", start: 144, end: 181 };
const $ssrg$source246 = { file: "main.ssrg", line: 11, column: 3, uri: "seseragi://artifact/collection-aggregates", start: 246, end: 283 };
const $ssrg$source491 = { file: "main.ssrg", line: 23, column: 3, uri: "seseragi://artifact/collection-aggregates", start: 491, end: 523 };
const $ssrg$source499 = { file: "main.ssrg", line: 23, column: 11, uri: "seseragi://artifact/collection-aggregates", start: 499, end: 522 };
const $ssrg$source565 = { file: "main.ssrg", line: 26, column: 3, uri: "seseragi://artifact/collection-aggregates", start: 565, end: 598 };
const $ssrg$source573 = { file: "main.ssrg", line: 26, column: 11, uri: "seseragi://artifact/collection-aggregates", start: 573, end: 597 };
const $ssrg$source641 = { file: "main.ssrg", line: 29, column: 3, uri: "seseragi://artifact/collection-aggregates", start: 641, end: 671 };
const $ssrg$source649 = { file: "main.ssrg", line: 29, column: 11, uri: "seseragi://artifact/collection-aggregates", start: 649, end: 670 };
const $ssrg$source664 = { file: "main.ssrg", line: 29, column: 26, uri: "seseragi://artifact/collection-aggregates", start: 664, end: 669 };
const $ssrg$source714 = { file: "main.ssrg", line: 32, column: 3, uri: "seseragi://artifact/collection-aggregates", start: 714, end: 753 };
const $ssrg$source722 = { file: "main.ssrg", line: 32, column: 11, uri: "seseragi://artifact/collection-aggregates", start: 722, end: 752 };
const $ssrg$source737 = { file: "main.ssrg", line: 32, column: 26, uri: "seseragi://artifact/collection-aggregates", start: 737, end: 751 };
const $ssrg$source792 = { file: "main.ssrg", line: 35, column: 3, uri: "seseragi://artifact/collection-aggregates", start: 792, end: 821 };
const $ssrg$source799 = { file: "main.ssrg", line: 35, column: 10, uri: "seseragi://artifact/collection-aggregates", start: 799, end: 820 };
const $ssrg$source859 = { file: "main.ssrg", line: 38, column: 3, uri: "seseragi://artifact/collection-aggregates", start: 859, end: 889 };
const $ssrg$source866 = { file: "main.ssrg", line: 38, column: 10, uri: "seseragi://artifact/collection-aggregates", start: 866, end: 888 };
const $ssrg$source928 = { file: "main.ssrg", line: 41, column: 3, uri: "seseragi://artifact/collection-aggregates", start: 928, end: 955 };
const $ssrg$source935 = { file: "main.ssrg", line: 41, column: 10, uri: "seseragi://artifact/collection-aggregates", start: 935, end: 954 };
const $ssrg$source948 = { file: "main.ssrg", line: 41, column: 23, uri: "seseragi://artifact/collection-aggregates", start: 948, end: 953 };
const $ssrg$source994 = { file: "main.ssrg", line: 44, column: 3, uri: "seseragi://artifact/collection-aggregates", start: 994, end: 1030 };
const $ssrg$source1001 = { file: "main.ssrg", line: 44, column: 10, uri: "seseragi://artifact/collection-aggregates", start: 1001, end: 1029 };
const $ssrg$source1014 = { file: "main.ssrg", line: 44, column: 23, uri: "seseragi://artifact/collection-aggregates", start: 1014, end: 1028 };
const $ssrg$source1069 = { file: "main.ssrg", line: 47, column: 3, uri: "seseragi://artifact/collection-aggregates", start: 1069, end: 1098 };
const $ssrg$source1076 = { file: "main.ssrg", line: 47, column: 10, uri: "seseragi://artifact/collection-aggregates", start: 1076, end: 1097 };
const $ssrg$source1136 = { file: "main.ssrg", line: 50, column: 3, uri: "seseragi://artifact/collection-aggregates", start: 1136, end: 1166 };
const $ssrg$source1143 = { file: "main.ssrg", line: 50, column: 10, uri: "seseragi://artifact/collection-aggregates", start: 1143, end: 1165 };
const $ssrg$source1205 = { file: "main.ssrg", line: 53, column: 3, uri: "seseragi://artifact/collection-aggregates", start: 1205, end: 1232 };
const $ssrg$source1212 = { file: "main.ssrg", line: 53, column: 10, uri: "seseragi://artifact/collection-aggregates", start: 1212, end: 1231 };
const $ssrg$source1225 = { file: "main.ssrg", line: 53, column: 23, uri: "seseragi://artifact/collection-aggregates", start: 1225, end: 1230 };
const $ssrg$source1271 = { file: "main.ssrg", line: 56, column: 3, uri: "seseragi://artifact/collection-aggregates", start: 1271, end: 1307 };
const $ssrg$source1278 = { file: "main.ssrg", line: 56, column: 10, uri: "seseragi://artifact/collection-aggregates", start: 1278, end: 1306 };
const $ssrg$source1291 = { file: "main.ssrg", line: 56, column: 23, uri: "seseragi://artifact/collection-aggregates", start: 1291, end: 1305 };
const $ssrg$source1381 = { file: "main.ssrg", line: 60, column: 5, uri: "seseragi://artifact/collection-aggregates", start: 1381, end: 1396 };
const $ssrg$source1402 = { file: "main.ssrg", line: 61, column: 5, uri: "seseragi://artifact/collection-aggregates", start: 1402, end: 1416 };
const $ssrg$source1422 = { file: "main.ssrg", line: 62, column: 5, uri: "seseragi://artifact/collection-aggregates", start: 1422, end: 1437 };
const $ssrg$source1443 = { file: "main.ssrg", line: 63, column: 5, uri: "seseragi://artifact/collection-aggregates", start: 1443, end: 1458 };
const $ssrg$source1464 = { file: "main.ssrg", line: 64, column: 5, uri: "seseragi://artifact/collection-aggregates", start: 1464, end: 1475 };
const $ssrg$source1481 = { file: "main.ssrg", line: 65, column: 5, uri: "seseragi://artifact/collection-aggregates", start: 1481, end: 1491 };
const $ssrg$source1497 = { file: "main.ssrg", line: 66, column: 5, uri: "seseragi://artifact/collection-aggregates", start: 1497, end: 1508 };
const $ssrg$source1514 = { file: "main.ssrg", line: 67, column: 5, uri: "seseragi://artifact/collection-aggregates", start: 1514, end: 1525 };
const $ssrg$source1531 = { file: "main.ssrg", line: 68, column: 5, uri: "seseragi://artifact/collection-aggregates", start: 1531, end: 1542 };
const $ssrg$source1548 = { file: "main.ssrg", line: 69, column: 5, uri: "seseragi://artifact/collection-aggregates", start: 1548, end: 1558 };
const $ssrg$source1564 = { file: "main.ssrg", line: 70, column: 5, uri: "seseragi://artifact/collection-aggregates", start: 1564, end: 1575 };
const $ssrg$source1581 = { file: "main.ssrg", line: 71, column: 5, uri: "seseragi://artifact/collection-aggregates", start: 1581, end: 1592 };
$ssrg$assertUnicodeVersion("17.0.0")

const productValues = <C,>(values: C) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => $ssrg$atSource($ssrg$source65, () => _ssrg_collection_product(__ssrg$evidence$0, _ssrg_int_one, _ssrg_int_mul_dictionary, values), false)
const anyPositive = <C,>(values: C) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => $ssrg$atSource($ssrg$source144, () => _ssrg_collection_any(__ssrg$evidence$0, (value: number) => (_ssrg_int_ord_dictionary["compare"](value)(0))["tag"] === "Greater", values), false)
const allPositive = <C,>(values: C) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => $ssrg$atSource($ssrg$source246, () => _ssrg_collection_all(__ssrg$evidence$0, (value: number) => (_ssrg_int_ord_dictionary["compare"](value)(0))["tag"] === "Greater", values), false)
const emptyValues = (unit: undefined) => [] as ReadonlyArray<number>
const yesNo = (value: boolean) => value ? "true" : "false"
const number = (value: number) => _ssrg_show_intShow["show"](value)
const productArray = (unit: undefined) => $ssrg$atSource($ssrg$source491, () => number($ssrg$atSource($ssrg$source499, () => ((productValues<ReadonlyArray<number>>([2, 3, 4])(_ssrg_array_reducible)) as number), false)), false)
const productList = (unit: undefined) => $ssrg$atSource($ssrg$source565, () => number($ssrg$atSource($ssrg$source573, () => ((productValues<List<number>>(_ssrg_list_from_array([2, 3, 4]))(_ssrg_list_reducible)) as number), false)), false)
const productRange = (unit: undefined) => $ssrg$atSource($ssrg$source641, () => number($ssrg$atSource($ssrg$source649, () => ((productValues<Readonly<{ start: number; end: number; inclusive: boolean }>>($ssrg$atSource($ssrg$source664, () => _ssrg_range_inclusive(1, 4), false))(_ssrg_range_reducible)) as number), false)), false)
const productEmpty = (unit: undefined) => $ssrg$atSource($ssrg$source714, () => number($ssrg$atSource($ssrg$source722, () => ((productValues<ReadonlyArray<number>>($ssrg$atSource($ssrg$source737, () => emptyValues(undefined), false))(_ssrg_array_reducible)) as number), false)), false)
const anyArray = (unit: undefined) => $ssrg$atSource($ssrg$source792, () => yesNo($ssrg$atSource($ssrg$source799, () => ((anyPositive<ReadonlyArray<number>>([0, 0, 2])(_ssrg_array_iterable)) as boolean), false)), false)
const anyList = (unit: undefined) => $ssrg$atSource($ssrg$source859, () => yesNo($ssrg$atSource($ssrg$source866, () => ((anyPositive<List<number>>(_ssrg_list_from_array([0, 0, 0]))(_ssrg_list_iterable)) as boolean), false)), false)
const anyRange = (unit: undefined) => $ssrg$atSource($ssrg$source928, () => yesNo($ssrg$atSource($ssrg$source935, () => ((anyPositive<Readonly<{ start: number; end: number; inclusive: boolean }>>($ssrg$atSource($ssrg$source948, () => _ssrg_range_inclusive(0, 2), false))(_ssrg_range_iterable)) as boolean), false)), false)
const anyEmpty = (unit: undefined) => $ssrg$atSource($ssrg$source994, () => yesNo($ssrg$atSource($ssrg$source1001, () => ((anyPositive<ReadonlyArray<number>>($ssrg$atSource($ssrg$source1014, () => emptyValues(undefined), false))(_ssrg_array_iterable)) as boolean), false)), false)
const allArray = (unit: undefined) => $ssrg$atSource($ssrg$source1069, () => yesNo($ssrg$atSource($ssrg$source1076, () => ((allPositive<ReadonlyArray<number>>([1, 2, 3])(_ssrg_array_iterable)) as boolean), false)), false)
const allList = (unit: undefined) => $ssrg$atSource($ssrg$source1136, () => yesNo($ssrg$atSource($ssrg$source1143, () => ((allPositive<List<number>>(_ssrg_list_from_array([1, 2, 3]))(_ssrg_list_iterable)) as boolean), false)), false)
const allRange = (unit: undefined) => $ssrg$atSource($ssrg$source1205, () => yesNo($ssrg$atSource($ssrg$source1212, () => ((allPositive<Readonly<{ start: number; end: number; inclusive: boolean }>>($ssrg$atSource($ssrg$source1225, () => _ssrg_range_inclusive(1, 3), false))(_ssrg_range_iterable)) as boolean), false)), false)
const allEmpty = (unit: undefined) => $ssrg$atSource($ssrg$source1271, () => yesNo($ssrg$atSource($ssrg$source1278, () => ((allPositive<ReadonlyArray<number>>($ssrg$atSource($ssrg$source1291, () => emptyValues(undefined), false))(_ssrg_array_iterable)) as boolean), false)), false)
export const collectionAggregateResults = (unit: undefined) => [$ssrg$atSource($ssrg$source1381, () => productArray(undefined), false), $ssrg$atSource($ssrg$source1402, () => productList(undefined), false), $ssrg$atSource($ssrg$source1422, () => productRange(undefined), false), $ssrg$atSource($ssrg$source1443, () => productEmpty(undefined), false), $ssrg$atSource($ssrg$source1464, () => anyArray(undefined), false), $ssrg$atSource($ssrg$source1481, () => anyList(undefined), false), $ssrg$atSource($ssrg$source1497, () => anyRange(undefined), false), $ssrg$atSource($ssrg$source1514, () => anyEmpty(undefined), false), $ssrg$atSource($ssrg$source1531, () => allArray(undefined), false), $ssrg$atSource($ssrg$source1548, () => allList(undefined), false), $ssrg$atSource($ssrg$source1564, () => allRange(undefined), false), $ssrg$atSource($ssrg$source1581, () => allEmpty(undefined), false)]
