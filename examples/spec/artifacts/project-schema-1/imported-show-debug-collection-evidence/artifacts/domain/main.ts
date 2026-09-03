import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
$ssrg$assertUnicodeVersion("17.0.0")

export type Badge =
  | { readonly tag: "Active" }
  | { readonly tag: "Paused" };
export const Active: Badge = { tag: "Active" } as const;
export const Paused: Badge = { tag: "Paused" } as const;
export const __ssrg$instance$Show$0 = { "show": (value: Badge) => (($ssrg_match: Badge): string => $ssrg_match.tag === "Active" ? "active" : "paused")(value) } as const;
export const __ssrg$instance$Debug$1 = { "debug": (value: Badge) => (($ssrg_match: Badge): string => $ssrg_match.tag === "Active" ? "Badge.Active" : "Badge.Paused")(value) } as const;
export const __ssrg$instance$Render$2 = <T,>(__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => ({ "render": (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: T }) => "imported collection requirement" }) as const;
export const renderImported = <T,>(value: T) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => __ssrg$evidence$0["render"](value)
