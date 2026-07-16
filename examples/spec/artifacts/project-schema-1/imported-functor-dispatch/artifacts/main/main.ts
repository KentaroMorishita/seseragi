import { incrementAll, __ssrg$instance$Functor$0 } from "./domain.js"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { Just as _ssrg_maybe_Just } from "@seseragi/runtime/sum"

const render = (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: bigint }) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: bigint }): string => $ssrg_match.tag === "Nothing" ? "Nothing" : $ssrg_match.tag === "Just" && $ssrg_match.value === 42n ? "Imported Functor: Just 42" : "Imported Functor: another value")(value)
export const main = (_unit: undefined) => _ssrg_console_println(render(incrementAll(_ssrg_maybe_Just(41n))(__ssrg$instance$Functor$0)))
