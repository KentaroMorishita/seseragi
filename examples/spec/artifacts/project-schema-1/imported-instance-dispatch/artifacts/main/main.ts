import { Active, describe, __ssrg$instance$Ready$0 } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
$ssrg$assertUnicodeVersion("17.0.0")

export const main = (_unit: undefined) => _ssrg_console_println(((describe(Active)(__ssrg$instance$Ready$0)) as string))
