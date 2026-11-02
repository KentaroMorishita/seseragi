import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { subtract as _ssrg_int_subtract, add as _ssrg_int_add, multiply as _ssrg_int_multiply, divide as _ssrg_int_divide, remainder as _ssrg_int_remainder, power as _ssrg_int_power } from "@seseragi/runtime/int"
import { floatAdd as _ssrg_float_add_dictionary, floatSub as _ssrg_float_sub_dictionary, floatMul as _ssrg_float_mul_dictionary, floatDiv as _ssrg_float_div_dictionary, floatRem as _ssrg_float_rem_dictionary, floatPow as _ssrg_float_pow_dictionary } from "@seseragi/runtime/float"
import { stringAdd as _ssrg_string_add_dictionary } from "@seseragi/runtime/string"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intDebug as _ssrg_debug_intDebug, arrayDebug as _ssrg_debug_arrayDebug, floatDebug as _ssrg_debug_floatDebug, stringDebug as _ssrg_debug_stringDebug } from "@seseragi/runtime/show"
const $ssrg$source118 = { file: "main.ssrg", line: 2, column: 18, uri: "seseragi://artifact/arithmetic-result-inference", start: 118, end: 123 };
const $ssrg$source146 = { file: "main.ssrg", line: 4, column: 22, uri: "seseragi://artifact/arithmetic-result-inference", start: 146, end: 151 };
const $ssrg$source180 = { file: "main.ssrg", line: 7, column: 3, uri: "seseragi://artifact/arithmetic-result-inference", start: 180, end: 185 };
const $ssrg$source189 = { file: "main.ssrg", line: 8, column: 3, uri: "seseragi://artifact/arithmetic-result-inference", start: 189, end: 194 };
const $ssrg$source198 = { file: "main.ssrg", line: 9, column: 3, uri: "seseragi://artifact/arithmetic-result-inference", start: 198, end: 203 };
const $ssrg$source207 = { file: "main.ssrg", line: 10, column: 3, uri: "seseragi://artifact/arithmetic-result-inference", start: 207, end: 212 };
const $ssrg$source216 = { file: "main.ssrg", line: 11, column: 3, uri: "seseragi://artifact/arithmetic-result-inference", start: 216, end: 221 };
const $ssrg$source225 = { file: "main.ssrg", line: 12, column: 3, uri: "seseragi://artifact/arithmetic-result-inference", start: 225, end: 231 };
const $ssrg$source267 = { file: "main.ssrg", line: 16, column: 3, uri: "seseragi://artifact/arithmetic-result-inference", start: 267, end: 277 };
const $ssrg$source281 = { file: "main.ssrg", line: 17, column: 3, uri: "seseragi://artifact/arithmetic-result-inference", start: 281, end: 291 };
const $ssrg$source295 = { file: "main.ssrg", line: 18, column: 3, uri: "seseragi://artifact/arithmetic-result-inference", start: 295, end: 304 };
const $ssrg$source308 = { file: "main.ssrg", line: 19, column: 3, uri: "seseragi://artifact/arithmetic-result-inference", start: 308, end: 317 };
const $ssrg$source321 = { file: "main.ssrg", line: 20, column: 3, uri: "seseragi://artifact/arithmetic-result-inference", start: 321, end: 330 };
const $ssrg$source334 = { file: "main.ssrg", line: 21, column: 3, uri: "seseragi://artifact/arithmetic-result-inference", start: 334, end: 344 };
const $ssrg$source361 = { file: "main.ssrg", line: 24, column: 14, uri: "seseragi://artifact/arithmetic-result-inference", start: 361, end: 376 };
const $ssrg$source452 = { file: "main.ssrg", line: 31, column: 5, uri: "seseragi://artifact/arithmetic-result-inference", start: 452, end: 478 };
const $ssrg$source461 = { file: "main.ssrg", line: 31, column: 14, uri: "seseragi://artifact/arithmetic-result-inference", start: 461, end: 477 };
const $ssrg$source483 = { file: "main.ssrg", line: 32, column: 5, uri: "seseragi://artifact/arithmetic-result-inference", start: 483, end: 508 };
const $ssrg$source492 = { file: "main.ssrg", line: 32, column: 14, uri: "seseragi://artifact/arithmetic-result-inference", start: 492, end: 507 };
const $ssrg$source513 = { file: "main.ssrg", line: 33, column: 5, uri: "seseragi://artifact/arithmetic-result-inference", start: 513, end: 545 };
const $ssrg$source522 = { file: "main.ssrg", line: 33, column: 14, uri: "seseragi://artifact/arithmetic-result-inference", start: 522, end: 544 };
const $ssrg$source550 = { file: "main.ssrg", line: 34, column: 5, uri: "seseragi://artifact/arithmetic-result-inference", start: 550, end: 587 };
const $ssrg$source559 = { file: "main.ssrg", line: 34, column: 14, uri: "seseragi://artifact/arithmetic-result-inference", start: 559, end: 586 };
const $ssrg$source592 = { file: "main.ssrg", line: 35, column: 5, uri: "seseragi://artifact/arithmetic-result-inference", start: 592, end: 628 };
const $ssrg$source601 = { file: "main.ssrg", line: 35, column: 14, uri: "seseragi://artifact/arithmetic-result-inference", start: 601, end: 627 };
const $ssrg$source608 = { file: "main.ssrg", line: 35, column: 21, uri: "seseragi://artifact/arithmetic-result-inference", start: 608, end: 626 };
const $ssrg$source619 = { file: "main.ssrg", line: 35, column: 32, uri: "seseragi://artifact/arithmetic-result-inference", start: 619, end: 625 };
const $ssrg$source633 = { file: "main.ssrg", line: 36, column: 5, uri: "seseragi://artifact/arithmetic-result-inference", start: 633, end: 655 };
const $ssrg$source642 = { file: "main.ssrg", line: 36, column: 14, uri: "seseragi://artifact/arithmetic-result-inference", start: 642, end: 654 };
$ssrg$assertUnicodeVersion("17.0.0")

const acceptInt = (value: number) => value
export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source452, () => _ssrg_console_println($ssrg$atSource($ssrg$source461, () => ((_ssrg_debug_intDebug["debug"](difference)) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source483, () => _ssrg_console_println($ssrg$atSource($ssrg$source492, () => ((_ssrg_debug_intDebug["debug"](annotated)) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source513, () => _ssrg_console_println($ssrg$atSource($ssrg$source522, () => ((_ssrg_debug_arrayDebug<number>(_ssrg_debug_intDebug)["debug"](arithmeticMatrix)) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source550, () => _ssrg_console_println($ssrg$atSource($ssrg$source559, () => ((_ssrg_debug_arrayDebug<number>(_ssrg_debug_floatDebug)["debug"](floatArithmeticMatrix)) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source592, () => _ssrg_console_println($ssrg$atSource($ssrg$source601, () => ((_ssrg_debug_intDebug["debug"]($ssrg$atSource($ssrg$source608, () => acceptInt(_ssrg_int_subtract(10, 3, $ssrg$source619)), false))) as string), false)), true), () => $ssrg$atSource($ssrg$source633, () => _ssrg_console_println($ssrg$atSource($ssrg$source642, () => ((_ssrg_debug_stringDebug["debug"](joined)) as string), false)), true))))))
const difference: number = _ssrg_int_subtract(1, 2, $ssrg$source118);
const annotated: number = _ssrg_int_subtract(1, 2, $ssrg$source146);
const arithmeticMatrix: ReadonlyArray<number> = [_ssrg_int_add(1, 2, $ssrg$source180), _ssrg_int_subtract(4, 1, $ssrg$source189), _ssrg_int_multiply(2, 3, $ssrg$source198), _ssrg_int_divide(8, 2, $ssrg$source207), _ssrg_int_remainder(7, 4, $ssrg$source216), _ssrg_int_power(2, 3, $ssrg$source225)];
const floatArithmeticMatrix: ReadonlyArray<number> = [$ssrg$atSource($ssrg$source267, () => _ssrg_float_add_dictionary["add"](1.5)(2.25), false), $ssrg$atSource($ssrg$source281, () => _ssrg_float_sub_dictionary["sub"](4.5)(1.25), false), $ssrg$atSource($ssrg$source295, () => _ssrg_float_mul_dictionary["mul"](2.5)(3.0), false), $ssrg$atSource($ssrg$source308, () => _ssrg_float_div_dictionary["div"](8.0)(2.0), false), $ssrg$atSource($ssrg$source321, () => _ssrg_float_rem_dictionary["rem"](7.5)(4.0), false), $ssrg$atSource($ssrg$source334, () => _ssrg_float_pow_dictionary["pow"](2.0)(3.0), false)];
const joined: string = $ssrg$atSource($ssrg$source361, () => _ssrg_string_add_dictionary["add"]("Sese")("ragi"), false);
