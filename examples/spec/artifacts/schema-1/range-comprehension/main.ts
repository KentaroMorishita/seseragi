import { multiply as _ssrg_int64_multiply, remainder as _ssrg_int64_remainder, add as _ssrg_int64_add } from "@seseragi/runtime/int64"
import { collectMap as _ssrg_range_comprehend, inclusive as _ssrg_range_inclusive } from "@seseragi/runtime/range"
import { reduce as _ssrg_array_reduce } from "@seseragi/runtime/array"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

const evenSquares = (limit: bigint) => _ssrg_range_comprehend(_ssrg_range_inclusive(1n, limit), (value) => _ssrg_int64_remainder(value, 2n) === 0n, (value) => _ssrg_int64_multiply(value, value))
const total = (unit: undefined) => _ssrg_array_reduce(0n, (_argument0) => (_argument1) => _ssrg_int64_add(_argument0, _argument1), evenSquares(10n))
const renderTotal = (total: bigint) => (($ssrg_match: bigint): string => $ssrg_match === 220n ? "even square total: 220" : "unexpected comprehension total")(total)
export const main = (_unit: undefined) => _ssrg_console_println(renderTotal(total(undefined)))
