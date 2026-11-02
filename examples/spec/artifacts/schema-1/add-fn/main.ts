import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
const $ssrg$source37 = { file: "main.ssrg", line: 1, column: 38, uri: "seseragi://artifact/add-fn", start: 37, end: 42 };
$ssrg$assertUnicodeVersion("17.0.0")

export const add = (x: number) => (y: number) => _ssrg_int_add(x, y, $ssrg$source37)
