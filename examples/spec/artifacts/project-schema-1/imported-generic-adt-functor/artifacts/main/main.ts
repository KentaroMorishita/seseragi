import { type Box, Boxed, transform, __ssrg$instance$Functor$0 } from "./domain.js"
import { add as _ssrg_int64_add } from "@seseragi/runtime/int64"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

const increment = (value: bigint) => _ssrg_int64_add(value, 1n)
const render = (value: Box<bigint>) => (($ssrg_match: Box<bigint>): string => $ssrg_match.tag === "Boxed" && $ssrg_match.value === 42n ? "Imported Box Functor: 42" : "Imported Box Functor: another value")(value)
export const main = (_unit: undefined) => _ssrg_console_println(render(transform(increment)(Boxed(41n))(__ssrg$instance$Functor$0)))
