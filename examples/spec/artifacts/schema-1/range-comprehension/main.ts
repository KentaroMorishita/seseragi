import { multiply as _ssrg_int_multiply, remainder as _ssrg_int_remainder, add as _ssrg_int_add } from "@seseragi/runtime/int"
import { collectMap as _ssrg_range_comprehend, inclusive as _ssrg_range_inclusive } from "@seseragi/runtime/range"
import { reduce as _ssrg_array_reduce } from "@seseragi/runtime/array"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

const evenSquares = (limit: number) => _ssrg_range_comprehend(_ssrg_range_inclusive(1, limit), (value) => _ssrg_int_remainder(value, 2) === 0, (value) => _ssrg_int_multiply(value, value))
const total = (unit: undefined) => _ssrg_array_reduce(0, (_argument0) => (_argument1) => _ssrg_int_add(_argument0, _argument1), evenSquares(10))
const renderTotal = (total: number) => (($ssrg_match: number): string => $ssrg_match === 220 ? "even square total: 220" : "unexpected comprehension total")(total)
export const main = (_unit: undefined) => _ssrg_console_println(renderTotal(total(undefined)))
