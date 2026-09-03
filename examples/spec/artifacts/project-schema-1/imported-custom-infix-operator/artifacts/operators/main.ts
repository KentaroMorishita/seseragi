import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { subtract as _ssrg_int_subtract } from "@seseragi/runtime/int"
$ssrg$assertUnicodeVersion("17.0.0")

export const __ssrg$operator$3c5e3e = (left: number) => (right: number) => _ssrg_int_subtract(left, right)
