declare const __ssrg$brand$Box: unique symbol;
export type Box<A> = {
  readonly "value": A;
  readonly [__ssrg$brand$Box]: true;
};
export const box = <A,>(value: A) => (({ "value": value } as const) as unknown as Box<A>)
export const __ssrg$method$Box$map = <A,>(self: Box<A>) => (transform: (argument: A) => A) => (({ "value": transform((self)["value"]) } as const) as unknown as Box<A>)
export const __ssrg$method$Box$get = <A,>(self: Box<A>) => (self)["value"]
const __ssrg$method$Box$hidden = <A,>(self: Box<A>) => (self)["value"]
