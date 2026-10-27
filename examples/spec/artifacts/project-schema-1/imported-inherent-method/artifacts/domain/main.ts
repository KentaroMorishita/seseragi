import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
$ssrg$assertUnicodeVersion("17.0.0")

interface $ssrg$Kind<Args extends readonly unknown[] = readonly unknown[]> { readonly args: Args; readonly type: unknown }
type $ssrg$Invoke<F extends $ssrg$Kind, Args extends readonly unknown[]> = (F & { readonly args: Args })["type"];
type $ssrg$Apply<F extends $ssrg$Kind, Args extends F["args"]> = $ssrg$Invoke<F, Args>;
export interface $ssrg$kind$Box extends $ssrg$Kind<readonly [unknown]> { readonly type: Box<this["args"][0]> }
declare const __ssrg$brand$Box: unique symbol;
export type Box<A> = {
  readonly [__ssrg$brand$Box]: readonly [A];
};
type __ssrg$representation$Box<A> = {
  readonly "value": A;
  readonly [__ssrg$brand$Box]: readonly [A];
};
export const box = <A,>(value: A) => (({ "value": value } as const) as unknown as Box<A>)
export const __ssrg$method$Box$map = <A,>(self: Box<A>) => (transform: (argument: A) => A) => (({ "value": transform((((((((self) as unknown)) as __ssrg$representation$Box<A>))["value"]) as A)) } as const) as unknown as Box<A>)
export const __ssrg$method$Box$get = <A,>(self: Box<A>) => (((((((self) as unknown)) as __ssrg$representation$Box<A>))["value"]) as A)
const __ssrg$method$Box$hidden = <A,>(self: Box<A>) => (((((((self) as unknown)) as __ssrg$representation$Box<A>))["value"]) as A)
