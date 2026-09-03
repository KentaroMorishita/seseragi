import { Nothing as _ssrg_maybe_Nothing, Just as _ssrg_maybe_Just } from "@seseragi/runtime/sum"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

export const __ssrg$instance$Functor$0 = { "map": <A, B,>(f: (argument: A) => B) => (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: A }) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: A }): { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: B } => $ssrg_match.tag === "Nothing" ? _ssrg_maybe_Nothing : $ssrg_match.tag === "Just" ? ((item: A): { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: B } => _ssrg_maybe_Just(f(item)))($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(value) } as const;
const transform = <F, A, B,>(f: (argument: A) => B) => (value: unknown) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => __ssrg$evidence$0["map"](f)(value)
const increment = (value: number) => _ssrg_int_add(value, 1)
const render = (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }): string => $ssrg_match.tag === "Nothing" ? "Nothing" : $ssrg_match.tag === "Just" && $ssrg_match.value === 42 ? "Just 42" : "Just another value")(value)
export const main = (_unit: undefined) => _ssrg_console_println(render(((transform(increment)(_ssrg_maybe_Just(41))(__ssrg$instance$Functor$0)) as { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number })))
