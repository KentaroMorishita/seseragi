import { __ssrg$operator$3c5e3e } from "./operators.js"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"

const calculate = (unit: undefined) => __ssrg$operator$3c5e3e(10n)(__ssrg$operator$3c5e3e(3n)(2n))
export const main = (_unit: undefined) => _ssrg_console_println("Imported custom infix: " + _ssrg_show_intShow["show"](calculate(undefined)))
