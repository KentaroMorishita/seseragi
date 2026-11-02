import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { print as _ssrg_console_print, println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source32 = { file: "main.ssrg", line: 2, column: 8, uri: "seseragi://artifact/effect-compact-console-sequence", start: 32, end: 51 };
const $ssrg$source53 = { file: "main.ssrg", line: 2, column: 29, uri: "seseragi://artifact/effect-compact-console-sequence", start: 53, end: 67 };
const $ssrg$source69 = { file: "main.ssrg", line: 2, column: 45, uri: "seseragi://artifact/effect-compact-console-sequence", start: 69, end: 84 };
$ssrg$assertUnicodeVersion("17.0.0")

export const announce = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source32, () => _ssrg_console_print("loading... "), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source53, () => _ssrg_console_println("done"), true), () => $ssrg$atSource($ssrg$source69, () => _ssrg_console_println("ready"), true)))
