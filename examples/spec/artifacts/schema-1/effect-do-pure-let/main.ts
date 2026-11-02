import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source98 = { file: "main.ssrg", line: 6, column: 5, uri: "seseragi://artifact/effect-do-pure-let", start: 98, end: 113 };
$ssrg$assertUnicodeVersion("17.0.0")

export const main = (_unit: undefined) => (() => { const message: string = "hello"; return $ssrg$atSource($ssrg$source98, () => _ssrg_console_println(message), true); })()
