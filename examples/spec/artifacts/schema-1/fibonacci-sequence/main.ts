import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
import { subtract as _ssrg_int_subtract, add as _ssrg_int_add } from "@seseragi/runtime/int"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { collectMap as _ssrg_range_comprehend, inclusive as _ssrg_range_inclusive } from "@seseragi/runtime/range"
$ssrg$assertUnicodeVersion("17.0.0")

const fibonacci = (index: number) => (() => { const loop = (remaining: number) => (current: number) => (next: number) => { const $ssrg$tail = Symbol(); let $ssrg$loopArguments: readonly [number, number, number] = [remaining, current, next]; while (true) { const remaining: number = $ssrg$loopArguments[0]; const current: number = $ssrg$loopArguments[1]; const next: number = $ssrg$loopArguments[2]; const $ssrg$result = _ssrg_int_eq_dictionary["eq"](remaining)(0) ? current : ({ [$ssrg$tail]: [_ssrg_int_subtract(remaining, 1), next, _ssrg_int_add(current, next)] } as never); const $ssrg$tailResult: unknown = $ssrg$result; if (typeof $ssrg$tailResult === "object" && $ssrg$tailResult !== null && $ssrg$tail in $ssrg$tailResult) { const $ssrg$arguments = ($ssrg$tailResult as { readonly [$ssrg$tail]: readonly [number, number, number] })[$ssrg$tail]; $ssrg$loopArguments = $ssrg$arguments; continue; } return $ssrg$result; } }; return loop(index)(0)(1); })()
export const fibonacciLines = (unit: undefined) => _ssrg_range_comprehend(_ssrg_range_inclusive(0, 10), (index) => true, (index) => _ssrg_show_intShow["show"](index) + ": " + _ssrg_show_intShow["show"](fibonacci(index)))
