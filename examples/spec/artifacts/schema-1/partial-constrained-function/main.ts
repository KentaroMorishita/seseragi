import { println as _ssrg_console_println } from "@seseragi/runtime/console"

export type Badge =
  | { readonly tag: "Active" };
export const Active: Badge = { tag: "Active" } as const;
export const __ssrg$instance$Ready$0 = { "ready": (value: Badge) => "Badge is ready" } as const;
const describe = <T,>(value: T) => (suffix: string) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => __ssrg$evidence$0["ready"](value) + suffix
const applyLabel = (labeler: (argument: string) => string) => labeler("!")
export const main = (_unit: undefined) => _ssrg_console_println(applyLabel((__ssrg$partial$0: string) => describe(Active)(__ssrg$partial$0)(__ssrg$instance$Ready$0)))
