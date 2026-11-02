import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source273 = { file: "main.ssrg", line: 17, column: 39, uri: "seseragi://artifact/user-instance-dictionary", start: 273, end: 285 };
const $ssrg$source396 = { file: "main.ssrg", line: 24, column: 3, uri: "seseragi://artifact/user-instance-dictionary", start: 396, end: 418 };
const $ssrg$source405 = { file: "main.ssrg", line: 24, column: 12, uri: "seseragi://artifact/user-instance-dictionary", start: 405, end: 417 };
$ssrg$assertUnicodeVersion("17.0.0")

export type Badge =
  | { readonly tag: "Active" }
  | { readonly tag: "Paused" };
export const Active: Badge = { tag: "Active" } as const;
export const Paused: Badge = { tag: "Paused" } as const;
export const __ssrg$instance$Render$0 = { "render": (value: Badge) => (($ssrg_match: Badge): string => $ssrg_match.tag === "Active" ? "active" : "paused")(value) } as const;
export const label = (value: Badge) => $ssrg$atSource($ssrg$source273, () => ((__ssrg$instance$Render$0["render"](value)) as string), false)
export const status = (unit: undefined) => "ready"
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source396, () => _ssrg_console_println($ssrg$atSource($ssrg$source405, () => label(Active), false)), true)
