import { __ssrg$operator$3c5e3e } from "./operators.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
$ssrg$assertUnicodeVersion("17.0.0")

const calculate = (unit: undefined) => __ssrg$operator$3c5e3e(10)(__ssrg$operator$3c5e3e(3)(2))
export const main = (_unit: undefined) => _ssrg_console_println("Imported custom infix: " + _ssrg_show_intShow["show"](calculate(undefined)))
