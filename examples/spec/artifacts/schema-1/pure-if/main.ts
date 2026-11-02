import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
const $ssrg$source42 = { file: "main.ssrg", line: 1, column: 43, uri: "seseragi://artifact/pure-if", start: 42, end: 52 };
$ssrg$assertUnicodeVersion("17.0.0")

export const classify = (value: number) => $ssrg$atSource($ssrg$source42, () => _ssrg_int_eq_dictionary["eq"](value)(0), false) ? "zero" : "other"
