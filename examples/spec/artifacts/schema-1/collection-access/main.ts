import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { fromArray as _ssrg_list_from_array, length as _ssrg_list_length, isEmpty as _ssrg_list_isEmpty, get as _ssrg_list_get, head as _ssrg_list_head, tail as _ssrg_list_tail, type List as List } from "@seseragi/runtime/list"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { length as _ssrg_array_length, isEmpty as _ssrg_array_isEmpty, get as _ssrg_array_get, head as _ssrg_array_head, tail as _ssrg_array_tail } from "@seseragi/runtime/array"
import { subtract as _ssrg_int_subtract } from "@seseragi/runtime/int"
const $ssrg$source382 = { file: "main.ssrg", line: 19, column: 18, uri: "seseragi://artifact/collection-access", start: 382, end: 393 };
const $ssrg$source510 = { file: "main.ssrg", line: 25, column: 19, uri: "seseragi://artifact/collection-access", start: 510, end: 538 };
const $ssrg$source518 = { file: "main.ssrg", line: 25, column: 27, uri: "seseragi://artifact/collection-access", start: 518, end: 537 };
const $ssrg$source653 = { file: "main.ssrg", line: 31, column: 19, uri: "seseragi://artifact/collection-access", start: 653, end: 680 };
const $ssrg$source661 = { file: "main.ssrg", line: 31, column: 27, uri: "seseragi://artifact/collection-access", start: 661, end: 679 };
const $ssrg$source755 = { file: "main.ssrg", line: 36, column: 5, uri: "seseragi://artifact/collection-access", start: 755, end: 786 };
const $ssrg$source763 = { file: "main.ssrg", line: 36, column: 13, uri: "seseragi://artifact/collection-access", start: 763, end: 785 };
const $ssrg$source792 = { file: "main.ssrg", line: 37, column: 5, uri: "seseragi://artifact/collection-access", start: 792, end: 830 };
const $ssrg$source799 = { file: "main.ssrg", line: 37, column: 12, uri: "seseragi://artifact/collection-access", start: 799, end: 829 };
const $ssrg$source815 = { file: "main.ssrg", line: 37, column: 28, uri: "seseragi://artifact/collection-access", start: 815, end: 828 };
const $ssrg$source836 = { file: "main.ssrg", line: 38, column: 5, uri: "seseragi://artifact/collection-access", start: 836, end: 872 };
const $ssrg$source850 = { file: "main.ssrg", line: 38, column: 19, uri: "seseragi://artifact/collection-access", start: 850, end: 871 };
const $ssrg$source878 = { file: "main.ssrg", line: 39, column: 5, uri: "seseragi://artifact/collection-access", start: 878, end: 920 };
const $ssrg$source892 = { file: "main.ssrg", line: 39, column: 19, uri: "seseragi://artifact/collection-access", start: 892, end: 919 };
const $ssrg$source904 = { file: "main.ssrg", line: 39, column: 31, uri: "seseragi://artifact/collection-access", start: 904, end: 909 };
const $ssrg$source926 = { file: "main.ssrg", line: 40, column: 5, uri: "seseragi://artifact/collection-access", start: 926, end: 962 };
const $ssrg$source940 = { file: "main.ssrg", line: 40, column: 19, uri: "seseragi://artifact/collection-access", start: 940, end: 961 };
const $ssrg$source968 = { file: "main.ssrg", line: 41, column: 5, uri: "seseragi://artifact/collection-access", start: 968, end: 1003 };
const $ssrg$source982 = { file: "main.ssrg", line: 41, column: 19, uri: "seseragi://artifact/collection-access", start: 982, end: 1002 };
const $ssrg$source1009 = { file: "main.ssrg", line: 42, column: 5, uri: "seseragi://artifact/collection-access", start: 1009, end: 1051 };
const $ssrg$source1023 = { file: "main.ssrg", line: 42, column: 19, uri: "seseragi://artifact/collection-access", start: 1023, end: 1050 };
const $ssrg$source1036 = { file: "main.ssrg", line: 42, column: 32, uri: "seseragi://artifact/collection-access", start: 1036, end: 1049 };
const $ssrg$source1057 = { file: "main.ssrg", line: 43, column: 5, uri: "seseragi://artifact/collection-access", start: 1057, end: 1090 };
const $ssrg$source1073 = { file: "main.ssrg", line: 43, column: 21, uri: "seseragi://artifact/collection-access", start: 1073, end: 1089 };
const $ssrg$source1096 = { file: "main.ssrg", line: 44, column: 5, uri: "seseragi://artifact/collection-access", start: 1096, end: 1140 };
const $ssrg$source1112 = { file: "main.ssrg", line: 44, column: 21, uri: "seseragi://artifact/collection-access", start: 1112, end: 1139 };
const $ssrg$source1125 = { file: "main.ssrg", line: 44, column: 34, uri: "seseragi://artifact/collection-access", start: 1125, end: 1138 };
const $ssrg$source1146 = { file: "main.ssrg", line: 45, column: 5, uri: "seseragi://artifact/collection-access", start: 1146, end: 1177 };
const $ssrg$source1154 = { file: "main.ssrg", line: 45, column: 13, uri: "seseragi://artifact/collection-access", start: 1154, end: 1176 };
const $ssrg$source1183 = { file: "main.ssrg", line: 46, column: 5, uri: "seseragi://artifact/collection-access", start: 1183, end: 1219 };
const $ssrg$source1190 = { file: "main.ssrg", line: 46, column: 12, uri: "seseragi://artifact/collection-access", start: 1190, end: 1218 };
const $ssrg$source1205 = { file: "main.ssrg", line: 46, column: 27, uri: "seseragi://artifact/collection-access", start: 1205, end: 1217 };
const $ssrg$source1225 = { file: "main.ssrg", line: 47, column: 5, uri: "seseragi://artifact/collection-access", start: 1225, end: 1261 };
const $ssrg$source1239 = { file: "main.ssrg", line: 47, column: 19, uri: "seseragi://artifact/collection-access", start: 1239, end: 1260 };
const $ssrg$source1267 = { file: "main.ssrg", line: 48, column: 5, uri: "seseragi://artifact/collection-access", start: 1267, end: 1309 };
const $ssrg$source1281 = { file: "main.ssrg", line: 48, column: 19, uri: "seseragi://artifact/collection-access", start: 1281, end: 1308 };
const $ssrg$source1292 = { file: "main.ssrg", line: 48, column: 30, uri: "seseragi://artifact/collection-access", start: 1292, end: 1297 };
const $ssrg$source1315 = { file: "main.ssrg", line: 49, column: 5, uri: "seseragi://artifact/collection-access", start: 1315, end: 1351 };
const $ssrg$source1329 = { file: "main.ssrg", line: 49, column: 19, uri: "seseragi://artifact/collection-access", start: 1329, end: 1350 };
const $ssrg$source1357 = { file: "main.ssrg", line: 50, column: 5, uri: "seseragi://artifact/collection-access", start: 1357, end: 1392 };
const $ssrg$source1371 = { file: "main.ssrg", line: 50, column: 19, uri: "seseragi://artifact/collection-access", start: 1371, end: 1391 };
const $ssrg$source1398 = { file: "main.ssrg", line: 51, column: 5, uri: "seseragi://artifact/collection-access", start: 1398, end: 1438 };
const $ssrg$source1412 = { file: "main.ssrg", line: 51, column: 19, uri: "seseragi://artifact/collection-access", start: 1412, end: 1437 };
const $ssrg$source1424 = { file: "main.ssrg", line: 51, column: 31, uri: "seseragi://artifact/collection-access", start: 1424, end: 1436 };
const $ssrg$source1444 = { file: "main.ssrg", line: 52, column: 5, uri: "seseragi://artifact/collection-access", start: 1444, end: 1476 };
const $ssrg$source1459 = { file: "main.ssrg", line: 52, column: 20, uri: "seseragi://artifact/collection-access", start: 1459, end: 1475 };
const $ssrg$source1482 = { file: "main.ssrg", line: 53, column: 5, uri: "seseragi://artifact/collection-access", start: 1482, end: 1523 };
const $ssrg$source1497 = { file: "main.ssrg", line: 53, column: 20, uri: "seseragi://artifact/collection-access", start: 1497, end: 1522 };
const $ssrg$source1509 = { file: "main.ssrg", line: 53, column: 32, uri: "seseragi://artifact/collection-access", start: 1509, end: 1521 };
$ssrg$assertUnicodeVersion("17.0.0")

const emptyArray = (unit: undefined) => [] as ReadonlyArray<number>
const emptyList = (unit: undefined) => _ssrg_list_from_array([] as ReadonlyArray<number>)
const number = (value: number) => _ssrg_show_intShow["show"](value)
const yesNo = (value: boolean) => value ? "true" : "false"
const showMaybeInt = (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }): string => $ssrg_match.tag === "Nothing" ? "none" : $ssrg_match.tag === "Just" ? ((item: number): string => $ssrg$atSource($ssrg$source382, () => number(item), false))($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(value)
const showMaybeArray = (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: ReadonlyArray<number> }) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: ReadonlyArray<number> }): string => $ssrg_match.tag === "Nothing" ? "none" : $ssrg_match.tag === "Just" ? ((items: ReadonlyArray<number>): string => $ssrg$atSource($ssrg$source510, () => number($ssrg$atSource($ssrg$source518, () => _ssrg_array_length(items), false)), false))($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(value)
const showMaybeList = (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: List<number> }) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: List<number> }): string => $ssrg_match.tag === "Nothing" ? "none" : $ssrg_match.tag === "Just" ? ((items: List<number>): string => $ssrg$atSource($ssrg$source653, () => number($ssrg$atSource($ssrg$source661, () => _ssrg_list_length(items), false)), false))($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(value)
export const collectionAccessResults = (unit: undefined) => [$ssrg$atSource($ssrg$source755, () => number($ssrg$atSource($ssrg$source763, () => _ssrg_array_length([10, 20]), false)), false), $ssrg$atSource($ssrg$source792, () => yesNo($ssrg$atSource($ssrg$source799, () => _ssrg_array_isEmpty($ssrg$atSource($ssrg$source815, () => emptyArray(undefined), false)), false)), false), $ssrg$atSource($ssrg$source836, () => showMaybeInt($ssrg$atSource($ssrg$source850, () => _ssrg_array_get(1, [10, 20]), false)), false), $ssrg$atSource($ssrg$source878, () => showMaybeInt($ssrg$atSource($ssrg$source892, () => _ssrg_array_get(_ssrg_int_subtract(0, 1, $ssrg$source904), [10, 20]), false)), false), $ssrg$atSource($ssrg$source926, () => showMaybeInt($ssrg$atSource($ssrg$source940, () => _ssrg_array_get(2, [10, 20]), false)), false), $ssrg$atSource($ssrg$source968, () => showMaybeInt($ssrg$atSource($ssrg$source982, () => _ssrg_array_head([10, 20]), false)), false), $ssrg$atSource($ssrg$source1009, () => showMaybeInt($ssrg$atSource($ssrg$source1023, () => _ssrg_array_head($ssrg$atSource($ssrg$source1036, () => emptyArray(undefined), false)), false)), false), $ssrg$atSource($ssrg$source1057, () => showMaybeArray($ssrg$atSource($ssrg$source1073, () => _ssrg_array_tail([10]), false)), false), $ssrg$atSource($ssrg$source1096, () => showMaybeArray($ssrg$atSource($ssrg$source1112, () => _ssrg_array_tail($ssrg$atSource($ssrg$source1125, () => emptyArray(undefined), false)), false)), false), $ssrg$atSource($ssrg$source1146, () => number($ssrg$atSource($ssrg$source1154, () => _ssrg_list_length(_ssrg_list_from_array([10, 20])), false)), false), $ssrg$atSource($ssrg$source1183, () => yesNo($ssrg$atSource($ssrg$source1190, () => _ssrg_list_isEmpty($ssrg$atSource($ssrg$source1205, () => emptyList(undefined), false)), false)), false), $ssrg$atSource($ssrg$source1225, () => showMaybeInt($ssrg$atSource($ssrg$source1239, () => _ssrg_list_get(1, _ssrg_list_from_array([10, 20])), false)), false), $ssrg$atSource($ssrg$source1267, () => showMaybeInt($ssrg$atSource($ssrg$source1281, () => _ssrg_list_get(_ssrg_int_subtract(0, 1, $ssrg$source1292), _ssrg_list_from_array([10, 20])), false)), false), $ssrg$atSource($ssrg$source1315, () => showMaybeInt($ssrg$atSource($ssrg$source1329, () => _ssrg_list_get(2, _ssrg_list_from_array([10, 20])), false)), false), $ssrg$atSource($ssrg$source1357, () => showMaybeInt($ssrg$atSource($ssrg$source1371, () => _ssrg_list_head(_ssrg_list_from_array([10, 20])), false)), false), $ssrg$atSource($ssrg$source1398, () => showMaybeInt($ssrg$atSource($ssrg$source1412, () => _ssrg_list_head($ssrg$atSource($ssrg$source1424, () => emptyList(undefined), false)), false)), false), $ssrg$atSource($ssrg$source1444, () => showMaybeList($ssrg$atSource($ssrg$source1459, () => _ssrg_list_tail(_ssrg_list_from_array([10])), false)), false), $ssrg$atSource($ssrg$source1482, () => showMaybeList($ssrg$atSource($ssrg$source1497, () => _ssrg_list_tail($ssrg$atSource($ssrg$source1509, () => emptyList(undefined), false)), false)), false)]
