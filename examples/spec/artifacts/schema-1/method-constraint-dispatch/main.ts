import { println as _ssrg_console_println } from "@seseragi/runtime/console"

export type Badge =
  | { readonly tag: "Active" };
export const Active: Badge = { tag: "Active" } as const;
export const __ssrg$instance$Labeled$0 = { "label": (value: Badge) => "active" } as const;
export const __ssrg$instance$Render$1 = { "render": (value: Badge) => (__ssrg$evidence$0: unknown) => __ssrg$evidence$0["label"](value) } as const;
export const main = (_unit: undefined) => _ssrg_console_println(__ssrg$instance$Render$1["render"](Active)(__ssrg$instance$Labeled$0))
