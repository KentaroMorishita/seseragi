import { Countdown, __ssrg$instance$Iterable$0, __ssrg$instance$Reducible$1 } from "./domain.js"
import { multiply as _ssrg_int64_multiply, remainder as _ssrg_int64_remainder, add as _ssrg_int64_add } from "@seseragi/runtime/int64"
import { collectMap as _ssrg_iterator_comprehend } from "@seseragi/runtime/iterator"
import { reduce as _ssrg_array_reduce } from "@seseragi/runtime/array"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

const oddSquares = (values: Countdown) => _ssrg_iterator_comprehend(__ssrg$instance$Iterable$0["iterate"](values), (value) => _ssrg_int64_remainder(value, 2n) !== 0n, (value) => _ssrg_int64_multiply(value, value))
const addInt = (left: bigint) => (right: bigint) => _ssrg_int64_add(left, right)
const total = <C,>(values: C) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => __ssrg$evidence$0["reduce"](0n)(addInt)(values)
const totals = (values: Countdown) => [_ssrg_array_reduce(0n, (_argument0) => (_argument1) => _ssrg_int64_add(_argument0, _argument1), oddSquares(values)), total(values)(__ssrg$instance$Reducible$1)] as const
const renderTotals = (values: readonly [bigint, bigint]) => (($ssrg_match: readonly [bigint, bigint]): string => $ssrg_match[0] === 35n && $ssrg_match[1] === 15n ? "imported collection totals: 35 / 15" : "unexpected imported collection totals")(values)
export const main = (_unit: undefined) => _ssrg_console_println(renderTotals(totals(Countdown(5n))))
