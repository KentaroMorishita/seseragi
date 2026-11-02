import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
import { remainder as _ssrg_int_remainder, subtract as _ssrg_int_subtract } from "@seseragi/runtime/int"
import { Just as _ssrg_maybe_Just, Nothing as _ssrg_maybe_Nothing } from "@seseragi/runtime/sum"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { fromArray as _ssrg_list_from_array, collectMap as _ssrg_list_comprehend, filter as _ssrg_list_filter, listReducible as _ssrg_list_reducible, filterMap as _ssrg_list_filterMap, flatMap as _ssrg_list_flatMap, type List as List } from "@seseragi/runtime/list"
import { arrayReducible as _ssrg_array_reducible, collectMap as _ssrg_array_comprehend, filter as _ssrg_array_filter, filterMap as _ssrg_array_filterMap, flatMap as _ssrg_array_flatMap } from "@seseragi/runtime/array"
import { join as _ssrg_collection_join } from "@seseragi/runtime/collection"
const $ssrg$source102 = { file: "main.ssrg", line: 5, column: 3, uri: "seseragi://artifact/collection-transform", start: 102, end: 111 };
const $ssrg$source166 = { file: "main.ssrg", line: 8, column: 6, uri: "seseragi://artifact/collection-transform", start: 166, end: 176 };
const $ssrg$source182 = { file: "main.ssrg", line: 8, column: 22, uri: "seseragi://artifact/collection-transform", start: 182, end: 198 };
const $ssrg$source265 = { file: "main.ssrg", line: 11, column: 11, uri: "seseragi://artifact/collection-transform", start: 265, end: 274 };
const $ssrg$source328 = { file: "main.ssrg", line: 14, column: 12, uri: "seseragi://artifact/collection-transform", start: 328, end: 337 };
const $ssrg$source386 = { file: "main.ssrg", line: 17, column: 3, uri: "seseragi://artifact/collection-transform", start: 386, end: 425 };
const $ssrg$source471 = { file: "main.ssrg", line: 20, column: 3, uri: "seseragi://artifact/collection-transform", start: 471, end: 510 };
const $ssrg$source584 = { file: "main.ssrg", line: 24, column: 5, uri: "seseragi://artifact/collection-transform", start: 584, end: 627 };
const $ssrg$source595 = { file: "main.ssrg", line: 24, column: 16, uri: "seseragi://artifact/collection-transform", start: 595, end: 626 };
const $ssrg$source633 = { file: "main.ssrg", line: 25, column: 5, uri: "seseragi://artifact/collection-transform", start: 633, end: 675 };
const $ssrg$source643 = { file: "main.ssrg", line: 25, column: 15, uri: "seseragi://artifact/collection-transform", start: 643, end: 674 };
const $ssrg$source681 = { file: "main.ssrg", line: 26, column: 5, uri: "seseragi://artifact/collection-transform", start: 681, end: 731 };
const $ssrg$source691 = { file: "main.ssrg", line: 26, column: 15, uri: "seseragi://artifact/collection-transform", start: 691, end: 730 };
const $ssrg$source737 = { file: "main.ssrg", line: 27, column: 5, uri: "seseragi://artifact/collection-transform", start: 737, end: 787 };
const $ssrg$source747 = { file: "main.ssrg", line: 27, column: 15, uri: "seseragi://artifact/collection-transform", start: 747, end: 786 };
const $ssrg$source793 = { file: "main.ssrg", line: 28, column: 5, uri: "seseragi://artifact/collection-transform", start: 793, end: 838 };
const $ssrg$source804 = { file: "main.ssrg", line: 28, column: 16, uri: "seseragi://artifact/collection-transform", start: 804, end: 837 };
const $ssrg$source844 = { file: "main.ssrg", line: 29, column: 5, uri: "seseragi://artifact/collection-transform", start: 844, end: 887 };
const $ssrg$source854 = { file: "main.ssrg", line: 29, column: 15, uri: "seseragi://artifact/collection-transform", start: 854, end: 886 };
$ssrg$assertUnicodeVersion("17.0.0")

const even = (value: number) => $ssrg$atSource($ssrg$source102, () => _ssrg_int_eq_dictionary["eq"](_ssrg_int_remainder(value, 2, $ssrg$source102))(0), false)
const labelEven = (value: number) => $ssrg$atSource($ssrg$source166, () => even(value), false) ? $ssrg$atSource($ssrg$source182, () => _ssrg_maybe_Just("#" + _ssrg_show_intShow["show"](value)), false) : _ssrg_maybe_Nothing
const signedArray = (value: number) => [value, _ssrg_int_subtract(0, value, $ssrg$source265)]
const signedList = (value: number) => _ssrg_list_from_array([value, _ssrg_int_subtract(0, value, $ssrg$source328)])
const showArray = (values: ReadonlyArray<number>) => $ssrg$atSource($ssrg$source386, () => _ssrg_collection_join(_ssrg_array_reducible, ",", _ssrg_array_comprehend(values, (value) => true, (value) => _ssrg_show_intShow["show"](value))), false)
const showList = (values: List<number>) => $ssrg$atSource($ssrg$source471, () => _ssrg_collection_join(_ssrg_array_reducible, ",", _ssrg_list_comprehend(values, (value) => true, (value) => _ssrg_show_intShow["show"](value))), false)
export const collectionTransformResults = (unit: undefined) => [$ssrg$atSource($ssrg$source584, () => showArray($ssrg$atSource($ssrg$source595, () => _ssrg_array_filter(even, [1, 2, 3, 4]), false)), false), $ssrg$atSource($ssrg$source633, () => showList($ssrg$atSource($ssrg$source643, () => _ssrg_list_filter(even, _ssrg_list_from_array([1, 2, 3, 4])), false)), false), $ssrg$atSource($ssrg$source681, () => _ssrg_collection_join(_ssrg_array_reducible, ",", $ssrg$atSource($ssrg$source691, () => _ssrg_array_filterMap(labelEven, [1, 2, 3, 4]), false)), false), $ssrg$atSource($ssrg$source737, () => _ssrg_collection_join(_ssrg_list_reducible, ",", $ssrg$atSource($ssrg$source747, () => _ssrg_list_filterMap(labelEven, _ssrg_list_from_array([1, 2, 3, 4])), false)), false), $ssrg$atSource($ssrg$source793, () => showArray($ssrg$atSource($ssrg$source804, () => _ssrg_array_flatMap(signedArray, [1, 2]), false)), false), $ssrg$atSource($ssrg$source844, () => showList($ssrg$atSource($ssrg$source854, () => _ssrg_list_flatMap(signedList, _ssrg_list_from_array([1, 2])), false)), false)]
