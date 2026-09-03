import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
$ssrg$assertUnicodeVersion("17.0.0")

export const add = (left: number) => (right: number) => _ssrg_int_add(left, right)
export const addTo = (value: number) => add(value)
