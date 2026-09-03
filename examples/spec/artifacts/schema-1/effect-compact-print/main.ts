import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { print as _ssrg_console_print } from "@seseragi/runtime/console"
$ssrg$assertUnicodeVersion("17.0.0")

export const announce = (_unit: undefined) => _ssrg_console_print("loading")
