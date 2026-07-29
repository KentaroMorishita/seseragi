import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { Just as _ssrg_maybe_Just } from "@seseragi/runtime/sum"

export const __ssrg$instance$Tag$0 = <T,>() => ({ "tag": (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: T }) => "Generic dictionary: maybe" }) as const;
export const label = (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }) => __ssrg$instance$Tag$0<number>()["tag"](value)
export const main = (_unit: undefined) => _ssrg_console_println(label(_ssrg_maybe_Just(1)))
