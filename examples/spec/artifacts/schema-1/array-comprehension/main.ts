import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { collectFlatMap as _ssrg_array_comprehend_flat, collectMap as _ssrg_array_comprehend } from "@seseragi/runtime/array"
import { intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
const $ssrg$source51 = { file: "main.ssrg", line: 2, column: 4, uri: "seseragi://artifact/array-comprehension", start: 51, end: 63 };
const $ssrg$source117 = { file: "main.ssrg", line: 5, column: 7, uri: "seseragi://artifact/array-comprehension", start: 117, end: 128 };
$ssrg$assertUnicodeVersion("17.0.0")

export const selectedPairs = (unit: undefined) => _ssrg_array_comprehend_flat([1, 2], (left) => true, (left) => _ssrg_array_comprehend([10, 20], (right) => $ssrg$atSource($ssrg$source117, () => _ssrg_int_eq_dictionary["eq"](right)(20), false), (right) => _ssrg_int_add(left, right, $ssrg$source51)))
