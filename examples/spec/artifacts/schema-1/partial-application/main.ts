import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
const $ssrg$source44 = { file: "main.ssrg", line: 1, column: 45, uri: "seseragi://artifact/partial-application", start: 44, end: 56 };
const $ssrg$source99 = { file: "main.ssrg", line: 2, column: 43, uri: "seseragi://artifact/partial-application", start: 99, end: 108 };
$ssrg$assertUnicodeVersion("17.0.0")

export const add = (left: number) => (right: number) => _ssrg_int_add(left, right, $ssrg$source44)
export const addTo = (value: number) => $ssrg$atSource($ssrg$source99, () => add(value), false)
