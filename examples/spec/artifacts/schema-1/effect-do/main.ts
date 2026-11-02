import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { succeed as _ssrg_effect_succeed } from "@seseragi/runtime/effect"
const $ssrg$source36 = { file: "main.ssrg", line: 2, column: 8, uri: "seseragi://artifact/effect-do", start: 36, end: 46 };
$ssrg$assertUnicodeVersion("17.0.0")

export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source36, () => _ssrg_effect_succeed(undefined), true)
