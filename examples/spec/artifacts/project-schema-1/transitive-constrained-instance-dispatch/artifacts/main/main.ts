import { report } from "./facade.js"
import { __ssrg$instance$Inspect$1, __ssrg$instance$Ready$0 } from "./provider.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { Just as _ssrg_maybe_Just } from "@seseragi/runtime/sum"
$ssrg$assertUnicodeVersion("17.0.0")

export const main = (_unit: undefined) => _ssrg_console_println(((report(_ssrg_maybe_Just(42))(__ssrg$instance$Inspect$1<number>(__ssrg$instance$Ready$0))) as string))
