import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { print as _ssrg_console_print } from "@seseragi/runtime/console"
const $ssrg$source27 = { file: "main.ssrg", line: 2, column: 3, uri: "seseragi://artifact/effect-compact-print", start: 27, end: 42 };
$ssrg$assertUnicodeVersion("17.0.0")

export const announce = (_unit: undefined) => $ssrg$atSource($ssrg$source27, () => _ssrg_console_print("loading"), true)
