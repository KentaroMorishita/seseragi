import { unfold as _ssrg_iterator_unfold, collectMap as _ssrg_iterator_comprehend, type Iterator as Iterator } from "@seseragi/runtime/iterator"
import { Just as _ssrg_maybe_Just, Nothing as _ssrg_maybe_Nothing } from "@seseragi/runtime/sum"
import { add as _ssrg_int64_add, multiply as _ssrg_int64_multiply, remainder as _ssrg_int64_remainder } from "@seseragi/runtime/int64"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { reduce as _ssrg_array_reduce } from "@seseragi/runtime/array"

export type Countdown =
  | { readonly tag: "Countdown"; readonly value: bigint };
export const Countdown = (value: bigint): Countdown => ({ tag: "Countdown", value } as const);
export const __ssrg$instance$Iterable$0 = { "iterate": (values: Countdown) => (($ssrg_match: Countdown): Iterator<bigint> => $ssrg_match.tag === "Countdown" ? ((limit: bigint): Iterator<bigint> => _ssrg_iterator_unfold(advance(limit), 1n))($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(values) } as const;
const advance = (limit: bigint) => (current: bigint) => current <= limit ? _ssrg_maybe_Just([current, _ssrg_int64_add(current, 1n)] as const) : _ssrg_maybe_Nothing
const oddSquares = (values: Countdown) => _ssrg_iterator_comprehend(__ssrg$instance$Iterable$0["iterate"](values), (value) => _ssrg_int64_remainder(value, 2n) !== 0n, (value) => _ssrg_int64_multiply(value, value))
const renderTotal = (total: bigint) => (($ssrg_match: bigint): string => $ssrg_match === 35n ? "user Iterable total: 35" : "unexpected user Iterable total")(total)
export const main = (_unit: undefined) => _ssrg_console_println(renderTotal(_ssrg_array_reduce(0n, (_argument0) => (_argument1) => _ssrg_int64_add(_argument0, _argument1), oddSquares(Countdown(5n)))))
