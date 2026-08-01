import { intShow as _ssrg_show_intShow, intDebug as _ssrg_debug_intDebug, floatShow as _ssrg_show_floatShow, boolDebug as _ssrg_debug_boolDebug, arrayDebug as _ssrg_debug_arrayDebug } from "@seseragi/runtime/show"
import { subtract as _ssrg_int_subtract } from "@seseragi/runtime/int"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

declare const __ssrg$brand$Snapshot: unique symbol;
export type Snapshot = {
  readonly "negative": number;
  readonly "negativeZero": number;
  readonly "inverted": boolean;
  readonly [__ssrg$brand$Snapshot]: true;
};
export const __ssrg$instance$Render$0 = { "render": (value: number) => _ssrg_show_intShow["show"](value) } as const;
export const negateInt = (value: number) => _ssrg_int_subtract(0, value)
export const negateFloat = (value: number) => -(value)
export const invert = (value: boolean) => !(value)
const identity = <A,>(value: A) => value
const __ssrg$operator$3c7e3e = (left: number) => (right: number) => _ssrg_int_subtract(left, right)
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println("first"), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_debug_intDebug["debug"](identity(_ssrg_int_subtract(0, 1)))), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_show_intShow["show"](_ssrg_int_subtract(0, 1))), () => _ssrg_effect_flatMap(_ssrg_console_println(__ssrg$instance$Render$0["render"](_ssrg_int_subtract(0, 2))), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_show_floatShow["show"](-(2.5))), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_debug_boolDebug["debug"](!(true))), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_debug_intDebug["debug"](_ssrg_int_subtract(0, 3))), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_debug_arrayDebug<number>(_ssrg_debug_intDebug)["debug"]([_ssrg_int_subtract(0, 4), _ssrg_int_subtract(0, 5)])), () => _ssrg_console_println(_ssrg_debug_intDebug["debug"](__ssrg$operator$3c7e3e(_ssrg_int_subtract(0, 5))(2)))))))))))
export const negative: number = _ssrg_int_subtract(0, 2);
export const negativeZero: number = -(0.0);
export const inverted: boolean = !(true);
export const minimum: number = _ssrg_int_subtract(0, 9007199254740991);
export const values: ReadonlyArray<number> = [_ssrg_int_subtract(0, 1), _ssrg_int_subtract(0, 2), _ssrg_int_subtract(0, 3)];
export const floats: ReadonlyArray<number> = [-(1.0), -(0.0), -(6.022e23)];
export const flags: ReadonlyArray<boolean> = [!(true), !(false)];
export const tuple: readonly [number, number, boolean] = [_ssrg_int_subtract(0, 4), -(2.5), !(false)] as const;
export const record: { readonly "inverted": boolean; readonly "negative": number } = ({ "negative": _ssrg_int_subtract(0, 5), "inverted": !(true) } as const);
export const snapshot: Snapshot = (({ "negative": _ssrg_int_subtract(0, 6), "negativeZero": -(0.0), "inverted": !(false) } as const) as unknown as Snapshot);
