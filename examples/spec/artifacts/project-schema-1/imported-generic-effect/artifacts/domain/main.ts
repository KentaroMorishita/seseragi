import { println as _ssrg_console_println } from "@seseragi/runtime/console"

export const announce = <A,>(value: A) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => _ssrg_console_println(__ssrg$evidence$0["show"](value))
