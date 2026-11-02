import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source68 = { file: "main.ssrg", line: 4, column: 8, uri: "seseragi://artifact/effect-do-multiple", start: 68, end: 81 };
const $ssrg$source83 = { file: "main.ssrg", line: 4, column: 23, uri: "seseragi://artifact/effect-do-multiple", start: 83, end: 96 };
$ssrg$assertUnicodeVersion("17.0.0")

export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source68, () => _ssrg_console_println("one"), true), () => $ssrg$atSource($ssrg$source83, () => _ssrg_console_println("two"), true))
