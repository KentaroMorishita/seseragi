import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { arrayReducible as _ssrg_array_reducible, collectMap as _ssrg_array_comprehend, append as _ssrg_array_append, concat as _ssrg_array_concat, reverse as _ssrg_array_reverse } from "@seseragi/runtime/array"
import { join as _ssrg_collection_join } from "@seseragi/runtime/collection"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { collectMap as _ssrg_list_comprehend, fromArray as _ssrg_list_from_array, append as _ssrg_list_append, concat as _ssrg_list_concat, reverse as _ssrg_list_reverse, type List as List } from "@seseragi/runtime/list"
const $ssrg$source117 = { file: "main.ssrg", line: 5, column: 3, uri: "seseragi://artifact/collection-compose", start: 117, end: 156 };
const $ssrg$source202 = { file: "main.ssrg", line: 8, column: 3, uri: "seseragi://artifact/collection-compose", start: 202, end: 241 };
const $ssrg$source390 = { file: "main.ssrg", line: 15, column: 5, uri: "seseragi://artifact/collection-compose", start: 390, end: 432 };
const $ssrg$source401 = { file: "main.ssrg", line: 15, column: 16, uri: "seseragi://artifact/collection-compose", start: 401, end: 431 };
const $ssrg$source438 = { file: "main.ssrg", line: 16, column: 5, uri: "seseragi://artifact/collection-compose", start: 438, end: 480 };
const $ssrg$source448 = { file: "main.ssrg", line: 16, column: 15, uri: "seseragi://artifact/collection-compose", start: 448, end: 479 };
const $ssrg$source486 = { file: "main.ssrg", line: 17, column: 5, uri: "seseragi://artifact/collection-compose", start: 486, end: 532 };
const $ssrg$source497 = { file: "main.ssrg", line: 17, column: 16, uri: "seseragi://artifact/collection-compose", start: 497, end: 531 };
const $ssrg$source538 = { file: "main.ssrg", line: 18, column: 5, uri: "seseragi://artifact/collection-compose", start: 538, end: 578 };
const $ssrg$source548 = { file: "main.ssrg", line: 18, column: 15, uri: "seseragi://artifact/collection-compose", start: 548, end: 577 };
const $ssrg$source562 = { file: "main.ssrg", line: 18, column: 29, uri: "seseragi://artifact/collection-compose", start: 562, end: 576 };
const $ssrg$source584 = { file: "main.ssrg", line: 19, column: 5, uri: "seseragi://artifact/collection-compose", start: 584, end: 620 };
const $ssrg$source595 = { file: "main.ssrg", line: 19, column: 16, uri: "seseragi://artifact/collection-compose", start: 595, end: 619 };
const $ssrg$source626 = { file: "main.ssrg", line: 20, column: 5, uri: "seseragi://artifact/collection-compose", start: 626, end: 661 };
const $ssrg$source636 = { file: "main.ssrg", line: 20, column: 15, uri: "seseragi://artifact/collection-compose", start: 636, end: 660 };
const $ssrg$source667 = { file: "main.ssrg", line: 21, column: 5, uri: "seseragi://artifact/collection-compose", start: 667, end: 695 };
const $ssrg$source678 = { file: "main.ssrg", line: 21, column: 16, uri: "seseragi://artifact/collection-compose", start: 678, end: 694 };
const $ssrg$source701 = { file: "main.ssrg", line: 22, column: 5, uri: "seseragi://artifact/collection-compose", start: 701, end: 728 };
const $ssrg$source711 = { file: "main.ssrg", line: 22, column: 15, uri: "seseragi://artifact/collection-compose", start: 711, end: 727 };
$ssrg$assertUnicodeVersion("17.0.0")

const showArray = (values: ReadonlyArray<number>) => $ssrg$atSource($ssrg$source117, () => _ssrg_collection_join(_ssrg_array_reducible, ",", _ssrg_array_comprehend(values, (value) => true, (value) => _ssrg_show_intShow["show"](value))), false)
const showList = (values: List<number>) => $ssrg$atSource($ssrg$source202, () => _ssrg_collection_join(_ssrg_array_reducible, ",", _ssrg_list_comprehend(values, (value) => true, (value) => _ssrg_show_intShow["show"](value))), false)
const nestedLists = (unit: undefined) => _ssrg_list_from_array([_ssrg_list_from_array([1, 2]), _ssrg_list_from_array([] as ReadonlyArray<number>), _ssrg_list_from_array([3, 4])])
export const collectionComposeResults = (unit: undefined) => [$ssrg$atSource($ssrg$source390, () => showArray($ssrg$atSource($ssrg$source401, () => _ssrg_array_append<number>([3, 4], [1, 2]), false)), false), $ssrg$atSource($ssrg$source438, () => showList($ssrg$atSource($ssrg$source448, () => _ssrg_list_append(_ssrg_list_from_array([3, 4]), _ssrg_list_from_array([1, 2])), false)), false), $ssrg$atSource($ssrg$source486, () => showArray($ssrg$atSource($ssrg$source497, () => _ssrg_array_concat([[1, 2], [] as ReadonlyArray<number>, [3, 4]]), false)), false), $ssrg$atSource($ssrg$source538, () => showList($ssrg$atSource($ssrg$source548, () => _ssrg_list_concat($ssrg$atSource($ssrg$source562, () => nestedLists(undefined), false)), false)), false), $ssrg$atSource($ssrg$source584, () => showArray($ssrg$atSource($ssrg$source595, () => _ssrg_array_reverse([1, 2, 3]), false)), false), $ssrg$atSource($ssrg$source626, () => showList($ssrg$atSource($ssrg$source636, () => _ssrg_list_reverse(_ssrg_list_from_array([1, 2, 3])), false)), false), $ssrg$atSource($ssrg$source667, () => showArray($ssrg$atSource($ssrg$source678, () => _ssrg_array_concat([] as ReadonlyArray<ReadonlyArray<number>>), false)), false), $ssrg$atSource($ssrg$source701, () => showList($ssrg$atSource($ssrg$source711, () => _ssrg_list_concat(_ssrg_list_from_array([] as ReadonlyArray<List<number>>)), false)), false)]
