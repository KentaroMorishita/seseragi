import { println as _ssrg_console_println } from "@seseragi/runtime/console"

export type Badge =
  | { readonly tag: "Active" };
export const Active: Badge = { tag: "Active" } as const;
export const __ssrg$instance$Ready$0 = { "ready": (value: Badge) => "Badge is ready" } as const;
export const describe = <T,>(value: T) => (__ssrg$evidence$0: unknown) => __ssrg$evidence$0["ready"](value)
export const main = (_unit: undefined) => _ssrg_console_println(describe(Active)(__ssrg$instance$Ready$0))
