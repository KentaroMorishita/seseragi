import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { Just as _ssrg_maybe_Just } from "@seseragi/runtime/sum"

export type Badge =
  | { readonly tag: "Active" };
export const Active: Badge = { tag: "Active" } as const;
export const __ssrg$instance$Ready$0 = { "ready": (value: Badge) => "Constrained dictionary: ready" } as const;
export const __ssrg$instance$Render$1 = <T,>(__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => ({ "render": (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: T }) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: T }): string => $ssrg_match.tag === "Nothing" ? "Constrained dictionary: empty" : $ssrg_match.tag === "Just" ? ((item: T): string => __ssrg$evidence$0["ready"](item))($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(value) }) as const;
export const label = (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: Badge }) => __ssrg$instance$Render$1<Badge>(__ssrg$instance$Ready$0)["render"](value)
export const main = (_unit: undefined) => _ssrg_console_println(label(_ssrg_maybe_Just(Active)))
