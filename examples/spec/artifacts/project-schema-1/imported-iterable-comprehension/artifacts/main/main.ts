import { Countdown, __ssrg$instance$Iterable$0 } from "./domain.js"
import { multiply as _ssrg_int64_multiply, remainder as _ssrg_int64_remainder, add as _ssrg_int64_add } from "@seseragi/runtime/int64"
import { collectMap as _ssrg_iterator_comprehend } from "@seseragi/runtime/iterator"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { reduce as _ssrg_array_reduce } from "@seseragi/runtime/array"

const oddSquares = (values: Countdown) => _ssrg_iterator_comprehend(__ssrg$instance$Iterable$0["iterate"](values), (value) => _ssrg_int64_remainder(value, 2n) !== 0n, (value) => _ssrg_int64_multiply(value, value))
const renderTotal = (total: bigint) => (($ssrg_match: bigint): string => $ssrg_match === 35n ? "imported Iterable total: 35" : "unexpected imported Iterable total")(total)
export const main = (_unit: undefined) => _ssrg_console_println(renderTotal(_ssrg_array_reduce(0n, (_argument0) => (_argument1) => _ssrg_int64_add(_argument0, _argument1), oddSquares(Countdown(5n)))))
