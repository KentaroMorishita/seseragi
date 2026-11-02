import { __ssrg$operator$3c5e3e } from "./operators.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
const $ssrg$source79 = { file: "project/imported-custom-infix-operator/src/main.ssrg", line: 4, column: 3, uri: "seseragi://fixture/imported-custom-infix-operator::main", start: 79, end: 93 };
const $ssrg$source86 = { file: "project/imported-custom-infix-operator/src/main.ssrg", line: 4, column: 10, uri: "seseragi://fixture/imported-custom-infix-operator::main", start: 86, end: 93 };
const $ssrg$source158 = { file: "project/imported-custom-infix-operator/src/main.ssrg", line: 9, column: 3, uri: "seseragi://fixture/imported-custom-infix-operator::main", start: 158, end: 206 };
const $ssrg$source192 = { file: "project/imported-custom-infix-operator/src/main.ssrg", line: 9, column: 37, uri: "seseragi://fixture/imported-custom-infix-operator::main", start: 192, end: 204 };
$ssrg$assertUnicodeVersion("17.0.0")

const calculate = (unit: undefined) => $ssrg$atSource($ssrg$source79, () => __ssrg$operator$3c5e3e(10)($ssrg$atSource($ssrg$source86, () => __ssrg$operator$3c5e3e(3)(2), false)), false)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source158, () => _ssrg_console_println("Imported custom infix: " + _ssrg_show_intShow["show"]($ssrg$atSource($ssrg$source192, () => calculate(undefined), false))), true)
