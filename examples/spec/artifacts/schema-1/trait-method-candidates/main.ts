import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

export type Badge =
  | { readonly tag: "Active" }
  | { readonly tag: "Paused" };
export const Active: Badge = { tag: "Active" } as const;
export const Paused: Badge = { tag: "Paused" } as const;
export type Mode =
  | { readonly tag: "Automatic" }
  | { readonly tag: "Manual" };
export const Automatic: Mode = { tag: "Automatic" } as const;
export const Manual: Mode = { tag: "Manual" } as const;
export const __ssrg$instance$Render$0 = { "present": (value: Badge) => (($ssrg_match: Badge): string => $ssrg_match.tag === "Active" ? "Status badge: active" : "Status badge: paused")(value) } as const;
export const __ssrg$instance$Describe$1 = { "present": (value: Mode) => (($ssrg_match: Mode): string => $ssrg_match.tag === "Automatic" ? "Mode badge: automatic" : "Mode badge: manual")(value) } as const;
export const badgeLabel = (value: Badge) => __ssrg$instance$Render$0["present"](value)
export const modeLabel = (value: Mode) => __ssrg$instance$Describe$1["present"](value)
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println(badgeLabel(Active)), () => _ssrg_console_println(modeLabel(Automatic)))
