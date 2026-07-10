import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { print as _ssrg_console_print, println as _ssrg_console_println } from "@seseragi/runtime/console"

export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_print("loading... "), (ignored: undefined) => _ssrg_console_println("done"))
