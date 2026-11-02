import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { reduce as _ssrg_array_reduce } from "@seseragi/runtime/array"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source219 = { file: "main.ssrg", line: 8, column: 22, uri: "seseragi://artifact/user-add-operator", start: 219, end: 240 };
const $ssrg$source226 = { file: "main.ssrg", line: 8, column: 29, uri: "seseragi://artifact/user-add-operator", start: 226, end: 239 };
const $ssrg$source302 = { file: "main.ssrg", line: 13, column: 3, uri: "seseragi://artifact/user-add-operator", start: 302, end: 315 };
const $ssrg$source358 = { file: "main.ssrg", line: 16, column: 3, uri: "seseragi://artifact/user-add-operator", start: 358, end: 390 };
const $ssrg$source378 = { file: "main.ssrg", line: 17, column: 14, uri: "seseragi://artifact/user-add-operator", start: 378, end: 385 };
const $ssrg$source546 = { file: "main.ssrg", line: 27, column: 3, uri: "seseragi://artifact/user-add-operator", start: 546, end: 569 };
$ssrg$assertUnicodeVersion("17.0.0")

export type Score =
  | { readonly tag: "Score"; readonly value: number };
export const Score = (value: number): Score => ({ tag: "Score", value } as const);
export const __ssrg$instance$Add$0 = { "add": (left: Score) => (right: number) => (($ssrg_match: Score): Score => $ssrg_match.tag === "Score" ? ((value: number): Score => $ssrg$atSource($ssrg$source219, () => Score(_ssrg_int_add(value, right, $ssrg$source226)), false))($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(left) } as const;
const addBonus = (bonus: number) => (score: Score) => $ssrg$atSource($ssrg$source302, () => __ssrg$instance$Add$0["add"](score)(bonus), false)
const total = (values: ReadonlyArray<number>) => $ssrg$atSource($ssrg$source358, () => addBonus(0)($ssrg$atSource($ssrg$source358, () => _ssrg_array_reduce($ssrg$atSource($ssrg$source378, () => Score(0), false), ((__ssrg$instance$Add$0["add"]) as (argument: Score) => (argument: number) => Score), values), false)), false)
const render = (score: Score) => (($ssrg_match: Score): string => $ssrg_match.tag === "Score" && $ssrg_match.value === 42 ? "User Add: 42" : "unexpected score")(score)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source546, () => _ssrg_console_println($ssrg$atSource($ssrg$source546, () => render($ssrg$atSource($ssrg$source546, () => total([10, 12, 20]), false)), false)), true)
