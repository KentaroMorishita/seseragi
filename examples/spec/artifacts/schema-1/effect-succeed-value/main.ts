import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { succeed as _ssrg_effect_succeed } from "@seseragi/runtime/effect"
const $ssrg$source22 = { file: "main.ssrg", line: 1, column: 23, uri: "seseragi://artifact/effect-succeed-value", start: 22, end: 37 };
$ssrg$assertUnicodeVersion("17.0.0")

export const ready = (_unit: undefined) => $ssrg$atSource($ssrg$source22, () => _ssrg_effect_succeed("ready"), true)
