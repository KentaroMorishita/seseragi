import { add as _ssrg_int64_add } from "@seseragi/runtime/int64"

export type Score =
  | { readonly tag: "Points"; readonly value: bigint };
export const Points = (value: bigint): Score => ({ tag: "Points", value } as const);
export const __ssrg$instance$Add$0 = { "add": (left: Score) => (right: Score) => (($ssrg_match: readonly [Score, Score]): Score => $ssrg_match[0].tag === "Points" && $ssrg_match[1].tag === "Points" ? ((leftValue: bigint, rightValue: bigint): Score => Points(_ssrg_int64_add(leftValue, rightValue)))($ssrg_match[0].value, $ssrg_match[1].value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())([left, right] as const) } as const;
