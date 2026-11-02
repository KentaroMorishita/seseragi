import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
const $ssrg$source152 = { file: "project/imported-user-add-operator/src/domain.ssrg", line: 7, column: 23, uri: "seseragi://fixture/imported-user-add-operator::domain", start: 152, end: 174 };
const $ssrg$source160 = { file: "project/imported-user-add-operator/src/domain.ssrg", line: 7, column: 31, uri: "seseragi://fixture/imported-user-add-operator::domain", start: 160, end: 173 };
$ssrg$assertUnicodeVersion("17.0.0")

export type Score =
  | { readonly tag: "Points"; readonly value: number };
export const Points = (value: number): Score => ({ tag: "Points", value } as const);
export const __ssrg$instance$Add$0 = { "add": (left: Score) => (right: number) => (($ssrg_match: Score): Score => $ssrg_match.tag === "Points" ? ((value: number): Score => $ssrg$atSource($ssrg$source152, () => Points(_ssrg_int_add(value, right, $ssrg$source160)), false))($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(left) } as const;
