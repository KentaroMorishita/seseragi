import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
import { subtract as _ssrg_int_subtract, add as _ssrg_int_add } from "@seseragi/runtime/int"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { collectMap as _ssrg_range_comprehend, inclusive as _ssrg_range_inclusive } from "@seseragi/runtime/range"
const $ssrg$source105 = { file: "main.ssrg", line: 3, column: 8, uri: "seseragi://artifact/fibonacci-sequence", start: 105, end: 119 };
const $ssrg$source154 = { file: "main.ssrg", line: 6, column: 7, uri: "seseragi://artifact/fibonacci-sequence", start: 154, end: 196 };
const $ssrg$source160 = { file: "main.ssrg", line: 6, column: 13, uri: "seseragi://artifact/fibonacci-sequence", start: 160, end: 173 };
const $ssrg$source181 = { file: "main.ssrg", line: 6, column: 34, uri: "seseragi://artifact/fibonacci-sequence", start: 181, end: 195 };
const $ssrg$source200 = { file: "main.ssrg", line: 8, column: 3, uri: "seseragi://artifact/fibonacci-sequence", start: 200, end: 214 };
const $ssrg$source286 = { file: "main.ssrg", line: 12, column: 17, uri: "seseragi://artifact/fibonacci-sequence", start: 286, end: 301 };
const $ssrg$source315 = { file: "main.ssrg", line: 12, column: 46, uri: "seseragi://artifact/fibonacci-sequence", start: 315, end: 321 };
$ssrg$assertUnicodeVersion("17.0.0")

const fibonacci = (index: number) => (() => { const loop = (remaining: number) => (current: number) => (next: number) => { const $ssrg$tail = Symbol(); let $ssrg$loopArguments: readonly [number, number, number] = [remaining, current, next]; while (true) { const remaining: number = $ssrg$loopArguments[0]; const current: number = $ssrg$loopArguments[1]; const next: number = $ssrg$loopArguments[2]; const $ssrg$result = $ssrg$atSource($ssrg$source105, () => _ssrg_int_eq_dictionary["eq"](remaining)(0), false) ? current : $ssrg$atSource($ssrg$source154, () => ({ [$ssrg$tail]: [_ssrg_int_subtract(remaining, 1, $ssrg$source160), next, _ssrg_int_add(current, next, $ssrg$source181)] } as never), false); const $ssrg$tailResult: unknown = $ssrg$result; if (typeof $ssrg$tailResult === "object" && $ssrg$tailResult !== null && $ssrg$tail in $ssrg$tailResult) { const $ssrg$arguments = ($ssrg$tailResult as { readonly [$ssrg$tail]: readonly [number, number, number] })[$ssrg$tail]; $ssrg$loopArguments = $ssrg$arguments; continue; } return $ssrg$result; } }; return $ssrg$atSource($ssrg$source200, () => loop(index)(0)(1), false); })()
export const fibonacciLines = (unit: undefined) => _ssrg_range_comprehend($ssrg$atSource($ssrg$source315, () => _ssrg_range_inclusive(0, 10), false), (index) => true, (index) => _ssrg_show_intShow["show"](index) + ": " + _ssrg_show_intShow["show"]($ssrg$atSource($ssrg$source286, () => fibonacci(index), false)))
