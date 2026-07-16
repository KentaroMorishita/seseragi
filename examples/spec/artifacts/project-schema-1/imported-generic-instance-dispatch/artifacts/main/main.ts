import { report, __ssrg$instance$Inspect$0 } from "./domain.js"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { Just as _ssrg_maybe_Just } from "@seseragi/runtime/sum"

export const main = (_unit: undefined) => _ssrg_console_println(report(_ssrg_maybe_Just(42n))(__ssrg$instance$Inspect$0<bigint>()))
