import { println as _ssrg_console_println } from "@seseragi/runtime/console"

export type Badge =
  | { readonly tag: "Active" }
  | { readonly tag: "Paused" };
export const Active: Badge = { tag: "Active" } as const;
export const Paused: Badge = { tag: "Paused" } as const;
export const __ssrg$instance$Render$0 = { "render": (value: Badge) => (($ssrg_match: Badge): string => $ssrg_match.tag === "Active" ? "active" : "paused")(value) } as const;
export const label = (value: Badge) => __ssrg$instance$Render$0["render"](value)
export const status = (unit: undefined) => "ready"
export const main = (_unit: undefined) => _ssrg_console_println(label(Active))
