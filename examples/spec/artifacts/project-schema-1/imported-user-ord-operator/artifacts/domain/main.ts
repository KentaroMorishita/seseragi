import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { intEq as _ssrg_int_eq_dictionary, intOrd as _ssrg_int_ord_dictionary } from "@seseragi/runtime/equality"
const $ssrg$source99 = { file: "project/imported-user-ord-operator/src/domain.ssrg", line: 3, column: 47, uri: "seseragi://fixture/imported-user-ord-operator::domain", start: 99, end: 124 };
const $ssrg$source204 = { file: "project/imported-user-ord-operator/src/domain.ssrg", line: 6, column: 56, uri: "seseragi://fixture/imported-user-ord-operator::domain", start: 204, end: 234 };
$ssrg$assertUnicodeVersion("17.0.0")

declare const __ssrg$brand$Score: unique symbol;
export type Score = {
  readonly "value": number;
  readonly [__ssrg$brand$Score]: true;
};
export const __ssrg$instance$Eq$0 = { "eq": (left: Score) => (right: Score) => $ssrg$atSource($ssrg$source99, () => _ssrg_int_eq_dictionary["eq"]((left)["value"])((right)["value"]), false) } as const;
export const __ssrg$instance$Ord$1 = (__ssrg$evidence$0: import("@seseragi/runtime/equality").Eq<Score>) => ({ ...__ssrg$evidence$0, "compare": (left: Score) => (right: Score) => $ssrg$atSource($ssrg$source204, () => ((_ssrg_int_ord_dictionary["compare"]((right)["value"])((left)["value"])) as { readonly tag: "Less" } | { readonly tag: "Equal" } | { readonly tag: "Greater" }), false) }) as const;
export const ordered = <A,>(left: A) => (right: A) => (__ssrg$evidence$0: import("@seseragi/runtime/equality").Ord<A> & import("@seseragi/runtime/equality").Eq<A>) => (__ssrg$evidence$0["compare"](left)(right))["tag"] !== "Greater"
