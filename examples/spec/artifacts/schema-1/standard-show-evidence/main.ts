import { stringShow as _ssrg_show_stringShow } from "@seseragi/runtime/show"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { Just as _ssrg_maybe_Just } from "@seseragi/runtime/sum"

export const __ssrg$instance$Render$0 = <T,>(__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => ({ "render": (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: T }) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: T }): string => $ssrg_match.tag === "Nothing" ? "No value" : $ssrg_match.tag === "Just" ? ((item: T): string => acknowledge(item)(__ssrg$evidence$0))($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(value) }) as const;
const acknowledge = <T,>(value: T) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => "Standard Show evidence"
export const label = (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string }) => __ssrg$instance$Render$0<string>(_ssrg_show_stringShow)["render"](value)
export const main = (_unit: undefined) => _ssrg_console_println(label(_ssrg_maybe_Just("ready")))
