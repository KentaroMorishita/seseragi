import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { multiply as _ssrg_int_multiply, remainder as _ssrg_int_remainder, add as _ssrg_int_add } from "@seseragi/runtime/int"
import { collectMap as _ssrg_range_comprehend, inclusive as _ssrg_range_inclusive } from "@seseragi/runtime/range"
import { intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
import { reduce as _ssrg_array_reduce } from "@seseragi/runtime/array"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source45 = { file: "main.ssrg", line: 2, column: 4, uri: "seseragi://artifact/range-comprehension", start: 45, end: 58 };
const $ssrg$source70 = { file: "main.ssrg", line: 2, column: 29, uri: "seseragi://artifact/range-comprehension", start: 70, end: 79 };
const $ssrg$source81 = { file: "main.ssrg", line: 2, column: 40, uri: "seseragi://artifact/range-comprehension", start: 81, end: 90 };
const $ssrg$source129 = { file: "main.ssrg", line: 5, column: 3, uri: "seseragi://artifact/range-comprehension", start: 129, end: 143 };
const $ssrg$source325 = { file: "main.ssrg", line: 15, column: 3, uri: "seseragi://artifact/range-comprehension", start: 325, end: 333 };
$ssrg$assertUnicodeVersion("17.0.0")

const evenSquares = (limit: number) => _ssrg_range_comprehend($ssrg$atSource($ssrg$source70, () => _ssrg_range_inclusive(1, limit), false), (value) => $ssrg$atSource($ssrg$source81, () => _ssrg_int_eq_dictionary["eq"](_ssrg_int_remainder(value, 2, $ssrg$source81))(0), false), (value) => _ssrg_int_multiply(value, value, $ssrg$source45))
const total = (unit: undefined) => $ssrg$atSource($ssrg$source129, () => _ssrg_array_reduce(0, (((_argument0) => (_argument1) => _ssrg_int_add(_argument0, _argument1)) as (argument: number) => (argument: number) => number), $ssrg$atSource($ssrg$source129, () => evenSquares(10), false)), false)
const renderTotal = (total: number) => (($ssrg_match: number): string => $ssrg_match === 220 ? "even square total: 220" : "unexpected comprehension total")(total)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source325, () => _ssrg_console_println($ssrg$atSource($ssrg$source325, () => renderTotal($ssrg$atSource($ssrg$source325, () => total(undefined), false)), false)), true)
