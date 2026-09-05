import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { subtract as _ssrg_int_subtract, add as _ssrg_int_add, multiply as _ssrg_int_multiply, divide as _ssrg_int_divide, remainder as _ssrg_int_remainder, power as _ssrg_int_power } from "@seseragi/runtime/int"
import { floatAdd as _ssrg_float_add_dictionary, floatSub as _ssrg_float_sub_dictionary, floatMul as _ssrg_float_mul_dictionary, floatDiv as _ssrg_float_div_dictionary, floatRem as _ssrg_float_rem_dictionary, floatPow as _ssrg_float_pow_dictionary } from "@seseragi/runtime/float"
import { stringAdd as _ssrg_string_add_dictionary } from "@seseragi/runtime/string"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intDebug as _ssrg_debug_intDebug, arrayDebug as _ssrg_debug_arrayDebug, floatDebug as _ssrg_debug_floatDebug, stringDebug as _ssrg_debug_stringDebug } from "@seseragi/runtime/show"
$ssrg$assertUnicodeVersion("17.0.0")

const acceptInt = (value: number) => value
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_debug_intDebug["debug"](difference)) as string)), () => _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_debug_intDebug["debug"](annotated)) as string)), () => _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_debug_arrayDebug<number>(_ssrg_debug_intDebug)["debug"](arithmeticMatrix)) as string)), () => _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_debug_arrayDebug<number>(_ssrg_debug_floatDebug)["debug"](floatArithmeticMatrix)) as string)), () => _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_debug_intDebug["debug"](acceptInt(_ssrg_int_subtract(10, 3)))) as string)), () => _ssrg_console_println(((_ssrg_debug_stringDebug["debug"](joined)) as string)))))))
const difference: number = _ssrg_int_subtract(1, 2);
const annotated: number = _ssrg_int_subtract(1, 2);
const arithmeticMatrix: ReadonlyArray<number> = [_ssrg_int_add(1, 2), _ssrg_int_subtract(4, 1), _ssrg_int_multiply(2, 3), _ssrg_int_divide(8, 2), _ssrg_int_remainder(7, 4), _ssrg_int_power(2, 3)];
const floatArithmeticMatrix: ReadonlyArray<number> = [_ssrg_float_add_dictionary["add"](1.5)(2.25), _ssrg_float_sub_dictionary["sub"](4.5)(1.25), _ssrg_float_mul_dictionary["mul"](2.5)(3.0), _ssrg_float_div_dictionary["div"](8.0)(2.0), _ssrg_float_rem_dictionary["rem"](7.5)(4.0), _ssrg_float_pow_dictionary["pow"](2.0)(3.0)];
const joined: string = _ssrg_string_add_dictionary["add"]("Sese")("ragi");
