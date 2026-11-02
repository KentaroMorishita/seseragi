import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { floatDiv as _ssrg_float_div_dictionary, floatSub as _ssrg_float_sub_dictionary, floatAdd as _ssrg_float_add_dictionary, floatMul as _ssrg_float_mul_dictionary } from "@seseragi/runtime/float"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { floatDebug as _ssrg_debug_floatDebug, arrayDebug as _ssrg_debug_arrayDebug } from "@seseragi/runtime/show"
const $ssrg$source35 = { file: "main.ssrg", line: 2, column: 3, uri: "seseragi://artifact/typescript-precedence-grouping", start: 35, end: 64 };
const $ssrg$source36 = { file: "main.ssrg", line: 2, column: 4, uri: "seseragi://artifact/typescript-precedence-grouping", start: 36, end: 47 };
const $ssrg$source52 = { file: "main.ssrg", line: 2, column: 20, uri: "seseragi://artifact/typescript-precedence-grouping", start: 52, end: 63 };
const $ssrg$source135 = { file: "main.ssrg", line: 5, column: 3, uri: "seseragi://artifact/typescript-precedence-grouping", start: 135, end: 162 };
const $ssrg$source136 = { file: "main.ssrg", line: 5, column: 4, uri: "seseragi://artifact/typescript-precedence-grouping", start: 136, end: 155 };
const $ssrg$source200 = { file: "main.ssrg", line: 8, column: 3, uri: "seseragi://artifact/typescript-precedence-grouping", start: 200, end: 211 };
const $ssrg$source244 = { file: "main.ssrg", line: 11, column: 3, uri: "seseragi://artifact/typescript-precedence-grouping", start: 244, end: 275 };
const $ssrg$source245 = { file: "main.ssrg", line: 11, column: 4, uri: "seseragi://artifact/typescript-precedence-grouping", start: 245, end: 262 };
const $ssrg$source246 = { file: "main.ssrg", line: 11, column: 5, uri: "seseragi://artifact/typescript-precedence-grouping", start: 246, end: 255 };
const $ssrg$source309 = { file: "main.ssrg", line: 16, column: 5, uri: "seseragi://artifact/typescript-precedence-grouping", start: 309, end: 358 };
const $ssrg$source318 = { file: "main.ssrg", line: 16, column: 14, uri: "seseragi://artifact/typescript-precedence-grouping", start: 318, end: 357 };
const $ssrg$source325 = { file: "main.ssrg", line: 16, column: 21, uri: "seseragi://artifact/typescript-precedence-grouping", start: 325, end: 334 };
const $ssrg$source336 = { file: "main.ssrg", line: 16, column: 32, uri: "seseragi://artifact/typescript-precedence-grouping", start: 336, end: 345 };
const $ssrg$source347 = { file: "main.ssrg", line: 16, column: 43, uri: "seseragi://artifact/typescript-precedence-grouping", start: 347, end: 356 };
const $ssrg$source363 = { file: "main.ssrg", line: 17, column: 5, uri: "seseragi://artifact/typescript-precedence-grouping", start: 363, end: 433 };
const $ssrg$source372 = { file: "main.ssrg", line: 17, column: 14, uri: "seseragi://artifact/typescript-precedence-grouping", start: 372, end: 432 };
const $ssrg$source379 = { file: "main.ssrg", line: 17, column: 21, uri: "seseragi://artifact/typescript-precedence-grouping", start: 379, end: 404 };
const $ssrg$source406 = { file: "main.ssrg", line: 17, column: 48, uri: "seseragi://artifact/typescript-precedence-grouping", start: 406, end: 431 };
const $ssrg$source438 = { file: "main.ssrg", line: 18, column: 5, uri: "seseragi://artifact/typescript-precedence-grouping", start: 438, end: 476 };
const $ssrg$source447 = { file: "main.ssrg", line: 18, column: 14, uri: "seseragi://artifact/typescript-precedence-grouping", start: 447, end: 475 };
$ssrg$assertUnicodeVersion("17.0.0")

const ratio = (value: number) => $ssrg$atSource($ssrg$source35, () => _ssrg_float_div_dictionary["div"]($ssrg$atSource($ssrg$source36, () => _ssrg_float_sub_dictionary["sub"](value)(1.0), false))($ssrg$atSource($ssrg$source52, () => _ssrg_float_add_dictionary["add"](value)(1.0), false)), false)
const averageDifference = (positive: number) => (negative: number) => $ssrg$atSource($ssrg$source135, () => _ssrg_float_div_dictionary["div"]($ssrg$atSource($ssrg$source136, () => _ssrg_float_sub_dictionary["sub"](positive)(negative), false))(2.0), false)
const double = (value: number) => $ssrg$atSource($ssrg$source200, () => _ssrg_float_mul_dictionary["mul"](value)(2.0), false)
export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source309, () => _ssrg_console_println($ssrg$atSource($ssrg$source318, () => ((_ssrg_debug_arrayDebug<number>(_ssrg_debug_floatDebug)["debug"]([$ssrg$atSource($ssrg$source325, () => ratio(3.0), false), $ssrg$atSource($ssrg$source336, () => ratio(7.0), false), $ssrg$atSource($ssrg$source347, () => ratio(1.0), false)])) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source363, () => _ssrg_console_println($ssrg$atSource($ssrg$source372, () => ((_ssrg_debug_arrayDebug<number>(_ssrg_debug_floatDebug)["debug"]([$ssrg$atSource($ssrg$source379, () => averageDifference(9.0)(1.0), false), $ssrg$atSource($ssrg$source406, () => averageDifference(2.0)(6.0), false)])) as string), false)), true), () => $ssrg$atSource($ssrg$source438, () => _ssrg_console_println($ssrg$atSource($ssrg$source447, () => ((_ssrg_debug_floatDebug["debug"](groupedThroughPipeline)) as string), false)), true)))
const groupedThroughPipeline: number = $ssrg$atSource($ssrg$source244, () => double($ssrg$atSource($ssrg$source245, () => _ssrg_float_div_dictionary["div"]($ssrg$atSource($ssrg$source246, () => _ssrg_float_sub_dictionary["sub"](9.0)(1.0), false))(2.0), false)), false);
