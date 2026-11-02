import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { intShow as _ssrg_show_intShow, rangeShow as _ssrg_show_rangeShow, intDebug as _ssrg_debug_intDebug, rangeDebug as _ssrg_debug_rangeDebug, arrayDebug as _ssrg_debug_arrayDebug } from "@seseragi/runtime/show"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { exclusive as _ssrg_range_exclusive, inclusive as _ssrg_range_inclusive } from "@seseragi/runtime/range"
const $ssrg$source49 = { file: "main.ssrg", line: 2, column: 3, uri: "seseragi://artifact/show-debug-range", start: 49, end: 59 };
const $ssrg$source111 = { file: "main.ssrg", line: 5, column: 3, uri: "seseragi://artifact/show-debug-range", start: 111, end: 122 };
const $ssrg$source183 = { file: "main.ssrg", line: 8, column: 3, uri: "seseragi://artifact/show-debug-range", start: 183, end: 195 };
const $ssrg$source229 = { file: "main.ssrg", line: 12, column: 5, uri: "seseragi://artifact/show-debug-range", start: 229, end: 250 };
const $ssrg$source238 = { file: "main.ssrg", line: 12, column: 14, uri: "seseragi://artifact/show-debug-range", start: 238, end: 249 };
const $ssrg$source244 = { file: "main.ssrg", line: 12, column: 20, uri: "seseragi://artifact/show-debug-range", start: 244, end: 248 };
const $ssrg$source255 = { file: "main.ssrg", line: 13, column: 5, uri: "seseragi://artifact/show-debug-range", start: 255, end: 278 };
const $ssrg$source264 = { file: "main.ssrg", line: 13, column: 14, uri: "seseragi://artifact/show-debug-range", start: 264, end: 277 };
const $ssrg$source271 = { file: "main.ssrg", line: 13, column: 21, uri: "seseragi://artifact/show-debug-range", start: 271, end: 276 };
const $ssrg$source283 = { file: "main.ssrg", line: 14, column: 5, uri: "seseragi://artifact/show-debug-range", start: 283, end: 306 };
const $ssrg$source292 = { file: "main.ssrg", line: 14, column: 14, uri: "seseragi://artifact/show-debug-range", start: 292, end: 305 };
const $ssrg$source298 = { file: "main.ssrg", line: 14, column: 20, uri: "seseragi://artifact/show-debug-range", start: 298, end: 304 };
const $ssrg$source311 = { file: "main.ssrg", line: 15, column: 5, uri: "seseragi://artifact/show-debug-range", start: 311, end: 340 };
const $ssrg$source320 = { file: "main.ssrg", line: 15, column: 14, uri: "seseragi://artifact/show-debug-range", start: 320, end: 339 };
const $ssrg$source327 = { file: "main.ssrg", line: 15, column: 21, uri: "seseragi://artifact/show-debug-range", start: 327, end: 331 };
const $ssrg$source333 = { file: "main.ssrg", line: 15, column: 27, uri: "seseragi://artifact/show-debug-range", start: 333, end: 338 };
$ssrg$assertUnicodeVersion("17.0.0")

export const showRange = (value: Readonly<{ start: number; end: number; inclusive: boolean }>) => $ssrg$atSource($ssrg$source49, () => ((_ssrg_show_rangeShow<number>(_ssrg_show_intShow)["show"](value)) as string), false)
export const debugRange = (value: Readonly<{ start: number; end: number; inclusive: boolean }>) => $ssrg$atSource($ssrg$source111, () => ((_ssrg_debug_rangeDebug<number>(_ssrg_debug_intDebug)["debug"](value)) as string), false)
export const debugRanges = (values: ReadonlyArray<Readonly<{ start: number; end: number; inclusive: boolean }>>) => $ssrg$atSource($ssrg$source183, () => ((_ssrg_debug_arrayDebug<Readonly<{ start: number; end: number; inclusive: boolean }>>(_ssrg_debug_rangeDebug<number>(_ssrg_debug_intDebug))["debug"](values)) as string), false)
export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source229, () => _ssrg_console_println($ssrg$atSource($ssrg$source238, () => ((_ssrg_show_rangeShow<number>(_ssrg_show_intShow)["show"]($ssrg$atSource($ssrg$source244, () => _ssrg_range_exclusive(1, 5), false))) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source255, () => _ssrg_console_println($ssrg$atSource($ssrg$source264, () => ((_ssrg_debug_rangeDebug<number>(_ssrg_debug_intDebug)["debug"]($ssrg$atSource($ssrg$source271, () => _ssrg_range_inclusive(5, 5), false))) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source283, () => _ssrg_console_println($ssrg$atSource($ssrg$source292, () => ((_ssrg_show_rangeShow<number>(_ssrg_show_intShow)["show"]($ssrg$atSource($ssrg$source298, () => _ssrg_range_inclusive(10, 1), false))) as string), false)), true), () => $ssrg$atSource($ssrg$source311, () => _ssrg_console_println($ssrg$atSource($ssrg$source320, () => ((_ssrg_debug_arrayDebug<Readonly<{ start: number; end: number; inclusive: boolean }>>(_ssrg_debug_rangeDebug<number>(_ssrg_debug_intDebug))["debug"]([$ssrg$atSource($ssrg$source327, () => _ssrg_range_exclusive(1, 3), false), $ssrg$atSource($ssrg$source333, () => _ssrg_range_inclusive(3, 3), false)])) as string), false)), true))))
