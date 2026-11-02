import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { fromArray as _ssrg_list_from_array, collectMap as _ssrg_list_comprehend, reduce as _ssrg_list_reduce, type List as List } from "@seseragi/runtime/list"
import { multiply as _ssrg_int_multiply, remainder as _ssrg_int_remainder, add as _ssrg_int_add } from "@seseragi/runtime/int"
import { intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source51 = { file: "main.ssrg", line: 2, column: 5, uri: "seseragi://artifact/list-comprehension", start: 51, end: 64 };
const $ssrg$source84 = { file: "main.ssrg", line: 2, column: 38, uri: "seseragi://artifact/list-comprehension", start: 84, end: 93 };
const $ssrg$source139 = { file: "main.ssrg", line: 5, column: 3, uri: "seseragi://artifact/list-comprehension", start: 139, end: 165 };
const $ssrg$source320 = { file: "main.ssrg", line: 15, column: 3, uri: "seseragi://artifact/list-comprehension", start: 320, end: 354 };
$ssrg$assertUnicodeVersion("17.0.0")

const oddSquares = (values: List<number>) => _ssrg_list_from_array(_ssrg_list_comprehend(values, (value) => $ssrg$atSource($ssrg$source84, () => _ssrg_int_eq_dictionary["eq"](_ssrg_int_remainder(value, 2, $ssrg$source84))(1), false), (value) => _ssrg_int_multiply(value, value, $ssrg$source51)))
const total = (values: List<number>) => $ssrg$atSource($ssrg$source139, () => _ssrg_list_reduce(0, (((_argument0) => (_argument1) => _ssrg_int_add(_argument0, _argument1)) as (argument: number) => (argument: number) => number), values), false)
const renderTotal = (total: number) => (($ssrg_match: number): string => $ssrg_match === 35 ? "persistent List total: 35" : "unexpected List total")(total)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source320, () => _ssrg_console_println($ssrg$atSource($ssrg$source320, () => renderTotal($ssrg$atSource($ssrg$source320, () => total($ssrg$atSource($ssrg$source320, () => oddSquares(_ssrg_list_from_array([1, 2, 3, 4, 5])), false)), false)), false)), true)
