import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { intOrd as _ssrg_int_ord_dictionary, intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
import { subtract as _ssrg_int_subtract, add as _ssrg_int_add } from "@seseragi/runtime/int"
$ssrg$assertUnicodeVersion("17.0.0")

const sumTo: (argument: number) => (argument: number) => number = (current: number) => (total: number) => { let $ssrg$scalarSlot0: number = current;let $ssrg$scalarSlot1: number = total; while (true) { const current: number = $ssrg$scalarSlot0;const total: number = $ssrg$scalarSlot1; if ((_ssrg_int_ord_dictionary["compare"](current)(0))["tag"] !== "Greater") { return total; } else { const $ssrg$scalarNext0 = _ssrg_int_subtract(current, 1);const $ssrg$scalarNext1 = _ssrg_int_add(total, current);$ssrg$scalarSlot0 = $ssrg$scalarNext0;$ssrg$scalarSlot1 = $ssrg$scalarNext1;continue; } } }
export const selfTailLoopWorks = (unit: undefined) => _ssrg_int_eq_dictionary["eq"](sumTo(100000)(0))(5000050000)
