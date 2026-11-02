import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
const $ssrg$source166 = { file: "main.ssrg", line: 9, column: 18, uri: "seseragi://artifact/inherent-method", start: 166, end: 186 };
const $ssrg$source229 = { file: "main.ssrg", line: 13, column: 3, uri: "seseragi://artifact/inherent-method", start: 229, end: 268 };
const $ssrg$source230 = { file: "main.ssrg", line: 13, column: 4, uri: "seseragi://artifact/inherent-method", start: 230, end: 263 };
const $ssrg$source249 = { file: "main.ssrg", line: 13, column: 23, uri: "seseragi://artifact/inherent-method", start: 249, end: 262 };
const $ssrg$source333 = { file: "main.ssrg", line: 18, column: 3, uri: "seseragi://artifact/inherent-method", start: 333, end: 393 };
const $ssrg$source363 = { file: "main.ssrg", line: 18, column: 33, uri: "seseragi://artifact/inherent-method", start: 363, end: 391 };
$ssrg$assertUnicodeVersion("17.0.0")

declare const __ssrg$brand$Box: unique symbol;
type Box<A> = {
  readonly "value": A;
  readonly [__ssrg$brand$Box]: true;
};
const __ssrg$method$Box$get = <A,>(self: Box<A>) => (self)["value"]
const __ssrg$method$Box$map = <A, B,>(self: Box<A>) => (transform: (argument: A) => B) => (({ "value": $ssrg$atSource($ssrg$source166, () => transform((self)["value"]), false) } as const) as unknown as Box<B>)
const evaluate = (box: Box<number>) => $ssrg$atSource($ssrg$source229, () => __ssrg$method$Box$get<number>($ssrg$atSource($ssrg$source230, () => __ssrg$method$Box$map<number, number>(box)((value: number) => _ssrg_int_add(value, value, $ssrg$source249)), false)), false)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source333, () => _ssrg_console_println("Inherent method: " + _ssrg_show_intShow["show"]($ssrg$atSource($ssrg$source363, () => evaluate((({ "value": 21 } as const) as unknown as Box<number>)), false))), true)
