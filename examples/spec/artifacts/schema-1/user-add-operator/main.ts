import { add as _ssrg_int64_add } from "@seseragi/runtime/int64"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

export type Score =
  | { readonly tag: "Score"; readonly value: bigint };
export const Score = (value: bigint): Score => ({ tag: "Score", value } as const);
export const __ssrg$instance$Add$0 = { "add": (left: Score) => (right: Score) => (($ssrg_match: readonly [Score, Score]): Score => $ssrg_match[0].tag === "Score" && $ssrg_match[1].tag === "Score" ? ((leftValue: bigint, rightValue: bigint): Score => Score(_ssrg_int64_add(leftValue, rightValue)))($ssrg_match[0].value, $ssrg_match[1].value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())([left, right] as const) } as const;
const combine = <T,>(left: T) => (right: T) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => __ssrg$evidence$0["add"](left)(right)
const render = (score: Score) => (($ssrg_match: Score): string => $ssrg_match.tag === "Score" && $ssrg_match.value === 42n ? "User Add: 42" : "unexpected score")(score)
export const main = (_unit: undefined) => _ssrg_console_println(render(combine(Score(22n))(Score(20n))(__ssrg$instance$Add$0)))
