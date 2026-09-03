import { announce } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { stringShow as _ssrg_show_stringShow } from "@seseragi/runtime/show"
$ssrg$assertUnicodeVersion("17.0.0")

export const main = (_unit: undefined) => announce("generic effect")(_ssrg_show_stringShow)
