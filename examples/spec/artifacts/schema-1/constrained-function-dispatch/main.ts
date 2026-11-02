import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source220 = { file: "main.ssrg", line: 14, column: 3, uri: "seseragi://artifact/constrained-function-dispatch", start: 220, end: 231 };
const $ssrg$source296 = { file: "main.ssrg", line: 19, column: 3, uri: "seseragi://artifact/constrained-function-dispatch", start: 296, end: 321 };
const $ssrg$source305 = { file: "main.ssrg", line: 19, column: 12, uri: "seseragi://artifact/constrained-function-dispatch", start: 305, end: 320 };
$ssrg$assertUnicodeVersion("17.0.0")

export type Badge =
  | { readonly tag: "Active" };
export const Active: Badge = { tag: "Active" } as const;
export const __ssrg$instance$Ready$0 = { "ready": (value: Badge) => "Badge is ready" } as const;
export const describe = <T,>(value: T) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => $ssrg$atSource($ssrg$source220, () => ((__ssrg$evidence$0["ready"](value)) as string), false)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source296, () => _ssrg_console_println($ssrg$atSource($ssrg$source305, () => ((describe<Badge>(Active)(__ssrg$instance$Ready$0)) as string), false)), true)
