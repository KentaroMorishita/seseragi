import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { print as _ssrg_console_print, println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source83 = { file: "main.ssrg", line: 5, column: 16, uri: "seseragi://artifact/effect-do-bind", start: 83, end: 102 };
const $ssrg$source107 = { file: "main.ssrg", line: 6, column: 5, uri: "seseragi://artifact/effect-do-bind", start: 107, end: 121 };
$ssrg$assertUnicodeVersion("17.0.0")

export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source83, () => _ssrg_console_print("loading... "), true), (ignored: undefined) => $ssrg$atSource($ssrg$source107, () => _ssrg_console_println("done"), true))
