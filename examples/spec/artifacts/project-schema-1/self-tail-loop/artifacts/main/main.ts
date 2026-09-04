import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { intOrd as _ssrg_int_ord_dictionary, intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
import { subtract as _ssrg_int_subtract, add as _ssrg_int_add } from "@seseragi/runtime/int"
$ssrg$assertUnicodeVersion("17.0.0")

const sumTo = (current: number) => (total: number) => { const $ssrg$tail = Symbol(); while (true) { const $ssrg$result = (_ssrg_int_ord_dictionary["compare"](current)(0))["tag"] !== "Greater" ? total : ({ [$ssrg$tail]: [_ssrg_int_subtract(current, 1), _ssrg_int_add(total, current)] } as never); const $ssrg$tailResult: unknown = $ssrg$result; if (typeof $ssrg$tailResult === "object" && $ssrg$tailResult !== null && $ssrg$tail in $ssrg$tailResult) { const $ssrg$arguments = ($ssrg$tailResult as { readonly [$ssrg$tail]: readonly [number, number] })[$ssrg$tail]; current = $ssrg$arguments[0]; total = $ssrg$arguments[1]; continue; } return $ssrg$result; } }
export const selfTailLoopWorks = (unit: undefined) => _ssrg_int_eq_dictionary["eq"](sumTo(100000)(0))(5000050000)
