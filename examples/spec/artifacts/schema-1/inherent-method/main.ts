import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"

declare const __ssrg$brand$Box: unique symbol;
type Box<A> = {
  readonly "value": A;
  readonly [__ssrg$brand$Box]: true;
};
const __ssrg$method$Box$get = <A,>(self: Box<A>) => (self)["value"]
const __ssrg$method$Box$map = <A, B,>(self: Box<A>) => (transform: (argument: A) => B) => (({ "value": transform((self)["value"]) } as const) as unknown as Box<B>)
const evaluate = (box: Box<number>) => __ssrg$method$Box$get(__ssrg$method$Box$map(box)((value: number) => _ssrg_int_add(value, value)))
export const main = (_unit: undefined) => _ssrg_console_println("Inherent method: " + _ssrg_show_intShow["show"](evaluate((({ "value": 21 } as const) as unknown as Box<number>))))
