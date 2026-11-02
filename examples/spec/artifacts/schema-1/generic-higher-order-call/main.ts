import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
const $ssrg$source52 = { file: "main.ssrg", line: 2, column: 3, uri: "seseragi://artifact/generic-higher-order-call", start: 52, end: 59 };
const $ssrg$source100 = { file: "main.ssrg", line: 5, column: 3, uri: "seseragi://artifact/generic-higher-order-call", start: 100, end: 109 };
const $ssrg$source148 = { file: "main.ssrg", line: 8, column: 3, uri: "seseragi://artifact/generic-higher-order-call", start: 148, end: 169 };
$ssrg$assertUnicodeVersion("17.0.0")

export const apply = <A, B,>(f: (argument: A) => B) => (value: A) => $ssrg$atSource($ssrg$source52, () => f(value), false)
export const increment = (value: number) => _ssrg_int_add(value, 1, $ssrg$source100)
export const example = (value: number) => $ssrg$atSource($ssrg$source148, () => apply<number, number>(increment)(value), false)
