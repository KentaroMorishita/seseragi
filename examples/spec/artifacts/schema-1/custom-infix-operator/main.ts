import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { subtract as _ssrg_int_subtract } from "@seseragi/runtime/int"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
const $ssrg$source124 = { file: "main.ssrg", line: 2, column: 40, uri: "seseragi://artifact/custom-infix-operator", start: 124, end: 138 };
const $ssrg$source131 = { file: "main.ssrg", line: 2, column: 47, uri: "seseragi://artifact/custom-infix-operator", start: 131, end: 138 };
const $ssrg$source177 = { file: "main.ssrg", line: 3, column: 39, uri: "seseragi://artifact/custom-infix-operator", start: 177, end: 185 };
const $ssrg$source252 = { file: "main.ssrg", line: 7, column: 3, uri: "seseragi://artifact/custom-infix-operator", start: 252, end: 264 };
const $ssrg$source325 = { file: "main.ssrg", line: 11, column: 3, uri: "seseragi://artifact/custom-infix-operator", start: 325, end: 337 };
const $ssrg$source402 = { file: "main.ssrg", line: 16, column: 3, uri: "seseragi://artifact/custom-infix-operator", start: 402, end: 480 };
const $ssrg$source433 = { file: "main.ssrg", line: 16, column: 34, uri: "seseragi://artifact/custom-infix-operator", start: 433, end: 451 };
const $ssrg$source461 = { file: "main.ssrg", line: 16, column: 62, uri: "seseragi://artifact/custom-infix-operator", start: 461, end: 478 };
$ssrg$assertUnicodeVersion("17.0.0")

const rightAssociated = (unit: undefined) => $ssrg$atSource($ssrg$source124, () => __ssrg$operator$3c5e3e(10)($ssrg$atSource($ssrg$source131, () => __ssrg$operator$3c5e3e(3)(2), false)), false)
const leftAssociated = (unit: undefined) => $ssrg$atSource($ssrg$source177, () => __ssrg$operator$3c7e3e($ssrg$atSource($ssrg$source177, () => __ssrg$operator$3c7e3e(10)(3), false))(2), false)
const __ssrg$operator$3c5e3e = (left: number) => (right: number) => _ssrg_int_subtract(left, right, $ssrg$source252)
const __ssrg$operator$3c7e3e = (left: number) => (right: number) => _ssrg_int_subtract(left, right, $ssrg$source325)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source402, () => _ssrg_console_println("Custom infix: right=" + _ssrg_show_intShow["show"]($ssrg$atSource($ssrg$source433, () => rightAssociated(undefined), false)) + ", left=" + _ssrg_show_intShow["show"]($ssrg$atSource($ssrg$source461, () => leftAssociated(undefined), false))), true)
