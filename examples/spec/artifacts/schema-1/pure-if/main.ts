import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
$ssrg$assertUnicodeVersion("17.0.0")

export const classify = (value: number) => _ssrg_int_eq_dictionary["eq"](value)(0) ? "zero" : "other"
