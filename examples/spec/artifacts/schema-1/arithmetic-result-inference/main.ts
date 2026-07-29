import { subtract as _ssrg_int_subtract, add as _ssrg_int_add, multiply as _ssrg_int_multiply, divide as _ssrg_int_divide, remainder as _ssrg_int_remainder, power as _ssrg_int_power } from "@seseragi/runtime/int"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intDebug as _ssrg_debug_intDebug, arrayDebug as _ssrg_debug_arrayDebug, stringDebug as _ssrg_debug_stringDebug } from "@seseragi/runtime/show"

const acceptInt = (value: number) => value
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_debug_intDebug["debug"](difference)), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_debug_intDebug["debug"](annotated)), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_debug_arrayDebug<number>(_ssrg_debug_intDebug)["debug"](arithmeticMatrix)), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_debug_intDebug["debug"](acceptInt(_ssrg_int_subtract(10, 3)))), () => _ssrg_console_println(_ssrg_debug_stringDebug["debug"](joined))))))
const difference: number = _ssrg_int_subtract(1, 2);
const annotated: number = _ssrg_int_subtract(1, 2);
const arithmeticMatrix: ReadonlyArray<number> = [_ssrg_int_add(1, 2), _ssrg_int_subtract(4, 1), _ssrg_int_multiply(2, 3), _ssrg_int_divide(8, 2), _ssrg_int_remainder(7, 4), _ssrg_int_power(2, 3)];
const joined: string = "Sese" + "ragi";
