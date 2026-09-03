import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { subtract as _ssrg_int_subtract } from "@seseragi/runtime/int"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
$ssrg$assertUnicodeVersion("17.0.0")

const rightAssociated = (unit: undefined) => __ssrg$operator$3c5e3e(10)(__ssrg$operator$3c5e3e(3)(2))
const leftAssociated = (unit: undefined) => __ssrg$operator$3c7e3e(__ssrg$operator$3c7e3e(10)(3))(2)
const __ssrg$operator$3c5e3e = (left: number) => (right: number) => _ssrg_int_subtract(left, right)
const __ssrg$operator$3c7e3e = (left: number) => (right: number) => _ssrg_int_subtract(left, right)
export const main = (_unit: undefined) => _ssrg_console_println("Custom infix: right=" + _ssrg_show_intShow["show"](rightAssociated(undefined)) + ", left=" + _ssrg_show_intShow["show"](leftAssociated(undefined)))
