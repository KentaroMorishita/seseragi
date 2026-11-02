import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source83 = { file: "project/imported-effect-console/src/effects.ssrg", line: 4, column: 3, uri: "seseragi://fixture/imported-effect-console::effects", start: 83, end: 98 };
$ssrg$assertUnicodeVersion("17.0.0")

export const announce = (message: string) => $ssrg$atSource($ssrg$source83, () => _ssrg_console_println(message), true)
