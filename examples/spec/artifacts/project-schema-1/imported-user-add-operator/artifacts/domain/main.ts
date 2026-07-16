import { add as _ssrg_int64_add } from "@seseragi/runtime/int64"

export type Score =
  | { readonly tag: "Points"; readonly value: bigint };
export const Points = (value: bigint): Score => ({ tag: "Points", value } as const);
export const __ssrg$instance$Add$0 = { "add": (left: Score) => (right: bigint) => (($ssrg_match: Score): Score => $ssrg_match.tag === "Points" ? ((value: bigint): Score => Points(_ssrg_int64_add(value, right)))($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(left) } as const;
