import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { exclusive as _ssrg_range_exclusive, inclusive as _ssrg_range_inclusive, reduce as _ssrg_range_reduce } from "@seseragi/runtime/range"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
$ssrg$assertUnicodeVersion("17.0.0")

const exclusiveValues = (unit: undefined) => _ssrg_range_exclusive(1, 10)
const inclusiveValues = (unit: undefined) => _ssrg_range_inclusive(1, 10)
const totalsAreCorrect = (unit: undefined) => (($ssrg_match: readonly [number, number]): boolean => $ssrg_match[0] === 45 && $ssrg_match[1] === 55 ? true : false)([_ssrg_range_reduce(0, (_argument0) => (_argument1) => _ssrg_int_add(_argument0, _argument1), exclusiveValues(undefined)), _ssrg_range_reduce(0, (_argument0) => (_argument1) => _ssrg_int_add(_argument0, _argument1), inclusiveValues(undefined))] as const)
const renderResult = (valid: boolean) => (($ssrg_match: boolean): string => $ssrg_match === true ? "exclusive: 45, inclusive: 55" : "unexpected range totals")(valid)
export const main = (_unit: undefined) => _ssrg_console_println(renderResult(totalsAreCorrect(undefined)))
