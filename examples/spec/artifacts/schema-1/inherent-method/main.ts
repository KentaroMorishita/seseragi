import { add as _ssrg_int64_add } from "@seseragi/runtime/int64"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"

declare const __ssrg$brand$Box: unique symbol;
type Box<A> = {
  readonly "value": A;
  readonly [__ssrg$brand$Box]: true;
};
const get = <A,>(self: Box<A>) => (self)["value"]
const map = <A, B,>(self: Box<A>) => (transform: (argument: A) => B) => (({ "value": transform((self)["value"]) } as const) as unknown as Box<B>)
const evaluate = (box: Box<bigint>) => get(map(box)((value: bigint) => _ssrg_int64_add(value, value)))
export const main = (_unit: undefined) => _ssrg_console_println("Inherent method: " + _ssrg_show_intShow["show"](evaluate((({ "value": 21n } as const) as unknown as Box<bigint>))))
