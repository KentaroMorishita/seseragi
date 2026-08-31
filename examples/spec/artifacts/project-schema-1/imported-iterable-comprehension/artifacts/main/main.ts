import { Countdown, __ssrg$instance$Iterable$0, __ssrg$instance$Reducible$1 } from "./domain.js"
import { multiply as _ssrg_int_multiply, remainder as _ssrg_int_remainder, add as _ssrg_int_add } from "@seseragi/runtime/int"
import { collectMap as _ssrg_iterator_comprehend } from "@seseragi/runtime/iterator"
import { reduce as _ssrg_array_reduce } from "@seseragi/runtime/array"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

const oddSquares = (values: Countdown) => _ssrg_iterator_comprehend(__ssrg$instance$Iterable$0["iterate"](values), (value) => _ssrg_int_remainder(value, 2) !== 0, (value) => _ssrg_int_multiply(value, value))
const addInt = (left: number) => (right: number) => _ssrg_int_add(left, right)
const total = <C,>(values: C) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => __ssrg$evidence$0["reduce"](0)(addInt)(values)
const totals = (values: Countdown) => [_ssrg_array_reduce(0, (_argument0) => (_argument1) => _ssrg_int_add(_argument0, _argument1), oddSquares(values)), total(values)(__ssrg$instance$Reducible$1(__ssrg$instance$Iterable$0))] as const
const renderTotals = (values: readonly [number, number]) => (($ssrg_match: readonly [number, number]): string => $ssrg_match[0] === 35 && $ssrg_match[1] === 15 ? "imported collection totals: 35 / 15" : "unexpected imported collection totals")(values)
export const main = (_unit: undefined) => _ssrg_console_println(renderTotals(totals(Countdown(5))))
