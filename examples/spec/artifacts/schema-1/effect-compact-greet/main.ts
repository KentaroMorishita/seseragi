import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source33 = { file: "main.ssrg", line: 2, column: 3, uri: "seseragi://artifact/effect-compact-greet", start: 33, end: 48 };
$ssrg$assertUnicodeVersion("17.0.0")

const greet = (name: string) => $ssrg$atSource($ssrg$source33, () => _ssrg_console_println("hello"), true)
