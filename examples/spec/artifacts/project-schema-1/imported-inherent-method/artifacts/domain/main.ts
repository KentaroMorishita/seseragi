import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
$ssrg$assertUnicodeVersion("17.0.0")

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
