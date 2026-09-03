import { active, describe } from "./facade.js"
import { __ssrg$instance$Ready$0, type Badge } from "./provider.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
$ssrg$assertUnicodeVersion("17.0.0")

export const main = (_unit: undefined) => _ssrg_console_println(((describe(active(undefined))(__ssrg$instance$Ready$0)) as string))
