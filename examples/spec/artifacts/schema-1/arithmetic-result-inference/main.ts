import { subtract as _ssrg_int64_subtract, add as _ssrg_int64_add, multiply as _ssrg_int64_multiply, divide as _ssrg_int64_divide, remainder as _ssrg_int64_remainder, power as _ssrg_int64_power } from "@seseragi/runtime/int64"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intDebug as _ssrg_debug_intDebug, arrayDebug as _ssrg_debug_arrayDebug, stringDebug as _ssrg_debug_stringDebug } from "@seseragi/runtime/show"

const difference: bigint = _ssrg_int64_subtract(1n, 2n);
const annotated: bigint = _ssrg_int64_subtract(1n, 2n);
const arithmeticMatrix: ReadonlyArray<bigint> = [_ssrg_int64_add(1n, 2n), _ssrg_int64_subtract(4n, 1n), _ssrg_int64_multiply(2n, 3n), _ssrg_int64_divide(8n, 2n), _ssrg_int64_remainder(7n, 4n), _ssrg_int64_power(2n, 3n)];
const joined: string = "Sese" + "ragi";
const acceptInt = (value: bigint) => value
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_debug_intDebug["debug"](difference)), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_debug_intDebug["debug"](annotated)), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_debug_arrayDebug<bigint>(_ssrg_debug_intDebug)["debug"](arithmeticMatrix)), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_debug_intDebug["debug"](acceptInt(_ssrg_int64_subtract(10n, 3n)))), () => _ssrg_console_println(_ssrg_debug_stringDebug["debug"](joined))))))
