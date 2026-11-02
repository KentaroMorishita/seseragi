import { announce } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { stringShow as _ssrg_show_stringShow } from "@seseragi/runtime/show"
const $ssrg$source60 = { file: "project/imported-generic-effect/src/main.ssrg", line: 4, column: 3, uri: "seseragi://fixture/imported-generic-effect::main", start: 60, end: 85 };
$ssrg$assertUnicodeVersion("17.0.0")

export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source60, () => announce<string>("generic effect")(_ssrg_show_stringShow), true)
