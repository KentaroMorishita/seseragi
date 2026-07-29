import { intShow as _ssrg_show_intShow, rangeShow as _ssrg_show_rangeShow, intDebug as _ssrg_debug_intDebug, rangeDebug as _ssrg_debug_rangeDebug, arrayDebug as _ssrg_debug_arrayDebug } from "@seseragi/runtime/show"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { exclusive as _ssrg_range_exclusive, inclusive as _ssrg_range_inclusive } from "@seseragi/runtime/range"

export const showRange = (value: Readonly<{ start: number; end: number; inclusive: boolean }>) => _ssrg_show_rangeShow<number>(_ssrg_show_intShow)["show"](value)
export const debugRange = (value: Readonly<{ start: number; end: number; inclusive: boolean }>) => _ssrg_debug_rangeDebug<number>(_ssrg_debug_intDebug)["debug"](value)
export const debugRanges = (values: ReadonlyArray<Readonly<{ start: number; end: number; inclusive: boolean }>>) => _ssrg_debug_arrayDebug<Readonly<{ start: number; end: number; inclusive: boolean }>>(_ssrg_debug_rangeDebug<number>(_ssrg_debug_intDebug))["debug"](values)
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_show_rangeShow<number>(_ssrg_show_intShow)["show"](_ssrg_range_exclusive(1, 5))), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_debug_rangeDebug<number>(_ssrg_debug_intDebug)["debug"](_ssrg_range_inclusive(5, 5))), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_show_rangeShow<number>(_ssrg_show_intShow)["show"](_ssrg_range_inclusive(10, 1))), () => _ssrg_console_println(_ssrg_debug_arrayDebug<Readonly<{ start: number; end: number; inclusive: boolean }>>(_ssrg_debug_rangeDebug<number>(_ssrg_debug_intDebug))["debug"]([_ssrg_range_exclusive(1, 3), _ssrg_range_inclusive(3, 3)])))))
