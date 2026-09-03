import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { floatDiv as _ssrg_float_div_dictionary, floatSub as _ssrg_float_sub_dictionary, floatAdd as _ssrg_float_add_dictionary, floatMul as _ssrg_float_mul_dictionary } from "@seseragi/runtime/float"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { floatDebug as _ssrg_debug_floatDebug, arrayDebug as _ssrg_debug_arrayDebug } from "@seseragi/runtime/show"
$ssrg$assertUnicodeVersion("17.0.0")

const ratio = (value: number) => _ssrg_float_div_dictionary["div"](_ssrg_float_sub_dictionary["sub"](value)(1.0))(_ssrg_float_add_dictionary["add"](value)(1.0))
const averageDifference = (positive: number) => (negative: number) => _ssrg_float_div_dictionary["div"](_ssrg_float_sub_dictionary["sub"](positive)(negative))(2.0)
const double = (value: number) => _ssrg_float_mul_dictionary["mul"](value)(2.0)
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_debug_arrayDebug<number>(_ssrg_debug_floatDebug)["debug"]([ratio(3.0), ratio(7.0), ratio(1.0)])), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_debug_arrayDebug<number>(_ssrg_debug_floatDebug)["debug"]([averageDifference(9.0)(1.0), averageDifference(2.0)(6.0)])), () => _ssrg_console_println(_ssrg_debug_floatDebug["debug"](groupedThroughPipeline))))
const groupedThroughPipeline: number = double(_ssrg_float_div_dictionary["div"](_ssrg_float_sub_dictionary["sub"](9.0)(1.0))(2.0));
