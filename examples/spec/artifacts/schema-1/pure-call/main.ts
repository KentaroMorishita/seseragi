import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
const $ssrg$source44 = { file: "main.ssrg", line: 1, column: 45, uri: "seseragi://artifact/pure-call", start: 44, end: 56 };
const $ssrg$source91 = { file: "main.ssrg", line: 2, column: 35, uri: "seseragi://artifact/pure-call", start: 91, end: 102 };
const $ssrg$source136 = { file: "main.ssrg", line: 3, column: 34, uri: "seseragi://artifact/pure-call", start: 136, end: 145 };
$ssrg$assertUnicodeVersion("17.0.0")

export const add = (left: number) => (right: number) => _ssrg_int_add(left, right, $ssrg$source44)
export const addOne = (value: number) => $ssrg$atSource($ssrg$source91, () => add(value)(1), false)
export const total = (unit: undefined) => $ssrg$atSource($ssrg$source136, () => addOne(41), false)
