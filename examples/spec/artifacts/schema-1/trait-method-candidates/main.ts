import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source579 = { file: "main.ssrg", line: 33, column: 44, uri: "seseragi://artifact/trait-method-candidates", start: 579, end: 592 };
const $ssrg$source635 = { file: "main.ssrg", line: 35, column: 42, uri: "seseragi://artifact/trait-method-candidates", start: 635, end: 648 };
const $ssrg$source722 = { file: "main.ssrg", line: 41, column: 5, uri: "seseragi://artifact/trait-method-candidates", start: 722, end: 749 };
const $ssrg$source731 = { file: "main.ssrg", line: 41, column: 14, uri: "seseragi://artifact/trait-method-candidates", start: 731, end: 748 };
const $ssrg$source754 = { file: "main.ssrg", line: 42, column: 5, uri: "seseragi://artifact/trait-method-candidates", start: 754, end: 783 };
const $ssrg$source763 = { file: "main.ssrg", line: 42, column: 14, uri: "seseragi://artifact/trait-method-candidates", start: 763, end: 782 };
$ssrg$assertUnicodeVersion("17.0.0")

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
export const badgeLabel = (value: Badge) => $ssrg$atSource($ssrg$source579, () => ((__ssrg$instance$Render$0["present"](value)) as string), false)
export const modeLabel = (value: Mode) => $ssrg$atSource($ssrg$source635, () => ((__ssrg$instance$Describe$1["present"](value)) as string), false)
export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source722, () => _ssrg_console_println($ssrg$atSource($ssrg$source731, () => badgeLabel(Active), false)), true), () => $ssrg$atSource($ssrg$source754, () => _ssrg_console_println($ssrg$atSource($ssrg$source763, () => modeLabel(Automatic), false)), true))
