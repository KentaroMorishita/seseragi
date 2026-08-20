import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { floatDebug as _ssrg_debug_floatDebug, arrayDebug as _ssrg_debug_arrayDebug } from "@seseragi/runtime/show"

const ratio = (value: number) => (value - 1.0) / (value + 1.0)
const averageDifference = (positive: number) => (negative: number) => (positive - negative) / 2.0
const double = (value: number) => value * 2.0
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_debug_arrayDebug<number>(_ssrg_debug_floatDebug)["debug"]([ratio(3.0), ratio(7.0), ratio(1.0)])), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_debug_arrayDebug<number>(_ssrg_debug_floatDebug)["debug"]([averageDifference(9.0)(1.0), averageDifference(2.0)(6.0)])), () => _ssrg_console_println(_ssrg_debug_floatDebug["debug"](groupedThroughPipeline))))
const groupedThroughPipeline: number = double((9.0 - 1.0) / 2.0);
