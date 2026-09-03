import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
$ssrg$assertUnicodeVersion("17.0.0")

export const announce = <A,>(value: A) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => _ssrg_console_println(__ssrg$evidence$0["show"](value))
