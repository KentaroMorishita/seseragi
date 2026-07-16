import { add as _ssrg_int64_add } from "@seseragi/runtime/int64"
import { reduce as _ssrg_array_reduce } from "@seseragi/runtime/array"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

export type Score =
  | { readonly tag: "Score"; readonly value: bigint };
export const Score = (value: bigint): Score => ({ tag: "Score", value } as const);
export const __ssrg$instance$Add$0 = { "add": (left: Score) => (right: bigint) => (($ssrg_match: Score): Score => $ssrg_match.tag === "Score" ? ((value: bigint): Score => Score(_ssrg_int64_add(value, right)))($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(left) } as const;
const addBonus = (bonus: bigint) => (score: Score) => __ssrg$instance$Add$0["add"](score)(bonus)
const total = (values: ReadonlyArray<bigint>) => addBonus(0n)(_ssrg_array_reduce(Score(0n), (_argument0) => (_argument1) => __ssrg$instance$Add$0["add"](_argument0)(_argument1), values))
const render = (score: Score) => (($ssrg_match: Score): string => $ssrg_match.tag === "Score" && $ssrg_match.value === 42n ? "User Add: 42" : "unexpected score")(score)
export const main = (_unit: undefined) => _ssrg_console_println(render(total([10n, 12n, 20n])))
