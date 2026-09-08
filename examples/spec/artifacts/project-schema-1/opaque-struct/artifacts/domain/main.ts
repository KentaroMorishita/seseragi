import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { boundedShow as _ssrg_show_boundedShow, intShow as _ssrg_show_intShow, type Show as _ssrg_show_Show } from "@seseragi/runtime/show"
import { intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
import { collectMap as _ssrg_array_comprehend } from "@seseragi/runtime/array"
$ssrg$assertUnicodeVersion("17.0.0")

declare const __ssrg$brand$Secret: unique symbol;
export type Secret = {
  readonly [__ssrg$brand$Secret]: true;
};
type __ssrg$representation$Secret = {
  readonly "value": number;
  readonly [__ssrg$brand$Secret]: true;
};
declare const __ssrg$brand$Box: unique symbol;
export type Box<A> = {
  readonly [__ssrg$brand$Box]: readonly [A];
};
type __ssrg$representation$Box<A> = {
  readonly "value": A;
  readonly [__ssrg$brand$Box]: readonly [A];
};
export const __ssrg$instance$Eq$0: { eq: (left: Secret) => (right: Secret) => boolean } = { eq: (left: Secret) => (right: Secret): boolean => { return _ssrg_int_eq_dictionary.eq((left as unknown as __ssrg$representation$Secret)["value"])((right as unknown as __ssrg$representation$Secret)["value"]); } };
export const __ssrg$instance$Show$1: _ssrg_show_Show<Secret> = _ssrg_show_boundedShow((value: Secret): string => "Secret { " + "value: " + _ssrg_show_intShow.show((value as unknown as __ssrg$representation$Secret)["value"]) + " }");
export const create = (value: number) => (({ "value": value } as const) as unknown as Secret)
export const read = (secret: Secret) => (((((((secret) as unknown)) as __ssrg$representation$Secret))["value"]) as number)
export const unpack = (secret: Secret) => (() => { const __ssrg$pattern$238: Secret = secret; return (() => { const value: number = (($ssrg_match: __ssrg$representation$Secret): number => ((value: number): number => value)($ssrg_match["value"]))(((((__ssrg$pattern$238) as unknown)) as __ssrg$representation$Secret)); return value; })(); })()
export const update = (secret: Secret) => (({ ...secret, "value": 9 } as const) as unknown as Secret)
export const box = <A,>(value: A) => (({ "value": value } as const) as unknown as Box<A>)
export const unbox = <A,>(boxed: Box<A>) => (() => { const __ssrg$pattern$437: Box<A> = boxed; return (() => { const value: A = (($ssrg_match: __ssrg$representation$Box<A>): A => ((value: A): A => value)($ssrg_match["value"]))(((((__ssrg$pattern$437) as unknown)) as __ssrg$representation$Box<A>)); return value; })(); })()
export const nested = (boxed: Box<Secret>) => (() => { const __ssrg$pattern$518: Box<Secret> = boxed; return (() => { const value: number = (($ssrg_match: __ssrg$representation$Box<__ssrg$representation$Secret>): number => ((value: number): number => value)($ssrg_match["value"]["value"]))(((((__ssrg$pattern$518) as unknown)) as __ssrg$representation$Box<__ssrg$representation$Secret>)); return value; })(); })()
export const collect = (secrets: ReadonlyArray<Secret>) => _ssrg_array_comprehend(secrets, ($ssrg_item0) => (($ssrg_match: __ssrg$representation$Secret): boolean => ((value: number): boolean => true)($ssrg_match["value"]))((((($ssrg_item0) as unknown)) as __ssrg$representation$Secret)), ($ssrg_item0) => (($ssrg_match: __ssrg$representation$Secret): number => ((value: number): number => value)($ssrg_match["value"]))((((($ssrg_item0) as unknown)) as __ssrg$representation$Secret)))
