import { subtract as _ssrg_int64_subtract } from "@seseragi/runtime/int64"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"

const rightAssociated = (unit: undefined) => __ssrg$operator$3c5e3e(10n)(__ssrg$operator$3c5e3e(3n)(2n))
const leftAssociated = (unit: undefined) => __ssrg$operator$3c7e3e(__ssrg$operator$3c7e3e(10n)(3n))(2n)
const __ssrg$operator$3c5e3e = (left: bigint) => (right: bigint) => _ssrg_int64_subtract(left, right)
const __ssrg$operator$3c7e3e = (left: bigint) => (right: bigint) => _ssrg_int64_subtract(left, right)
export const main = (_unit: undefined) => _ssrg_console_println("Custom infix: right=" + _ssrg_show_intShow["show"](rightAssociated(undefined)) + ", left=" + _ssrg_show_intShow["show"](leftAssociated(undefined)))
