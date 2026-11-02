import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { exclusive as _ssrg_range_exclusive, inclusive as _ssrg_range_inclusive, reduce as _ssrg_range_reduce } from "@seseragi/runtime/range"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source46 = { file: "main.ssrg", line: 1, column: 47, uri: "seseragi://artifact/range-reduce", start: 46, end: 51 };
const $ssrg$source99 = { file: "main.ssrg", line: 3, column: 47, uri: "seseragi://artifact/range-reduce", start: 99, end: 105 };
const $ssrg$source162 = { file: "main.ssrg", line: 7, column: 5, uri: "seseragi://artifact/range-reduce", start: 162, end: 180 };
const $ssrg$source202 = { file: "main.ssrg", line: 8, column: 5, uri: "seseragi://artifact/range-reduce", start: 202, end: 220 };
const $ssrg$source450 = { file: "main.ssrg", line: 21, column: 3, uri: "seseragi://artifact/range-reduce", start: 450, end: 469 };
$ssrg$assertUnicodeVersion("17.0.0")

const exclusiveValues = (unit: undefined) => $ssrg$atSource($ssrg$source46, () => _ssrg_range_exclusive(1, 10), false)
const inclusiveValues = (unit: undefined) => $ssrg$atSource($ssrg$source99, () => _ssrg_range_inclusive(1, 10), false)
const totalsAreCorrect = (unit: undefined) => (($ssrg_match: readonly [number, number]): boolean => $ssrg_match[0] === 45 && $ssrg_match[1] === 55 ? true : false)([$ssrg$atSource($ssrg$source162, () => _ssrg_range_reduce(0, (((_argument0) => (_argument1) => _ssrg_int_add(_argument0, _argument1)) as (argument: number) => (argument: number) => number), $ssrg$atSource($ssrg$source162, () => exclusiveValues(undefined), false)), false), $ssrg$atSource($ssrg$source202, () => _ssrg_range_reduce(0, (((_argument0) => (_argument1) => _ssrg_int_add(_argument0, _argument1)) as (argument: number) => (argument: number) => number), $ssrg$atSource($ssrg$source202, () => inclusiveValues(undefined), false)), false)] as const)
const renderResult = (valid: boolean) => (($ssrg_match: boolean): string => $ssrg_match === true ? "exclusive: 45, inclusive: 55" : "unexpected range totals")(valid)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source450, () => _ssrg_console_println($ssrg$atSource($ssrg$source450, () => renderResult($ssrg$atSource($ssrg$source450, () => totalsAreCorrect(undefined), false)), false)), true)
