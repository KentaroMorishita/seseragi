import { unfold as _ssrg_iterator_unfold, collectMap as _ssrg_iterator_comprehend, type Iterator as Iterator } from "@seseragi/runtime/iterator"
import { reduce as _ssrg_range_reduce, inclusive as _ssrg_range_inclusive } from "@seseragi/runtime/range"
import { Just as _ssrg_maybe_Just, Nothing as _ssrg_maybe_Nothing } from "@seseragi/runtime/sum"
import { add as _ssrg_int_add, multiply as _ssrg_int_multiply, remainder as _ssrg_int_remainder } from "@seseragi/runtime/int"
import { reduce as _ssrg_array_reduce } from "@seseragi/runtime/array"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

export type Countdown =
  | { readonly tag: "Countdown"; readonly value: number };
export const Countdown = (value: number): Countdown => ({ tag: "Countdown", value } as const);
export const __ssrg$instance$Iterable$0 = { "iterate": (values: Countdown) => (($ssrg_match: Countdown): Iterator<number> => $ssrg_match.tag === "Countdown" ? ((limit: number): Iterator<number> => _ssrg_iterator_unfold(advance(limit), 1))($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(values) } as const;
export const __ssrg$instance$Reducible$1 = { "reduce": <B,>(initial: B) => (step: (argument: B) => (argument: number) => B) => (values: Countdown) => (($ssrg_match: Countdown): B => $ssrg_match.tag === "Countdown" ? ((limit: number): B => _ssrg_range_reduce(initial, step, _ssrg_range_inclusive(1, limit)))($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(values) } as const;
const advance = (limit: number) => (current: number) => current <= limit ? _ssrg_maybe_Just([current, _ssrg_int_add(current, 1)] as const) : _ssrg_maybe_Nothing
const oddSquares = (values: Countdown) => _ssrg_iterator_comprehend(__ssrg$instance$Iterable$0["iterate"](values), (value) => _ssrg_int_remainder(value, 2) !== 0, (value) => _ssrg_int_multiply(value, value))
const addInt = (left: number) => (right: number) => _ssrg_int_add(left, right)
const total = <C,>(values: C) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => __ssrg$evidence$0["reduce"](0)(addInt)(values)
const totals = (values: Countdown) => [_ssrg_array_reduce(0, (_argument0) => (_argument1) => _ssrg_int_add(_argument0, _argument1), oddSquares(values)), total(values)(__ssrg$instance$Reducible$1)] as const
const renderTotals = (values: readonly [number, number]) => (($ssrg_match: readonly [number, number]): string => $ssrg_match[0] === 35 && $ssrg_match[1] === 15 ? "user collection totals: 35 / 15" : "unexpected user collection totals")(values)
export const main = (_unit: undefined) => _ssrg_console_println(renderTotals(totals(Countdown(5))))
