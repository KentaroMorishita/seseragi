import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { reduce as _ssrg_array_reduce } from "@seseragi/runtime/array"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
$ssrg$assertUnicodeVersion("17.0.0")

export type Score =
  | { readonly tag: "Score"; readonly value: number };
export const Score = (value: number): Score => ({ tag: "Score", value } as const);
export const __ssrg$instance$Add$0 = { "add": (left: Score) => (right: number) => (($ssrg_match: Score): Score => $ssrg_match.tag === "Score" ? ((value: number): Score => Score(_ssrg_int_add(value, right)))($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(left) } as const;
const addBonus = (bonus: number) => (score: Score) => __ssrg$instance$Add$0["add"](score)(bonus)
const total = (values: ReadonlyArray<number>) => addBonus(0)(_ssrg_array_reduce(Score(0), (((_argument0) => (_argument1) => __ssrg$instance$Add$0["add"](_argument0)(_argument1)) as (argument: Score) => (argument: number) => Score), values))
const render = (score: Score) => (($ssrg_match: Score): string => $ssrg_match.tag === "Score" && $ssrg_match.value === 42 ? "User Add: 42" : "unexpected score")(score)
export const main = (_unit: undefined) => _ssrg_console_println(render(total([10, 12, 20])))
