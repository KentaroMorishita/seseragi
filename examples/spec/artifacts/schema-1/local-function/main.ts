import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
import { add as _ssrg_int_add, subtract as _ssrg_int_subtract } from "@seseragi/runtime/int"
const $ssrg$source156 = { file: "main.ssrg", line: 7, column: 8, uri: "seseragi://artifact/local-function", start: 156, end: 168 };
const $ssrg$source180 = { file: "main.ssrg", line: 8, column: 7, uri: "seseragi://artifact/local-function", start: 180, end: 202 };
const $ssrg$source190 = { file: "main.ssrg", line: 8, column: 17, uri: "seseragi://artifact/local-function", start: 190, end: 201 };
const $ssrg$source218 = { file: "main.ssrg", line: 10, column: 7, uri: "seseragi://artifact/local-function", start: 218, end: 241 };
const $ssrg$source229 = { file: "main.ssrg", line: 10, column: 18, uri: "seseragi://artifact/local-function", start: 229, end: 240 };
const $ssrg$source245 = { file: "main.ssrg", line: 12, column: 3, uri: "seseragi://artifact/local-function", start: 245, end: 256 };
$ssrg$assertUnicodeVersion("17.0.0")

export const localFunctionWorks = (unit: undefined) => (() => { const offset: number = 2; return (() => { const identity = <A,>(value: A) => value; return (() => { const countdown = (current: number) => { const $ssrg$tail = Symbol(); let $ssrg$loopArguments: readonly [number] = [current]; while (true) { const current: number = $ssrg$loopArguments[0]; const $ssrg$result = $ssrg$atSource($ssrg$source156, () => _ssrg_int_eq_dictionary["eq"](current)(0), false) ? $ssrg$atSource($ssrg$source180, () => identity<number>(_ssrg_int_add(40, offset, $ssrg$source190)), false) : $ssrg$atSource($ssrg$source218, () => ({ [$ssrg$tail]: [_ssrg_int_subtract(current, 1, $ssrg$source229)] } as never), false); const $ssrg$tailResult: unknown = $ssrg$result; if (typeof $ssrg$tailResult === "object" && $ssrg$tailResult !== null && $ssrg$tail in $ssrg$tailResult) { const $ssrg$arguments = ($ssrg$tailResult as { readonly [$ssrg$tail]: readonly [number] })[$ssrg$tail]; $ssrg$loopArguments = $ssrg$arguments; continue; } return $ssrg$result; } }; return $ssrg$atSource($ssrg$source245, () => _ssrg_int_eq_dictionary["eq"]($ssrg$atSource($ssrg$source245, () => countdown(3), false))(42), false); })(); })(); })()
