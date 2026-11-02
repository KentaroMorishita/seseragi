import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { intEq as _ssrg_int_eq_dictionary, intOrd as _ssrg_int_ord_dictionary } from "@seseragi/runtime/equality"
import { remainder as _ssrg_int_remainder, subtract as _ssrg_int_subtract } from "@seseragi/runtime/int"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { arrayReducible as _ssrg_array_reducible, collectMap as _ssrg_array_comprehend, find as _ssrg_array_find, take as _ssrg_array_take, drop as _ssrg_array_drop } from "@seseragi/runtime/array"
import { join as _ssrg_collection_join } from "@seseragi/runtime/collection"
import { collectMap as _ssrg_list_comprehend, find as _ssrg_list_find, fromArray as _ssrg_list_from_array, take as _ssrg_list_take, drop as _ssrg_list_drop, type List as List } from "@seseragi/runtime/list"
const $ssrg$source102 = { file: "main.ssrg", line: 5, column: 3, uri: "seseragi://artifact/collection-slice", start: 102, end: 111 };
const $ssrg$source323 = { file: "main.ssrg", line: 17, column: 3, uri: "seseragi://artifact/collection-slice", start: 323, end: 362 };
const $ssrg$source408 = { file: "main.ssrg", line: 20, column: 3, uri: "seseragi://artifact/collection-slice", start: 408, end: 447 };
const $ssrg$source517 = { file: "main.ssrg", line: 24, column: 5, uri: "seseragi://artifact/collection-slice", start: 517, end: 555 };
const $ssrg$source528 = { file: "main.ssrg", line: 24, column: 16, uri: "seseragi://artifact/collection-slice", start: 528, end: 554 };
const $ssrg$source561 = { file: "main.ssrg", line: 25, column: 5, uri: "seseragi://artifact/collection-slice", start: 561, end: 603 };
const $ssrg$source572 = { file: "main.ssrg", line: 25, column: 16, uri: "seseragi://artifact/collection-slice", start: 572, end: 602 };
const $ssrg$source609 = { file: "main.ssrg", line: 26, column: 5, uri: "seseragi://artifact/collection-slice", start: 609, end: 650 };
const $ssrg$source620 = { file: "main.ssrg", line: 26, column: 16, uri: "seseragi://artifact/collection-slice", start: 620, end: 649 };
const $ssrg$source633 = { file: "main.ssrg", line: 26, column: 29, uri: "seseragi://artifact/collection-slice", start: 633, end: 638 };
const $ssrg$source656 = { file: "main.ssrg", line: 27, column: 5, uri: "seseragi://artifact/collection-slice", start: 656, end: 691 };
const $ssrg$source667 = { file: "main.ssrg", line: 27, column: 16, uri: "seseragi://artifact/collection-slice", start: 667, end: 690 };
const $ssrg$source697 = { file: "main.ssrg", line: 28, column: 5, uri: "seseragi://artifact/collection-slice", start: 697, end: 733 };
const $ssrg$source708 = { file: "main.ssrg", line: 28, column: 16, uri: "seseragi://artifact/collection-slice", start: 708, end: 732 };
const $ssrg$source739 = { file: "main.ssrg", line: 29, column: 5, uri: "seseragi://artifact/collection-slice", start: 739, end: 780 };
const $ssrg$source750 = { file: "main.ssrg", line: 29, column: 16, uri: "seseragi://artifact/collection-slice", start: 750, end: 779 };
const $ssrg$source763 = { file: "main.ssrg", line: 29, column: 29, uri: "seseragi://artifact/collection-slice", start: 763, end: 768 };
const $ssrg$source786 = { file: "main.ssrg", line: 30, column: 5, uri: "seseragi://artifact/collection-slice", start: 786, end: 821 };
const $ssrg$source797 = { file: "main.ssrg", line: 30, column: 16, uri: "seseragi://artifact/collection-slice", start: 797, end: 820 };
const $ssrg$source827 = { file: "main.ssrg", line: 31, column: 5, uri: "seseragi://artifact/collection-slice", start: 827, end: 863 };
const $ssrg$source838 = { file: "main.ssrg", line: 31, column: 16, uri: "seseragi://artifact/collection-slice", start: 838, end: 862 };
const $ssrg$source869 = { file: "main.ssrg", line: 32, column: 5, uri: "seseragi://artifact/collection-slice", start: 869, end: 907 };
const $ssrg$source880 = { file: "main.ssrg", line: 32, column: 16, uri: "seseragi://artifact/collection-slice", start: 880, end: 906 };
const $ssrg$source913 = { file: "main.ssrg", line: 33, column: 5, uri: "seseragi://artifact/collection-slice", start: 913, end: 955 };
const $ssrg$source924 = { file: "main.ssrg", line: 33, column: 16, uri: "seseragi://artifact/collection-slice", start: 924, end: 954 };
const $ssrg$source961 = { file: "main.ssrg", line: 34, column: 5, uri: "seseragi://artifact/collection-slice", start: 961, end: 1001 };
const $ssrg$source971 = { file: "main.ssrg", line: 34, column: 15, uri: "seseragi://artifact/collection-slice", start: 971, end: 1000 };
const $ssrg$source983 = { file: "main.ssrg", line: 34, column: 27, uri: "seseragi://artifact/collection-slice", start: 983, end: 988 };
const $ssrg$source1007 = { file: "main.ssrg", line: 35, column: 5, uri: "seseragi://artifact/collection-slice", start: 1007, end: 1041 };
const $ssrg$source1017 = { file: "main.ssrg", line: 35, column: 15, uri: "seseragi://artifact/collection-slice", start: 1017, end: 1040 };
const $ssrg$source1047 = { file: "main.ssrg", line: 36, column: 5, uri: "seseragi://artifact/collection-slice", start: 1047, end: 1082 };
const $ssrg$source1057 = { file: "main.ssrg", line: 36, column: 15, uri: "seseragi://artifact/collection-slice", start: 1057, end: 1081 };
const $ssrg$source1088 = { file: "main.ssrg", line: 37, column: 5, uri: "seseragi://artifact/collection-slice", start: 1088, end: 1128 };
const $ssrg$source1098 = { file: "main.ssrg", line: 37, column: 15, uri: "seseragi://artifact/collection-slice", start: 1098, end: 1127 };
const $ssrg$source1110 = { file: "main.ssrg", line: 37, column: 27, uri: "seseragi://artifact/collection-slice", start: 1110, end: 1115 };
const $ssrg$source1134 = { file: "main.ssrg", line: 38, column: 5, uri: "seseragi://artifact/collection-slice", start: 1134, end: 1168 };
const $ssrg$source1144 = { file: "main.ssrg", line: 38, column: 15, uri: "seseragi://artifact/collection-slice", start: 1144, end: 1167 };
const $ssrg$source1174 = { file: "main.ssrg", line: 39, column: 5, uri: "seseragi://artifact/collection-slice", start: 1174, end: 1209 };
const $ssrg$source1184 = { file: "main.ssrg", line: 39, column: 15, uri: "seseragi://artifact/collection-slice", start: 1184, end: 1208 };
$ssrg$assertUnicodeVersion("17.0.0")

const even = (value: number) => $ssrg$atSource($ssrg$source102, () => _ssrg_int_eq_dictionary["eq"](_ssrg_int_remainder(value, 2, $ssrg$source102))(0), false)
const negative = (value: number) => (_ssrg_int_ord_dictionary["compare"](value)(0))["tag"] === "Less"
const showMaybe = (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }): string => $ssrg_match.tag === "Nothing" ? "none" : $ssrg_match.tag === "Just" ? ((item: number): string => _ssrg_show_intShow["show"](item))($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(value)
const showArray = (values: ReadonlyArray<number>) => $ssrg$atSource($ssrg$source323, () => _ssrg_collection_join(_ssrg_array_reducible, ",", _ssrg_array_comprehend(values, (value) => true, (value) => _ssrg_show_intShow["show"](value))), false)
const showList = (values: List<number>) => $ssrg$atSource($ssrg$source408, () => _ssrg_collection_join(_ssrg_array_reducible, ",", _ssrg_list_comprehend(values, (value) => true, (value) => _ssrg_show_intShow["show"](value))), false)
export const collectionSliceResults = (unit: undefined) => [$ssrg$atSource($ssrg$source517, () => showMaybe($ssrg$atSource($ssrg$source528, () => _ssrg_array_find(even, [1, 2, 4]), false)), false), $ssrg$atSource($ssrg$source561, () => showMaybe($ssrg$atSource($ssrg$source572, () => _ssrg_array_find(negative, [1, 2, 4]), false)), false), $ssrg$atSource($ssrg$source609, () => showArray($ssrg$atSource($ssrg$source620, () => _ssrg_array_take(_ssrg_int_subtract(0, 1, $ssrg$source633), [1, 2, 3]), false)), false), $ssrg$atSource($ssrg$source656, () => showArray($ssrg$atSource($ssrg$source667, () => _ssrg_array_take(2, [1, 2, 3]), false)), false), $ssrg$atSource($ssrg$source697, () => showArray($ssrg$atSource($ssrg$source708, () => _ssrg_array_take(20, [1, 2, 3]), false)), false), $ssrg$atSource($ssrg$source739, () => showArray($ssrg$atSource($ssrg$source750, () => _ssrg_array_drop(_ssrg_int_subtract(0, 1, $ssrg$source763), [1, 2, 3]), false)), false), $ssrg$atSource($ssrg$source786, () => showArray($ssrg$atSource($ssrg$source797, () => _ssrg_array_drop(2, [1, 2, 3]), false)), false), $ssrg$atSource($ssrg$source827, () => showArray($ssrg$atSource($ssrg$source838, () => _ssrg_array_drop(20, [1, 2, 3]), false)), false), $ssrg$atSource($ssrg$source869, () => showMaybe($ssrg$atSource($ssrg$source880, () => _ssrg_list_find(even, _ssrg_list_from_array([1, 2, 4])), false)), false), $ssrg$atSource($ssrg$source913, () => showMaybe($ssrg$atSource($ssrg$source924, () => _ssrg_list_find(negative, _ssrg_list_from_array([1, 2, 4])), false)), false), $ssrg$atSource($ssrg$source961, () => showList($ssrg$atSource($ssrg$source971, () => _ssrg_list_take(_ssrg_int_subtract(0, 1, $ssrg$source983), _ssrg_list_from_array([1, 2, 3])), false)), false), $ssrg$atSource($ssrg$source1007, () => showList($ssrg$atSource($ssrg$source1017, () => _ssrg_list_take(2, _ssrg_list_from_array([1, 2, 3])), false)), false), $ssrg$atSource($ssrg$source1047, () => showList($ssrg$atSource($ssrg$source1057, () => _ssrg_list_take(20, _ssrg_list_from_array([1, 2, 3])), false)), false), $ssrg$atSource($ssrg$source1088, () => showList($ssrg$atSource($ssrg$source1098, () => _ssrg_list_drop(_ssrg_int_subtract(0, 1, $ssrg$source1110), _ssrg_list_from_array([1, 2, 3])), false)), false), $ssrg$atSource($ssrg$source1134, () => showList($ssrg$atSource($ssrg$source1144, () => _ssrg_list_drop(2, _ssrg_list_from_array([1, 2, 3])), false)), false), $ssrg$atSource($ssrg$source1174, () => showList($ssrg$atSource($ssrg$source1184, () => _ssrg_list_drop(20, _ssrg_list_from_array([1, 2, 3])), false)), false)]
