import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { arrayReducible as _ssrg_array_reducible, collectMap as _ssrg_array_comprehend, toList as _ssrg_array_toList } from "@seseragi/runtime/array"
import { join as _ssrg_collection_join } from "@seseragi/runtime/collection"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { collectMap as _ssrg_list_comprehend, toArray as _ssrg_list_toArray, fromArray as _ssrg_list_from_array, type List as List } from "@seseragi/runtime/list"
const $ssrg$source117 = { file: "main.ssrg", line: 5, column: 3, uri: "seseragi://artifact/collection-convert", start: 117, end: 156 };
const $ssrg$source202 = { file: "main.ssrg", line: 8, column: 3, uri: "seseragi://artifact/collection-convert", start: 202, end: 241 };
const $ssrg$source313 = { file: "main.ssrg", line: 12, column: 5, uri: "seseragi://artifact/collection-convert", start: 313, end: 347 };
const $ssrg$source323 = { file: "main.ssrg", line: 12, column: 15, uri: "seseragi://artifact/collection-convert", start: 323, end: 346 };
const $ssrg$source353 = { file: "main.ssrg", line: 13, column: 5, uri: "seseragi://artifact/collection-convert", start: 353, end: 389 };
const $ssrg$source364 = { file: "main.ssrg", line: 13, column: 16, uri: "seseragi://artifact/collection-convert", start: 364, end: 388 };
const $ssrg$source395 = { file: "main.ssrg", line: 14, column: 5, uri: "seseragi://artifact/collection-convert", start: 395, end: 446 };
const $ssrg$source406 = { file: "main.ssrg", line: 14, column: 16, uri: "seseragi://artifact/collection-convert", start: 406, end: 445 };
const $ssrg$source421 = { file: "main.ssrg", line: 14, column: 31, uri: "seseragi://artifact/collection-convert", start: 421, end: 444 };
const $ssrg$source452 = { file: "main.ssrg", line: 15, column: 5, uri: "seseragi://artifact/collection-convert", start: 452, end: 503 };
const $ssrg$source462 = { file: "main.ssrg", line: 15, column: 15, uri: "seseragi://artifact/collection-convert", start: 462, end: 502 };
const $ssrg$source477 = { file: "main.ssrg", line: 15, column: 30, uri: "seseragi://artifact/collection-convert", start: 477, end: 501 };
const $ssrg$source509 = { file: "main.ssrg", line: 16, column: 5, uri: "seseragi://artifact/collection-convert", start: 509, end: 536 };
const $ssrg$source519 = { file: "main.ssrg", line: 16, column: 15, uri: "seseragi://artifact/collection-convert", start: 519, end: 535 };
const $ssrg$source542 = { file: "main.ssrg", line: 17, column: 5, uri: "seseragi://artifact/collection-convert", start: 542, end: 571 };
const $ssrg$source553 = { file: "main.ssrg", line: 17, column: 16, uri: "seseragi://artifact/collection-convert", start: 553, end: 570 };
$ssrg$assertUnicodeVersion("17.0.0")

const showArray = (values: ReadonlyArray<number>) => $ssrg$atSource($ssrg$source117, () => _ssrg_collection_join(_ssrg_array_reducible, ",", _ssrg_array_comprehend(values, (value) => true, (value) => _ssrg_show_intShow["show"](value))), false)
const showList = (values: List<number>) => $ssrg$atSource($ssrg$source202, () => _ssrg_collection_join(_ssrg_array_reducible, ",", _ssrg_list_comprehend(values, (value) => true, (value) => _ssrg_show_intShow["show"](value))), false)
export const collectionConvertResults = (unit: undefined) => [$ssrg$atSource($ssrg$source313, () => showList($ssrg$atSource($ssrg$source323, () => _ssrg_array_toList([1, 2, 3]), false)), false), $ssrg$atSource($ssrg$source353, () => showArray($ssrg$atSource($ssrg$source364, () => _ssrg_list_toArray(_ssrg_list_from_array([4, 5, 6])), false)), false), $ssrg$atSource($ssrg$source395, () => showArray($ssrg$atSource($ssrg$source406, () => _ssrg_list_toArray($ssrg$atSource($ssrg$source421, () => _ssrg_array_toList([1, 2, 3]), false)), false)), false), $ssrg$atSource($ssrg$source452, () => showList($ssrg$atSource($ssrg$source462, () => _ssrg_array_toList($ssrg$atSource($ssrg$source477, () => _ssrg_list_toArray(_ssrg_list_from_array([4, 5, 6])), false)), false)), false), $ssrg$atSource($ssrg$source509, () => showList($ssrg$atSource($ssrg$source519, () => _ssrg_array_toList([] as ReadonlyArray<number>), false)), false), $ssrg$atSource($ssrg$source542, () => showArray($ssrg$atSource($ssrg$source553, () => _ssrg_list_toArray(_ssrg_list_from_array([] as ReadonlyArray<number>)), false)), false)]
