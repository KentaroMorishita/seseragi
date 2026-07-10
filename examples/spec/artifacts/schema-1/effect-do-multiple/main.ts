import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println("one"), () => _ssrg_console_println("two"))
