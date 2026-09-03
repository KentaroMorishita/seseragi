import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { print as _ssrg_console_print, println as _ssrg_console_println } from "@seseragi/runtime/console"
$ssrg$assertUnicodeVersion("17.0.0")

export const announce = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_print("loading... "), () => _ssrg_effect_flatMap(_ssrg_console_println("done"), () => _ssrg_console_println("ready")))
