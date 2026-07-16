import { unfold as _ssrg_iterator_unfold, collectMap as _ssrg_iterator_comprehend, type Iterator as Iterator } from "@seseragi/runtime/iterator"
import { reduce as _ssrg_range_reduce, inclusive as _ssrg_range_inclusive } from "@seseragi/runtime/range"
import { Just as _ssrg_maybe_Just, Nothing as _ssrg_maybe_Nothing } from "@seseragi/runtime/sum"
import { add as _ssrg_int64_add, multiply as _ssrg_int64_multiply, remainder as _ssrg_int64_remainder } from "@seseragi/runtime/int64"
import { reduce as _ssrg_array_reduce } from "@seseragi/runtime/array"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

export type Countdown =
  | { readonly tag: "Countdown"; readonly value: bigint };
export const Countdown = (value: bigint): Countdown => ({ tag: "Countdown", value } as const);
export const __ssrg$instance$Iterable$0 = { "iterate": (values: Countdown) => (($ssrg_match: Countdown): Iterator<bigint> => $ssrg_match.tag === "Countdown" ? ((limit: bigint): Iterator<bigint> => _ssrg_iterator_unfold(advance(limit), 1n))($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(values) } as const;
export const __ssrg$instance$Reducible$1 = { "reduce": <B,>(initial: B) => (step: (argument: B) => (argument: bigint) => B) => (values: Countdown) => (($ssrg_match: Countdown): B => $ssrg_match.tag === "Countdown" ? ((limit: bigint): B => _ssrg_range_reduce(initial, step, _ssrg_range_inclusive(1n, limit)))($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(values) } as const;
const advance = (limit: bigint) => (current: bigint) => current <= limit ? _ssrg_maybe_Just([current, _ssrg_int64_add(current, 1n)] as const) : _ssrg_maybe_Nothing
const oddSquares = (values: Countdown) => _ssrg_iterator_comprehend(__ssrg$instance$Iterable$0["iterate"](values), (value) => _ssrg_int64_remainder(value, 2n) !== 0n, (value) => _ssrg_int64_multiply(value, value))
const addInt = (left: bigint) => (right: bigint) => _ssrg_int64_add(left, right)
const total = <C,>(values: C) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => __ssrg$evidence$0["reduce"](0n)(addInt)(values)
const totals = (values: Countdown) => [_ssrg_array_reduce(0n, (_argument0) => (_argument1) => _ssrg_int64_add(_argument0, _argument1), oddSquares(values)), total(values)(__ssrg$instance$Reducible$1)] as const
const renderTotals = (values: readonly [bigint, bigint]) => (($ssrg_match: readonly [bigint, bigint]): string => $ssrg_match[0] === 35n && $ssrg_match[1] === 15n ? "user collection totals: 35 / 15" : "unexpected user collection totals")(values)
export const main = (_unit: undefined) => _ssrg_console_println(renderTotals(totals(Countdown(5n))))
