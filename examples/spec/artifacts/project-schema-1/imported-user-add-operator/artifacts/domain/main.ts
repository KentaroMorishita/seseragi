import { add as _ssrg_int_add } from "@seseragi/runtime/int"

export type Score =
  | { readonly tag: "Points"; readonly value: number };
export const Points = (value: number): Score => ({ tag: "Points", value } as const);
export const __ssrg$instance$Add$0 = { "add": (left: Score) => (right: number) => (($ssrg_match: Score): Score => $ssrg_match.tag === "Points" ? ((value: number): Score => Points(_ssrg_int_add(value, right)))($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(left) } as const;
