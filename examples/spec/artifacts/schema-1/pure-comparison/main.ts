import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { intEq as _ssrg_int_eq_dictionary, boolEq as _ssrg_bool_eq_dictionary, stringEq as _ssrg_string_eq_dictionary } from "@seseragi/runtime/equality"
const $ssrg$source35 = { file: "main.ssrg", line: 1, column: 36, uri: "seseragi://artifact/pure-comparison", start: 35, end: 45 };
const $ssrg$source98 = { file: "main.ssrg", line: 2, column: 53, uri: "seseragi://artifact/pure-comparison", start: 98, end: 111 };
$ssrg$assertUnicodeVersion("17.0.0")

export const isZero = (value: number) => $ssrg$atSource($ssrg$source35, () => _ssrg_int_eq_dictionary["eq"](value)(0), false)
export const sameBool = (left: boolean) => (right: boolean) => $ssrg$atSource($ssrg$source98, () => _ssrg_bool_eq_dictionary["eq"](left)(right), false)
export const sameString = (left: string) => (right: string) => _ssrg_string_eq_dictionary["eq"](left)(right) === false
