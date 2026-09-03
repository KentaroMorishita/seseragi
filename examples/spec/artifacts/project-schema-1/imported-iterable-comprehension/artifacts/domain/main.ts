import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { unfold as _ssrg_iterator_unfold, type Iterator as Iterator } from "@seseragi/runtime/iterator"
import { reduce as _ssrg_range_reduce, inclusive as _ssrg_range_inclusive } from "@seseragi/runtime/range"
import { Just as _ssrg_maybe_Just, Nothing as _ssrg_maybe_Nothing } from "@seseragi/runtime/sum"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
$ssrg$assertUnicodeVersion("17.0.0")

export type Countdown =
  | { readonly tag: "Countdown"; readonly value: number };
export const Countdown = (value: number): Countdown => ({ tag: "Countdown", value } as const);
export const __ssrg$instance$Iterable$0 = { "iterate": (values: Countdown) => (($ssrg_match: Countdown): Iterator<number> => $ssrg_match.tag === "Countdown" ? ((limit: number): Iterator<number> => _ssrg_iterator_unfold(advance(limit), 1))($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(values) } as const;
export const __ssrg$instance$Reducible$1 = (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => ({ ...__ssrg$evidence$0, "reduce": <B,>(initial: B) => (step: (argument: B) => (argument: number) => B) => (values: Countdown) => (($ssrg_match: Countdown): B => $ssrg_match.tag === "Countdown" ? ((limit: number): B => _ssrg_range_reduce(initial, step, _ssrg_range_inclusive(1, limit)))($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(values) }) as const;
const advance = (limit: number) => (current: number) => current <= limit ? _ssrg_maybe_Just([current, _ssrg_int_add(current, 1)] as const) : _ssrg_maybe_Nothing
