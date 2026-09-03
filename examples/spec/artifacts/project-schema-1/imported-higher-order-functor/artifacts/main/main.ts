import { transform, __ssrg$instance$Functor$0 } from "./domain.js"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { Just as _ssrg_maybe_Just } from "@seseragi/runtime/sum"

const increment = (value: number) => _ssrg_int_add(value, 1)
const render = (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }): string => $ssrg_match.tag === "Nothing" ? "Nothing" : $ssrg_match.tag === "Just" && $ssrg_match.value === 42 ? "Imported mapper: Just 42" : "Imported mapper: another value")(value)
export const main = (_unit: undefined) => _ssrg_console_println(render(((transform(increment)(_ssrg_maybe_Just(41))(__ssrg$instance$Functor$0)) as { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number })))
