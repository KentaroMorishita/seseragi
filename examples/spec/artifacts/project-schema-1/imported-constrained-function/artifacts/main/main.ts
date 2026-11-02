import { describe } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source216 = { file: "project/imported-constrained-function/src/main.ssrg", line: 13, column: 3, uri: "seseragi://fixture/imported-constrained-function::main", start: 216, end: 241 };
const $ssrg$source225 = { file: "project/imported-constrained-function/src/main.ssrg", line: 13, column: 12, uri: "seseragi://fixture/imported-constrained-function::main", start: 225, end: 240 };
$ssrg$assertUnicodeVersion("17.0.0")

export type Badge =
  | { readonly tag: "Active" };
export const Active: Badge = { tag: "Active" } as const;
export const __ssrg$instance$Ready$0 = { "ready": (value: Badge) => "imported ready" } as const;
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source216, () => _ssrg_console_println($ssrg$atSource($ssrg$source225, () => ((describe<Badge>(Active)(__ssrg$instance$Ready$0)) as string), false)), true)
