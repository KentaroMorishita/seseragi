import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { intEq as _ssrg_int_eq_dictionary, intOrd as _ssrg_int_ord_dictionary } from "@seseragi/runtime/equality"
$ssrg$assertUnicodeVersion("17.0.0")

declare const __ssrg$brand$Score: unique symbol;
export type Score = {
  readonly "value": number;
  readonly [__ssrg$brand$Score]: true;
};
export const __ssrg$instance$Eq$0 = { "eq": (left: Score) => (right: Score) => _ssrg_int_eq_dictionary["eq"]((left)["value"])((right)["value"]) } as const;
export const __ssrg$instance$Ord$1 = (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => ({ ...__ssrg$evidence$0, "compare": (left: Score) => (right: Score) => ((_ssrg_int_ord_dictionary["compare"]((right)["value"])((left)["value"])) as { readonly tag: "Less" } | { readonly tag: "Equal" } | { readonly tag: "Greater" }) }) as const;
export const ordered = <A,>(left: A) => (right: A) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => (__ssrg$evidence$0["compare"](left)(right))["tag"] !== "Greater"
