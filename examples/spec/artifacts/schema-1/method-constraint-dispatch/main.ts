import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source323 = { file: "main.ssrg", line: 20, column: 5, uri: "seseragi://artifact/method-constraint-dispatch", start: 323, end: 334 };
const $ssrg$source401 = { file: "main.ssrg", line: 26, column: 3, uri: "seseragi://artifact/method-constraint-dispatch", start: 401, end: 424 };
const $ssrg$source410 = { file: "main.ssrg", line: 26, column: 12, uri: "seseragi://artifact/method-constraint-dispatch", start: 410, end: 423 };
$ssrg$assertUnicodeVersion("17.0.0")

export type Badge =
  | { readonly tag: "Active" };
export const Active: Badge = { tag: "Active" } as const;
export const __ssrg$instance$Labeled$0 = { "label": (value: Badge) => "active" } as const;
export const __ssrg$instance$Render$1 = { "render": (value: Badge) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => $ssrg$atSource($ssrg$source323, () => ((__ssrg$evidence$0["label"](value)) as string), false), "__ssrg$method$value$render": (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => (value: Badge) => $ssrg$atSource($ssrg$source323, () => ((__ssrg$evidence$0["label"](value)) as string), false) } as const;
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source401, () => _ssrg_console_println($ssrg$atSource($ssrg$source410, () => ((__ssrg$instance$Render$1["render"](Active)(__ssrg$instance$Labeled$0)) as string), false)), true)
