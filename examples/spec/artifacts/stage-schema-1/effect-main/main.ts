import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source63 = { file: "main.ssrg", line: 4, column: 3, uri: "seseragi://artifact/effect-main", start: 63, end: 78 };
$ssrg$assertUnicodeVersion("17.0.0")

export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source63, () => _ssrg_console_println("hello"), true)
