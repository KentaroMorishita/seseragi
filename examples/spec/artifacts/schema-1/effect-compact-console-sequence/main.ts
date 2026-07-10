import { print as _ssrg_console_print, println as _ssrg_console_println } from "@seseragi/runtime/console"

export const announce = (_unit: undefined) => (() => { _ssrg_console_print("loading... "); _ssrg_console_println("done"); return _ssrg_console_println("ready"); })()
