import { unfold as _ssrg_iterator_unfold, type Iterator as Iterator } from "@seseragi/runtime/iterator"
import { reduce as _ssrg_range_reduce, inclusive as _ssrg_range_inclusive } from "@seseragi/runtime/range"
import { Just as _ssrg_maybe_Just, Nothing as _ssrg_maybe_Nothing } from "@seseragi/runtime/sum"
import { add as _ssrg_int64_add } from "@seseragi/runtime/int64"

export type Countdown =
  | { readonly tag: "Countdown"; readonly value: bigint };
export const Countdown = (value: bigint): Countdown => ({ tag: "Countdown", value } as const);
export const __ssrg$instance$Iterable$0 = { "iterate": (values: Countdown) => (($ssrg_match: Countdown): Iterator<bigint> => $ssrg_match.tag === "Countdown" ? ((limit: bigint): Iterator<bigint> => _ssrg_iterator_unfold(advance(limit), 1n))($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(values) } as const;
export const __ssrg$instance$Reducible$1 = { "reduce": <B,>(initial: B) => (step: (argument: B) => (argument: bigint) => B) => (values: Countdown) => (($ssrg_match: Countdown): B => $ssrg_match.tag === "Countdown" ? ((limit: bigint): B => _ssrg_range_reduce(initial, step, _ssrg_range_inclusive(1n, limit)))($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(values) } as const;
const advance = (limit: bigint) => (current: bigint) => current <= limit ? _ssrg_maybe_Just([current, _ssrg_int64_add(current, 1n)] as const) : _ssrg_maybe_Nothing
