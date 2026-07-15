import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { Just as _ssrg_maybe_Just } from "@seseragi/runtime/sum"

export type Badge =
  | { readonly tag: "Active" };
export const Active: Badge = { tag: "Active" } as const;
export const __ssrg$instance$Ready$0 = { "ready": (value: Badge) => "Constrained dictionary: active" } as const;
export const __ssrg$instance$Render$1 = <T,>(_evidence0: unknown) => ({ "render": (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: T }) => "Constrained dictionary: ready" }) as const;
export const label = (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: Badge }) => __ssrg$instance$Render$1<Badge>(__ssrg$instance$Ready$0)["render"](value)
export const main = (_unit: undefined) => _ssrg_console_println(label(_ssrg_maybe_Just(Active)))
